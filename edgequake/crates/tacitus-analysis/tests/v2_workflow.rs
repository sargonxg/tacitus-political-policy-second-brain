use std::collections::BTreeMap;
use tacitus_analysis::v2_workflow::*;
use tacitus_capsules::policy::PolicyPackage;
fn package(raw: &str) -> PolicyPackage {
    let v: serde_json::Value = serde_json::from_str(raw).unwrap();
    PolicyPackage {
        manifest: serde_json::from_value(v["manifest"].clone()).unwrap(),
        data: serde_json::from_value(v["data"].clone()).unwrap(),
        evidence: v["evidence"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str().unwrap().as_bytes().to_vec()))
            .collect::<BTreeMap<_, _>>(),
    }
}
const HEAT: &str = include_str!("fixtures/v2/nyc-heat-resilience.native-v2-input.json");
const COMPUTE: &str =
    include_str!("fixtures/v2/compute-infrastructure-governance.native-v2-input.json");
const ACCESS: &str = include_str!("fixtures/v2/humanitarian-access-analysis.native-v2-input.json");
fn request(p: &PolicyPackage) -> Request {
    Request {
        question: "What does this evidence establish?".into(),
        known_as_of: p.manifest.created_at,
        valid_at: 0,
        time_mode: TimeMode::DiscloseUnknown,
        max_bytes: 200000,
    }
}
#[test]
fn three_corrected_capsules_import_inspect_context_document() {
    for raw in [HEAT, COMPUTE, ACCESS] {
        let p = package(raw);
        let mut c = LocalCatalog::default();
        let d = c.import_native_v2(&p.export().unwrap()).unwrap();
        assert_eq!(d.schema_version, 2);
        assert_eq!(d.assertions, 5);
        assert_eq!(c.search("").unwrap().len(), 1);
        assert_eq!(
            c.inspect(&d.archive_digest).unwrap().archive_digest,
            d.archive_digest
        );
        let e = c.export_review(&d.archive_digest, &request(&p)).unwrap();
        assert_eq!(e.document.layered_context.packet.claims.len(), 5);
        assert_eq!(e.document.artifacts, p.data.artifacts);
        assert!(e.document.temporal_status.contains("UNKNOWN"));
        assert!(e.document.layered_context.packet.claims.values().all(|a| a
            .assertion
            .conditions
            .iter()
            .any(|s| s.contains("World-validity interval is unknown"))));
        let t = render_text(&e, 500000).unwrap();
        assert!(t.contains("SYNTHETIC ILLUSTRATIVE"));
        assert!(t.contains("UNKNOWN WORLD VALIDITY"));
        assert!(t.contains("COMPLETE QUALIFIED APPENDIX"));
    }
}
#[test]
fn date_verified_mode_rejects_unknown() {
    let p = package(HEAT);
    let mut c = LocalCatalog::default();
    let d = c.import_native_v2(&p.export().unwrap()).unwrap();
    let mut r = request(&p);
    r.time_mode = TimeMode::RequireDeclaredInterval;
    assert!(c.export_review(&d.archive_digest, &r).is_err());
}
#[test]
fn revocation_blocks_entire_opaque_output() {
    let p = package(ACCESS);
    let mut c = LocalCatalog::default();
    let d = c.import_native_v2(&p.export().unwrap()).unwrap();
    c.revoke_source(&d.archive_digest, &p.data.sources[0].key())
        .unwrap();
    assert!(c.export_review(&d.archive_digest, &request(&p)).is_err());
    assert!(c.search("").unwrap().is_empty());
    assert!(c
        .inspect(&d.archive_digest)
        .unwrap()
        .state
        .starts_with("revoked"));
}
#[test]
fn reimport_cannot_clear_revocation() {
    let p = package(HEAT);
    let bytes = p.export().unwrap();
    let mut c = LocalCatalog::default();
    let d = c.import_native_v2(&bytes).unwrap();
    c.revoke(&d.archive_digest).unwrap();
    assert!(c
        .import_native_v2(&bytes)
        .unwrap()
        .state
        .starts_with("revoked"));
    assert!(c.export_review(&d.archive_digest, &request(&p)).is_err());
}
#[test]
fn duplicate_version_different_content_rejected() {
    let mut p = package(HEAT);
    let mut c = LocalCatalog::default();
    c.import_native_v2(&p.export().unwrap()).unwrap();
    p.manifest.title.push('!');
    assert!(c.import_native_v2(&p.export().unwrap()).is_err());
}
#[test]
fn future_checkpoint_and_budget_fail_closed() {
    let p = package(COMPUTE);
    let mut c = LocalCatalog::default();
    let d = c.import_native_v2(&p.export().unwrap()).unwrap();
    let mut r = request(&p);
    let full = c.export_review(&d.archive_digest, &r).unwrap();
    r.max_bytes = full.bytes - 1;
    assert!(c.export_review(&d.archive_digest, &r).is_err());
    r = request(&p);
    r.known_as_of -= 1;
    assert!(c.export_review(&d.archive_digest, &r).is_err());
}
#[test]
fn deterministic_and_mutation_checked() {
    let p = package(HEAT);
    let mut c = LocalCatalog::default();
    let d = c.import_native_v2(&p.export().unwrap()).unwrap();
    let mut a = c.export_review(&d.archive_digest, &request(&p)).unwrap();
    let b = c.export_review(&d.archive_digest, &request(&p)).unwrap();
    assert_eq!(a.digest, b.digest);
    a.document.temporal_status = "verified".into();
    assert!(render_text(&a, 500000).is_err());
}
#[test]
fn missing_authority_is_never_promoted() {
    let p = package(HEAT);
    let mut c = LocalCatalog::default();
    let d = c.import_native_v2(&p.export().unwrap()).unwrap();
    let e = c.export_review(&d.archive_digest, &request(&p)).unwrap();
    assert!(e.document.review_state.starts_with("unreviewed"));
    assert!(e
        .document
        .layered_context
        .packet
        .claims
        .values()
        .all(|a| matches!(a.knowledge, tacitus_analysis::layers::Knowledge::Unknown)));
}
#[test]
fn no_automatic_native_v1_migration() {
    let value: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/native-v1-legacy.json")).unwrap();
    let v = &value["inputs"];
    let p = tacitus_capsules::Package {
        manifest: serde_json::from_value(v["manifest"].clone()).unwrap(),
        data: serde_json::from_value(v["data"].clone()).unwrap(),
        evidence: v["evidence"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str().unwrap().as_bytes().to_vec()))
            .collect(),
    };
    let bytes = p.export().unwrap();
    assert_eq!(
        tacitus_capsules::policy::PolicyPackage::import(&bytes)
            .unwrap()
            .manifest
            .schema_version,
        2
    );
    assert!(
        tacitus_capsules::policy::PolicyPackage::import_strict(&bytes)
            .unwrap_err()
            .contains("no implicit migration")
    );
    assert!(LocalCatalog::default()
        .import_native_v2(&bytes)
        .unwrap_err()
        .contains("no implicit migration"));
}

#[test]
fn temporal_disclosure_derived_without_mutating_assertions() {
    let mut p = package(HEAT);
    for a in &mut p.data.assertions {
        a.conditions
            .retain(|c| !c.contains("World-validity interval is unknown"));
    }
    let fingerprints: BTreeMap<_, _> = p
        .data
        .assertions
        .iter()
        .map(|a| (a.id.clone(), a.fingerprint()))
        .collect();
    let mut c = LocalCatalog::default();
    let d = c.import_native_v2(&p.export().unwrap()).unwrap();
    let e = c.export_review(&d.archive_digest, &request(&p)).unwrap();
    assert!(e
        .document
        .claim_temporal_status
        .values()
        .all(|s| s.starts_with("UNKNOWN")));
    for (id, a) in &e.document.layered_context.packet.claims {
        assert_eq!(fingerprints[id], a.assertion.fingerprint());
    }
    for s in e.document.source_metadata.values() {
        assert!(s.text.is_empty());
        assert!(s.valid_from.is_none());
        assert!(s.valid_until.is_none());
        assert!(s.available_at > 0);
    }
    assert!(render_text(&e, 500000)
        .unwrap()
        .contains("Temporal qualification: UNKNOWN"));
}
