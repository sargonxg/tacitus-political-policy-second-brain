use std::collections::BTreeMap;
use tacitus_core::{digest, policy::*, Span};
fn fixture() -> (CognitiveMethod, HumanReview, BTreeMap<String, SourceRecord>) {
    let s = SourceRecord {
        id: "s".into(),
        version: 1,
        workspace: "w".into(),
        document_id: None,
        title: "Test".into(),
        author: None,
        original_hash: None,
        text: "Only after approval".into(),
        text_hash: digest(b"Only after approval"),
        language: "en".into(),
        extractor: "test".into(),
        published_at: None,
        retrieved_at: 1,
        available_at: 1,
        valid_from: None,
        valid_until: None,
        source_family: "origin".into(),
        withdrawn: false,
        restricted: false,
        redistributable: true,
        license: Some("CC0".into()),
    };
    let span = Span {
        source: s.key(),
        text_hash: s.text_hash.clone(),
        start: 0,
        end: s.text.len(),
        locator: None,
        original_language: "en".into(),
        translation: None,
    };
    let m = CognitiveMethod {
        id: "m".into(),
        version: 1,
        workspace: "w".into(),
        kind: "distinction".into(),
        description: "Distinguish authorization from expenditure".into(),
        scope: "Budget decisions".into(),
        prerequisites: vec!["scope_confirmed".into()],
        failure_conditions: vec!["Appropriation record missing".into()],
        evidence: vec![span.clone()],
        created_at: 2,
        available_at: 2,
        ontology_version: ONTOLOGY_VERSION.into(),
    };
    let r = HumanReview {
        id: "r".into(),
        target_id: m.id.clone(),
        target_kind: "method".into(),
        target_fingerprint: m.fingerprint(),
        evidence_hashes: BTreeMap::from([(span.source, span.text_hash)]),
        reviewer: "human".into(),
        decision: "accept".into(),
        rationale: "Applicable to this budget question".into(),
        reviewed_at: 3,
        origin: "authenticated_human".into(),
    };
    (m, r, BTreeMap::from([(s.key(), s)]))
}
#[test]
fn applicability_is_review_scope_and_version_bound() {
    let (m, mut r, s) = fixture();
    assert_eq!(
        applicability(&m, Some(&r), &s, "w", 4, &BTreeMap::new()).state,
        "guarded"
    );
    assert_eq!(
        applicability(
            &m,
            Some(&r),
            &s,
            "w",
            4,
            &BTreeMap::from([("scope_confirmed".into(), false)])
        )
        .state,
        "blocked"
    );
    let facts = BTreeMap::from([("scope_confirmed".into(), true)]);
    assert_eq!(
        applicability(&m, Some(&r), &s, "w", 4, &facts).state,
        "active"
    );
    r.decision = "reject".into();
    assert_eq!(
        applicability(&m, Some(&r), &s, "w", 4, &facts).state,
        "guarded"
    );
    r.decision = "accept".into();
    r.origin = "imported_attestation".into();
    assert_eq!(
        applicability(&m, Some(&r), &s, "w", 4, &facts).state,
        "guarded"
    );
    assert_eq!(
        applicability(&m, Some(&r), &BTreeMap::new(), "w", 4, &facts).state,
        "stale"
    );
}
#[test]
fn source_hash_language_and_utf8_are_exact() {
    let (_, _, s) = fixture();
    let source = s.values().next().unwrap();
    let span = Span {
        source: source.key(),
        text_hash: "wrong".into(),
        start: 0,
        end: 4,
        locator: None,
        original_language: "en".into(),
        translation: None,
    };
    assert!(source.passage(&span).is_err());
    let mut mutated = source.clone();
    mutated.text.push('!');
    assert!(mutated.validate().is_err());
}
