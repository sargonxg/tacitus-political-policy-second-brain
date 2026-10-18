use std::collections::{BTreeMap, BTreeSet};
use tacitus_analysis::cited_brief::{compile, Request};
use tacitus_capsules::{evidence_name, CapsuleData, Manifest, Package};
use tacitus_core::*;
fn source() -> SourceVersion {
    let text = "The fictional Alder delegation may discuss participation, subject to Council approval. Budget authorization is not expenditure.".to_string();
    SourceVersion {
        id: "alder-statement".into(),
        version: 1,
        workspace: "first".into(),
        title: "Synthetic conditional statement".into(),
        author: "Fictional delegation".into(),
        original_hash: digest(text.as_bytes()),
        text_hash: digest(text.as_bytes()),
        text,
        language: "en".into(),
        extraction_tool: "synthetic-utf8/1".into(),
        published_at: 10,
        retrieved_at: 12,
        available: Interval {
            start: 12,
            end: None,
        },
        valid: Interval {
            start: 10,
            end: None,
        },
        source_family: "alder-original".into(),
        withdrawn: false,
        restricted: false,
        redistributable: true,
        license: "CC0-1.0".into(),
    }
}
fn assertion(s: &SourceVersion) -> Assertion {
    Assertion {
        id: "claim-1".into(),
        version: 1,
        workspace: s.workspace.clone(),
        profile: "position".into(),
        speaker: s.author.clone(),
        proposition: "Delegation expressed conditional willingness to discuss.".into(),
        subject: "Alder delegation".into(),
        object: None,
        epistemic_type: EpistemicType::ReportedStatement,
        polarity: "positive".into(),
        modality: "may".into(),
        conditions: vec!["Council approval".into()],
        exceptions: vec![],
        audience: "public".into(),
        speech_act: "statement".into(),
        valid: s.valid.clone(),
        available: s.available.clone(),
        evidence: vec![Span {
            source: s.key(),
            text_hash: s.text_hash.clone(),
            start: 0,
            end: s.text.len(),
            locator: Some("paragraph 1".into()),
            original_language: "en".into(),
            translation: None,
        }],
        dependencies: vec![],
        challenges: vec![],
        supersedes: None,
        confidence: "source wording explicit".into(),
        confidence_rationale: "Establishes statement, not delivered participation.".into(),
    }
}
fn package() -> Package {
    let s = source();
    let a = assertion(&s);
    let review = ReviewReceipt {
        assertion_id: a.id.clone(),
        assertion_fingerprint: a.fingerprint(),
        evidence_hashes: BTreeMap::from([(s.key(), s.text_hash.clone())]),
        reviewer: "synthetic-test-reviewer".into(),
        rationale: "Test action only".into(),
        reviewed_at: 20,
        decision: ReviewDecision::Caveat,
        origin: ReviewOrigin::SyntheticTest,
    };
    let evidence = BTreeMap::from([(evidence_name(&s), s.text.as_bytes().to_vec())]);
    let mut metadata = s;
    metadata.text.clear();
    Package {
        manifest: Manifest {
            schema_version: 1,
            package_version: 1,
            id: "synthetic-capsule".into(),
            author: "Fictional analyst".into(),
            title: "Conditional participation".into(),
            domain: "policy".into(),
            scope: "fictional pilot".into(),
            purpose: "regression".into(),
            profile: "context".into(),
            created_at: 20,
            valid_until: None,
            supersedes: None,
            dependencies: vec![],
            license: "CC0-1.0".into(),
            confidentiality: "public".into(),
            sharing_policy: "intentional synthetic test export".into(),
            evidence_mode: "evidence_included".into(),
            limitations: vec![],
            files: BTreeMap::new(),
        },
        data: CapsuleData {
            assertions: vec![a],
            sources: vec![metadata],
            methods: vec![],
            reviews: vec![review],
            artifacts: BTreeMap::from([(
                "brief".into(),
                "Participation is conditional; appropriation is unresolved.".into(),
            )]),
        },
        evidence,
    }
}

fn request() -> Request {
    Request {
        question: "Has the delegation committed?".into(),
        workspace: "review".into(),
        known_as_of: 30,
        valid_at: 30,
        priority: vec!["claim-1".into()],
        max_bytes: 20000,
        revoked_sources: BTreeSet::new(),
        package_revoked: false,
    }
}
#[test]
fn preserves_qualified_claim_and_exact_excerpt() {
    let p = package();
    let out = compile(&p, &request()).unwrap();
    assert_eq!(
        out.brief.entries[0].assertion.conditions,
        vec!["Council approval"]
    );
    assert_eq!(out.brief.entries[0].citations[0].excerpt, source().text);
    assert!(out.brief.entries[0].citations[0].source.text.is_empty());
    assert_eq!(out.brief.entries[0].assertion.workspace, "review");
}
#[test]
fn deterministic_exact_budget() {
    let p = package();
    let r = request();
    let a = compile(&p, &r).unwrap();
    let b = compile(&p, &r).unwrap();
    assert_eq!(a.digest, b.digest);
    assert_eq!(a.bytes, serde_json::to_vec(&a.brief).unwrap().len());
    let mut r = r;
    r.max_bytes = a.bytes;
    assert_eq!(compile(&p, &r).unwrap().brief.entries.len(), 1);
    r.max_bytes -= 1;
    let mut p = p;
    p.data.artifacts.clear();
    r.max_bytes = compile(&p, &request()).unwrap().bytes - 1;
    let short = compile(&p, &r).unwrap();
    assert!(short.brief.entries.is_empty());
    assert!(short.bytes <= r.max_bytes);
    assert!(short.brief.omissions.contains_key("claim-1"));
}
#[test]
fn revocation_rechecked() {
    let p = package();
    let mut r = request();
    r.revoked_sources.insert(source().key());
    let mut p = p;
    p.data.artifacts.clear();
    assert!(compile(&p, &r).unwrap().brief.entries.is_empty());
    r.package_revoked = true;
    assert!(compile(&p, &r).is_err());
}
#[test]
fn future_and_expired_excluded() {
    let mut p = package();
    let mut r = request();
    r.known_as_of = 19;
    assert!(compile(&p, &r).is_err());
    r.known_as_of = 30;
    p.data.assertions[0].valid.end = Some(25);
    p.data.artifacts.clear();
    assert!(compile(&p, &r).unwrap().brief.entries.is_empty());
}
#[test]
fn relations_fail_closed() {
    let mut p = package();
    p.data.assertions[0].dependencies.push("absent".into());
    assert!(compile(&p, &request()).is_err());
    p.data.assertions[0].dependencies.clear();
    p.data.assertions[0].challenges.push("claim-1".into());
    assert!(compile(&p, &request()).is_err());
}
#[test]
fn duplicate_and_unknown_priority_rejected() {
    let p = package();
    let mut r = request();
    r.priority.push("claim-1".into());
    assert!(compile(&p, &r).is_err());
    r.priority = vec!["absent".into()];
    assert!(compile(&p, &r).is_err());
}
#[test]
fn tampering_rejected() {
    let mut p = package();
    p.evidence.values_mut().next().unwrap().push(b'!');
    assert!(compile(&p, &request()).is_err());
}
#[test]
fn references_only_fail_without_excerpt() {
    let mut p = package();
    p.manifest.evidence_mode = "references_only".into();
    p.manifest.limitations.push("No included evidence".into());
    p.evidence.clear();
    assert!(compile(&p, &request()).is_err());
}
#[test]
fn attribution_cannot_be_silently_truncated() {
    let mut p = package();
    p.data
        .artifacts
        .insert("limitation".into(), "x".repeat(5000));
    let mut r = request();
    r.max_bytes = 1024;
    assert!(compile(&p, &r).is_err());
}

#[test]
fn document_retains_appendix_and_independent_budget() {
    let out = compile(&package(), &request()).unwrap();
    let t = tacitus_analysis::cited_brief::render_text(&out, 50000).unwrap();
    assert!(t.contains("Council approval"));
    assert!(t.contains("Complete qualified evidence"));
    assert!(tacitus_analysis::cited_brief::render_text(&out, t.len() - 1).is_err());
    assert_eq!(
        t,
        tacitus_analysis::cited_brief::render_text(&out, 50000).unwrap()
    );
}
#[test]
fn mutated_export_not_rendered_as_verified() {
    let mut out = compile(&package(), &request()).unwrap();
    out.brief.question = "changed".into();
    assert!(tacitus_analysis::cited_brief::render_text(&out, 50000).is_err());
}

#[test]
fn opaque_artifacts_fail_closed_on_revocation_budget_or_time() {
    let p = package();
    let mut r = request();
    r.revoked_sources.insert(source().key());
    assert!(compile(&p, &r).is_err());
    r = request();
    r.max_bytes = compile(&p, &r).unwrap().bytes - 1;
    assert!(compile(&p, &r).is_err());
    let mut p = p;
    p.data.assertions[0].valid.end = Some(25);
    assert!(compile(&p, &request()).is_err());
}
#[test]
fn opaque_methods_fail_closed_on_revocation() {
    let mut p = package();
    p.data.artifacts.clear();
    p.data.methods.push(tacitus_capsules::Method {
        id: "guard".into(),
        version: 1,
        kind: "procedure".into(),
        description: "Sensitive source summary".into(),
        scope: "test".into(),
        failure_conditions: vec!["Unavailable evidence".into()],
        provenance: vec![source().key()],
        review_state: "proposed".into(),
    });
    let mut r = request();
    r.revoked_sources.insert(source().key());
    assert!(compile(&p, &r).is_err());
}

#[test]
fn real_humanitarian_opaque_provenance_cannot_leak_after_revocation() {
    let value: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/humanitarian-access.exchange.json")).unwrap();
    let v = &value["inputs"];
    let p = Package {
        manifest: serde_json::from_value(v["manifest"].clone()).unwrap(),
        data: serde_json::from_value(v["data"].clone()).unwrap(),
        evidence: v["evidence"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str().unwrap().as_bytes().to_vec()))
            .collect(),
    };
    let mut r = Request {
        question: "Which access constraints are documented?".into(),
        workspace: "review".into(),
        known_as_of: p.manifest.created_at,
        valid_at: p.manifest.created_at,
        priority: p.data.assertions.iter().map(|a| a.id.clone()).collect(),
        max_bytes: 80000,
        revoked_sources: BTreeSet::new(),
        package_revoked: false,
    };
    let original = compile(&p, &r).unwrap();
    assert!(original
        .brief
        .artifacts
        .values()
        .any(|v| v.contains("access-law")));
    assert!(original
        .brief
        .methods
        .iter()
        .all(|m| m.review_state == "unreviewed_proposal"));
    r.revoked_sources.insert("summary-access-law@1".into());
    assert!(compile(&p, &r).is_err());
    r.revoked_sources.clear();
    r.priority.remove(0);
    assert!(compile(&p, &r).is_err());
    r.priority = p.data.assertions.iter().map(|a| a.id.clone()).collect();
    r.max_bytes = original.bytes - 1;
    assert!(compile(&p, &r).is_err());
}
#[test]
fn trusted_method_label_is_downgraded_for_direct_callers() {
    let mut p = package();
    p.data.methods.push(tacitus_capsules::Method {
        id: "method".into(),
        version: 1,
        kind: "procedure".into(),
        description: "Review all qualifications".into(),
        scope: "test".into(),
        failure_conditions: vec!["Missing evidence".into()],
        provenance: vec![source().key()],
        review_state: "accepted_canonical".into(),
    });
    let out = compile(&p, &request()).unwrap();
    assert_eq!(out.brief.methods[0].review_state, "unreviewed_proposal");
    assert!(!serde_json::to_string(&out)
        .unwrap()
        .contains("accepted_canonical"));
}
