use serde_json::{json, Value};
use tacitus_analysis::knowledge_capture::{self as k, Capture};
use tacitus_core::digest;
const RAW: &[u8] = include_bytes!("fixtures/minister/capture.json");
fn query(question: &str, budget: usize) -> Vec<u8> {
    let mut q: Value =
        serde_json::from_slice(include_bytes!("fixtures/minister/speech-fr.query.json")).unwrap();
    q["question"] = json!(question);
    q["max_bytes"] = json!(budget);
    serde_json::to_vec(&q).unwrap()
}
#[test]
fn original_french_span_recovers_claim_and_required_closure() {
    let z = k::pack(RAW).unwrap();
    let out: Value =
        serde_json::from_slice(&k::reuse(&z, &query("applicable", 16384)).unwrap()).unwrap();
    assert_eq!(out["context"]["claims"].as_object().unwrap().len(), 5);
    assert!(out["context"]["claims"].get("application-fr").is_some());
    assert!(out["context"]["claims"].get("phased-exceptions").is_some());
    let small: Value =
        serde_json::from_slice(&k::reuse(&z, &query("applicable", 8192)).unwrap()).unwrap();
    assert!(small["context"]["claims"].as_object().unwrap().is_empty());
}
#[test]
fn unquoted_document_text_and_assistant_translation_do_not_create_seeds() {
    let mut c: Capture = serde_json::from_slice(RAW).unwrap();
    let sid = c
        .assertions
        .iter()
        .find(|a| a.id == "application-fr")
        .unwrap()
        .evidence[0]
        .source
        .clone();
    let source = c.documents.iter_mut().find(|s| s.key() == sid).unwrap();
    source.text.push_str("\nUnquotedquasar\n");
    source.text_hash = digest(source.text.as_bytes());
    source.original_hash = Some(source.text_hash.clone());
    let hash = source.text_hash.clone();
    for a in &mut c.assertions {
        for e in &mut a.evidence {
            if e.source == sid {
                e.text_hash = hash.clone();
                e.translation = Some("Translationonlyultraviolet".into());
            }
        }
    }
    c.graph.claim_fingerprints = c
        .assertions
        .iter()
        .map(|a| (a.id.clone(), a.fingerprint()))
        .collect();
    let z = k::pack(&serde_json::to_vec(&c).unwrap()).unwrap();
    for term in ["Unquotedquasar", "Translationonlyultraviolet"] {
        let out: Value =
            serde_json::from_slice(&k::reuse(&z, &query(term, 16384)).unwrap()).unwrap();
        assert!(out["context"]["claims"].as_object().unwrap().is_empty());
    }
}
#[test]
fn source_channel_does_not_bypass_revocation_or_strict_time() {
    let z = k::pack(RAW).unwrap();
    let mut q: Value = serde_json::from_slice(&query("applicable", 16384)).unwrap();
    q["time_mode"] = json!("require_declared_interval");
    assert!(k::reuse(&z, &serde_json::to_vec(&q).unwrap()).is_err());
    let c: Capture = serde_json::from_slice(RAW).unwrap();
    q["time_mode"] = json!("disclose_unknown");
    q["revoked_sources"] = json!([c.documents[0].key()]);
    assert!(k::reuse(&z, &serde_json::to_vec(&q).unwrap()).is_err());
}
#[test]
fn overlapping_original_span_scan_has_explicit_work_bound() {
    let mut c: Capture =
        serde_json::from_slice(include_bytes!("fixtures/knowledge/capture.json")).unwrap();
    c.documents[0].text = "x".repeat(500000);
    c.documents[0].text_hash = digest(c.documents[0].text.as_bytes());
    c.documents[0].original_hash = Some(c.documents[0].text_hash.clone());
    let key = c.documents[0].key();
    let hash = c.documents[0].text_hash.clone();
    for a in &mut c.assertions {
        for s in &mut a.evidence {
            if s.source == key {
                s.text_hash = hash.clone();
            }
        }
    }
    for i in 0..20 {
        let mut a = c.assertions[0].clone();
        a.id = format!("scan{i}");
        a.evidence[0].start = i;
        a.evidence[0].end = 500000;
        c.observed_at.insert(a.id.clone(), None);
        c.assertions.push(a);
    }
    c.graph.claim_fingerprints = c
        .assertions
        .iter()
        .map(|a| (a.id.clone(), a.fingerprint()))
        .collect();
    let z = k::pack(&serde_json::to_vec(&c).unwrap()).unwrap();
    let err = k::reuse(&z, include_bytes!("fixtures/knowledge/port.query.json")).unwrap_err();
    assert!(err.contains("scan exceeds 8 MiB"));
}
