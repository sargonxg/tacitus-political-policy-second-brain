use std::collections::BTreeMap;
use tacitus_capsules::{
    policy::{evidence_name, Data, PolicyPackage},
    Manifest,
};
use tacitus_core::{digest, policy::*, Span};
fn package() -> PolicyPackage {
    let text = "Authorization is not expenditure.".to_string();
    let mut s = SourceRecord {
        id: "source".into(),
        version: 1,
        workspace: "w".into(),
        document_id: None,
        title: "Synthetic public fixture".into(),
        author: Some("Council".into()),
        original_hash: None,
        text_hash: digest(text.as_bytes()),
        text: text.clone(),
        language: "en".into(),
        extractor: "test".into(),
        published_at: Some(1),
        retrieved_at: 2,
        available_at: 2,
        valid_from: None,
        valid_until: None,
        source_family: "one-origin".into(),
        withdrawn: false,
        restricted: false,
        redistributable: true,
        license: Some("CC0".into()),
    };
    let span = Span {
        source: s.key(),
        text_hash: s.text_hash.clone(),
        start: 0,
        end: text.len(),
        locator: None,
        original_language: "en".into(),
        translation: None,
    };
    let m = CognitiveMethod {
        id: "method".into(),
        version: 1,
        workspace: "w".into(),
        kind: "distinction".into(),
        description: text.clone(),
        scope: "Budget-stage analysis".into(),
        prerequisites: vec!["scope_confirmed".into()],
        failure_conditions: vec!["Missing budget records".into()],
        evidence: vec![span.clone()],
        created_at: 3,
        available_at: 3,
        ontology_version: ONTOLOGY_VERSION.into(),
    };
    let review = HumanReview {
        id: "review".into(),
        target_id: m.id.clone(),
        target_kind: "method".into(),
        target_fingerprint: m.fingerprint(),
        evidence_hashes: BTreeMap::from([(span.source, span.text_hash)]),
        reviewer: "reviewer".into(),
        decision: "accept".into(),
        rationale: "Read the source".into(),
        reviewed_at: 4,
        origin: "authenticated_human".into(),
    };
    let evidence = BTreeMap::from([(evidence_name(&s), text.into_bytes())]);
    s.text.clear();
    let manifest = Manifest {
        schema_version: 2,
        package_version: 1,
        id: "capsule".into(),
        author: "analyst".into(),
        title: "Budget stages".into(),
        domain: "policy".into(),
        scope: "w".into(),
        purpose: "Budget method reuse".into(),
        profile: "cognitive_method".into(),
        created_at: 5,
        valid_until: None,
        supersedes: None,
        dependencies: vec![],
        license: "CC0".into(),
        confidentiality: "public".into(),
        sharing_policy: "Public synthetic evidence only".into(),
        evidence_mode: "evidence_included".into(),
        limitations: vec!["Integrity is not truth".into()],
        files: BTreeMap::new(),
    };
    PolicyPackage {
        manifest,
        data: Data {
            sources: vec![s],
            assertions: vec![],
            methods: vec![m],
            reviews: vec![review],
            artifacts: BTreeMap::new(),
        },
        evidence,
    }
}
#[test]
fn exact_native_export_preserves_method_but_downgrades_review() {
    let p = package();
    let preview = p.preview().unwrap();
    let bytes = p.export().unwrap();
    let imported = PolicyPackage::import(&bytes).unwrap();
    let source = imported.sources().unwrap();
    assert_eq!(
        source.values().next().unwrap().text,
        "Authorization is not expenditure."
    );
    assert_eq!(
        imported.data.methods[0].description,
        p.data.methods[0].description
    );
    assert_eq!(imported.data.reviews[0].origin, "imported_attestation");
    assert_eq!(
        applicability(
            &imported.data.methods[0],
            Some(&imported.data.reviews[0]),
            &source,
            "w",
            10,
            &BTreeMap::from([("scope_confirmed".into(), true)])
        )
        .state,
        "guarded"
    );
    assert!(preview.contains_key("capsule.json"));
    assert_eq!(bytes, p.export().unwrap());
}
#[test]
fn package_checks_rights_and_all_assertion_fields() {
    let mut p = package();
    p.data.sources[0].redistributable = false;
    assert!(p.export().is_err());
    let mut p = package();
    p.data.methods[0].workspace = "other".into();
    assert!(p.export().is_err());
    let mut p = package();
    p.data.methods[0].evidence[0].end += 1;
    assert!(p.export().is_err());
}
#[test]
fn v1_is_not_silently_v2_and_unknown_fields_fail() {
    let mut p = package();
    p.manifest.schema_version = 1;
    assert!(p.validate().is_err());
    let mut value = serde_json::to_value(package().data).unwrap();
    value["arbitrary_endpoint"] = serde_json::json!("https://untrusted.invalid");
    assert!(serde_json::from_value::<Data>(value).is_err());
}
