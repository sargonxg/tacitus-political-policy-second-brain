use serde_json::{json, Value};
use tacitus_analysis::knowledge_capture::{pack, reuse};
const CAP: &[u8] = include_bytes!("fixtures/minister/capture.json");
const SPEECH: &[u8] = include_bytes!("fixtures/minister/speech-fr.query.json");
#[test]
fn multilingual_speech_preserves_original_and_unreviewed_translation() {
    let z = pack(CAP).unwrap();
    let v: Value = serde_json::from_slice(&reuse(&z, SPEECH).unwrap()).unwrap();
    let c = v["context"]["claims"].as_object().unwrap();
    assert_eq!(
        c.keys().map(String::as_str).collect::<Vec<_>>(),
        vec![
            "application-en",
            "application-fr",
            "phased-exceptions",
            "speech-guidance",
            "version-warning"
        ]
    );
    let fr = &c["application-fr"];
    assert_eq!(fr["citations"][0]["span"]["original_language"], "fr");
    assert_eq!(
        fr["citations"][0]["text"],
        "Il est applicable à\u{a0}partir du 2\u{a0}août 2026."
    );
    assert_eq!(
        fr["citations"][0]["span"]["translation"],
        "It applies from 2 August 2026."
    );
    assert!(fr["assertion"]["conditions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|x| x
            .as_str()
            .unwrap()
            .contains("Translation status: unreviewed")));
    assert!(c["application-en"]["citations"][0]["span"]["translation"].is_null());
    assert_eq!(
        c["speech-guidance"]["assertion"]["epistemic_type"],
        "recommendation"
    );
    assert!(v["context"]["entities"]["application"]["event_time"].is_null());
}
#[test]
fn institutional_publication_and_calendar_date_do_not_invent_timestamp() {
    let z = pack(CAP).unwrap();
    let v: Value = serde_json::from_slice(
        &reuse(
            &z,
            include_bytes!("fixtures/minister/publication.query.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let claims = v["context"]["claims"].as_object().unwrap();
    assert_eq!(
        claims.keys().map(String::as_str).collect::<Vec<_>>(),
        vec!["publication-record"]
    );
    assert!(v["context"]["entities"]["publication"]["event_time"].is_null());
    assert!(v["source_metadata"]["research-note@1"]["published_at"].is_null());
    assert!(claims["publication-record"]["assertion"]["conditions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|x| x.as_str().unwrap().contains("2024-07-12")));
    assert_eq!(
        v["context"]["entities"]["parliament"]["kind"],
        "institution"
    );
}
#[test]
fn no_cross_language_semantic_match_or_current_deadline_claim() {
    let z = pack(CAP).unwrap();
    let v: Value = serde_json::from_slice(
        &reuse(
            &z,
            include_bytes!("fixtures/minister/no-overlap-fr.query.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(v["context"]["claims"].as_object().unwrap().is_empty());
    assert!(v["status"].as_str().unwrap().contains("omission_only"));
    let mut q: Value = serde_json::from_slice(SPEECH).unwrap();
    q["time_mode"] = json!("require_declared_interval");
    assert!(reuse(&z, &serde_json::to_vec(&q).unwrap()).is_err());
}
