use serde_json::{json, Value};
use tacitus_analysis::{extraction_review::*, knowledge_capture};
use tacitus_capsules::policy::PolicyPackage;
use tacitus_core::policy::SourceRecord;
const RAW: &[u8] = include_bytes!("fixtures/extraction/proposal.json");
fn docs() -> Vec<SourceRecord> {
    serde_json::from_slice(include_bytes!("fixtures/extraction/current-sources.json")).unwrap()
}
fn decision() -> Decision {
    serde_json::from_slice(include_bytes!("fixtures/extraction/local-decision.json")).unwrap()
}
fn modified(f: impl FnOnce(&mut Value)) -> Vec<u8> {
    let mut v: Value = serde_json::from_slice(RAW).unwrap();
    f(&mut v);
    serde_json::to_vec(&v).unwrap()
}
#[test]
fn reviewed_proposal_promotes_and_reuses_without_imported_authority() {
    let p = Proposal::parse(RAW).unwrap();
    let d = decision();
    let actor = d.actor.clone();
    let grant = p.acknowledge(&docs(), d).unwrap();
    let out = p.promote(grant, &docs(), 300, &actor).unwrap();
    let native = PolicyPackage::import_strict(&out.archive).unwrap();
    assert!(native.data.reviews.is_empty());
    let context: Value = serde_json::from_slice(
        &knowledge_capture::reuse(
            &out.archive,
            include_bytes!("fixtures/knowledge/port.query.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(context["context"]["claims"].as_object().unwrap().len(), 2);
    assert!(
        String::from_utf8(serde_json::to_vec(&native.data.artifacts).unwrap())
            .unwrap()
            .contains("process-local")
    );
    assert_eq!(out.receipt.proposal_digest, p.digest());
}
#[test]
fn same_text_metadata_and_membership_changes_invalidate_review() {
    let p = Proposal::parse(RAW).unwrap();
    for kind in 0..8 {
        let grant = p.acknowledge(&docs(), decision()).unwrap();
        let mut current = docs();
        match kind {
            0 => current[0].license = Some("changed".into()),
            1 => current[0].withdrawn = true,
            2 => current[0].restricted = true,
            3 => current[0].available_at += 1,
            4 => current[0].version += 1,
            5 => {
                current.pop();
            }
            6 => {
                let mut x = current[0].clone();
                x.id = "extra".into();
                current.push(x);
            }
            _ => current.push(current[0].clone()),
        };
        assert!(
            p.promote(grant, &current, 300, "synthetic-local-operator")
                .is_err(),
            "case {kind}"
        );
    }
}
#[test]
fn order_is_irrelevant_but_exact_proposal_bytes_and_actor_are_bound() {
    let p = Proposal::parse(RAW).unwrap();
    let grant = p.acknowledge(&docs(), decision()).unwrap();
    let mut reversed = docs();
    reversed.reverse();
    assert!(p
        .promote(grant, &reversed, 300, "synthetic-local-operator")
        .is_ok());
    let grant = p.acknowledge(&docs(), decision()).unwrap();
    let mut raw = RAW.to_vec();
    raw.push(b' ');
    let different = Proposal::parse(&raw).unwrap();
    assert!(different
        .promote(grant, &docs(), 300, "synthetic-local-operator")
        .is_err());
    let grant = p.acknowledge(&docs(), decision()).unwrap();
    assert!(p.promote(grant, &docs(), 300, "another-operator").is_err());
}
#[test]
fn malformed_or_hallucinated_span_and_unresolved_entity_rejected() {
    for raw in [
        modified(|v| v["entity_mentions"]["eva"][0]["quote"] = json!("invented quote")),
        modified(|v| v["entity_mentions"]["eva"][0]["span"]["start"] = json!(80)),
        modified(|v| v["entity_mentions"]["eva"][0]["span"]["original_language"] = json!("fr")),
        modified(|v| {
            v["entity_mentions"]
                .as_object_mut()
                .unwrap()
                .remove("council");
        }),
        modified(|v| v["capture"]["graph"]["nodes"]["eva"]["claims"] = json!(["missing"])),
        modified(|v| v["capture"]["assertions"][0]["evidence"][0]["end"] = json!(99999)),
    ] {
        assert!(Proposal::parse(&raw).is_err());
    }
}
#[test]
fn every_entity_linked_claim_needs_mention_and_corrections_must_resolve() {
    assert!(Proposal::parse(&modified(|v| {
        v["entity_mentions"]["council"]
            .as_array_mut()
            .unwrap()
            .pop();
    }))
    .is_err());
    assert!(Proposal::parse(&modified(
        |v| v["capture"]["graph"]["links"][0]["from"] = json!("missing")
    ))
    .is_err());
    assert!(Proposal::parse(&modified(|v| {
        v["capture"]["graph"]["links"][0]["from"] = json!("access");
        v["capture"]["graph"]["links"][0]["to"] = json!("withdrawal");
    }))
    .is_err());
}
#[test]
fn candidate_cannot_claim_review_or_settled_truth() {
    for raw in [
        modified(|v| v["capture"]["confirms_rights"] = json!(true)),
        modified(|v| v["capture"]["graph"]["knowledge"]["access"] = json!("reported_true")),
        modified(|v| v["review_grant"] = json!(true)),
    ] {
        assert!(Proposal::parse(&raw).is_err());
    }
    let p = Proposal::parse(RAW).unwrap();
    for kind in 0..5 {
        let mut d = decision();
        match kind {
            0 => d.acknowledges_corrections = false,
            1 => d.reviewed_manifest_and_source_metadata = false,
            2 => d.confirms_rights = false,
            3 => d.workspace = "other".into(),
            _ => d.reviewed_all_claims_entities_relations_and_times = false,
        };
        assert!(p.acknowledge(&docs(), d).is_err());
    }
}
#[test]
fn temporal_inconsistency_and_future_review_refuse() {
    for raw in [
        modified(|v| v["extractor"]["produced_at"] = json!(301)),
        modified(|v| v["capture"]["documents"][0]["valid_from"] = json!(400)),
        modified(|v| {
            v["capture"]["documents"][0]["valid_from"] = json!(200);
            v["capture"]["documents"][0]["valid_until"] = json!(100);
        }),
    ] {
        match Proposal::parse(&raw) {
            Err(_) => {}
            Ok(p) => {
                let mut d = decision();
                d.proposal_digest = p.digest();
                let e: Envelope = serde_json::from_slice(&raw).unwrap();
                assert!(p.acknowledge(&e.capture.documents, d).is_err());
            }
        }
    }
    let p = Proposal::parse(RAW).unwrap();
    let mut d = decision();
    d.reviewed_at = 299;
    assert!(p.acknowledge(&docs(), d).is_err());
    let mut d = decision();
    d.reviewed_at = 301;
    let grant = p.acknowledge(&docs(), d).unwrap();
    assert!(p
        .promote(grant, &docs(), 300, "synthetic-local-operator")
        .is_err());
}
#[test]
fn duplicate_input_keys_and_sources_reject() {
    assert!(Proposal::parse(&modified(|v| {
        let d = v["capture"]["documents"][0].clone();
        v["capture"]["documents"].as_array_mut().unwrap().push(d);
    }))
    .is_err());
    let s = String::from_utf8(RAW.to_vec()).unwrap().replacen(
        "\"format\":",
        "\"format\":\"bad\",\"format\":",
        1,
    );
    assert!(Proposal::parse(s.as_bytes()).is_err());
}
#[test]
fn review_metadata_is_not_available_before_its_recording_checkpoint() {
    let p = Proposal::parse(RAW).unwrap();
    let mut d = decision();
    d.reviewed_at = 301;
    let g = p.acknowledge(&docs(), d).unwrap();
    let out = p
        .promote(g, &docs(), 301, "synthetic-local-operator")
        .unwrap();
    let native = PolicyPackage::import_strict(&out.archive).unwrap();
    assert_eq!(native.manifest.created_at, 301);
    assert!(knowledge_capture::reuse(
        &out.archive,
        include_bytes!("fixtures/knowledge/port.query.json")
    )
    .is_err());
}
