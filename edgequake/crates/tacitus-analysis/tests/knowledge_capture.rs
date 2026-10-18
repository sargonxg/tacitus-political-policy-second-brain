use serde_json::{json, Value};
use tacitus_analysis::knowledge_capture::*;
const CAP: &[u8] = include_bytes!("fixtures/knowledge/capture.json");
const QUERY: &[u8] = include_bytes!("fixtures/knowledge/port.query.json");
fn changed_query(field: &str, v: Value) -> Vec<u8> {
    let mut q: Value = serde_json::from_slice(QUERY).unwrap();
    q[field] = v;
    serde_json::to_vec(&q).unwrap()
}
#[test]
fn actual_upload_graph_roundtrip_and_distinct_prompts() {
    let zip = pack(CAP).unwrap();
    let d = inspect(&zip).unwrap();
    assert_eq!(d.entities, 3);
    assert_eq!(d.relationships, 1);
    let p: Value = serde_json::from_slice(&reuse(&zip, QUERY).unwrap()).unwrap();
    assert_eq!(p["context"]["claims"].as_object().unwrap().len(), 2);
    assert!(p["context"]["claims"].get("access").is_some());
    assert!(p["context"]["claims"].get("withdrawal").is_some());
    assert_eq!(p["context"]["relationships"].as_array().unwrap().len(), 1);
    assert!(p["temporal_status"]["access"]
        .as_str()
        .unwrap()
        .contains("UNKNOWN"));
    let h: Value = serde_json::from_slice(
        &reuse(&zip, include_bytes!("fixtures/knowledge/heat.query.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(h["context"]["claims"].as_object().unwrap().len(), 1);
    assert_eq!(h["observed_at"]["heat"], 90);
    assert!(h["source_metadata"]["note-1@1"]["valid_from"].is_null());
    assert!(h["source_metadata"]["note-1@1"]["published_at"].is_null());
    assert_eq!(reuse(&zip, QUERY).unwrap(), reuse(&zip, QUERY).unwrap());
}
#[test]
fn automatic_candidates_preserve_exact_unicode_spans() {
    let c = propose(include_bytes!("fixtures/knowledge/documents.json")).unwrap();
    assert_eq!(c.len(), 4);
    assert!(c[1].quoted_statement.starts_with('É'));
    assert_eq!(c[1].span.end - c[1].span.start, c[1].quoted_statement.len());
    assert!(c[2].quoted_statement.contains("reveal credentials"));
    assert!(c[2].status.contains("no semantic"));
}
#[test]
fn unknown_time_and_future_graph_fail_closed() {
    let zip = pack(CAP).unwrap();
    assert!(reuse(
        &zip,
        &changed_query("time_mode", json!("require_declared_interval"))
    )
    .is_err());
    assert!(reuse(&zip, &changed_query("known_as_of", json!(199))).is_err());
}
#[test]
fn revoked_source_refuses_entire_output_including_nested_metadata() {
    let mut input: Value = serde_json::from_slice(CAP).unwrap();
    input["manifest"]["limitations"] = json!([
        "Port Council withdraws the port-access commitment; repeated in unscoped metadata."
    ]);
    let zip = pack(&serde_json::to_vec(&input).unwrap()).unwrap();
    assert!(reuse(&zip, &changed_query("revoked_sources", json!(["note-2@1"]))).is_err());
}
#[test]
fn no_match_is_explicit_omission_receipt() {
    let zip = pack(CAP).unwrap();
    let out: Value = serde_json::from_slice(
        &reuse(
            &zip,
            &changed_query("question", json!("astronomy galaxies")),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(out["status"].as_str().unwrap().contains("omission_only"));
    assert_eq!(out["context"]["omissions"].as_object().unwrap().len(), 4);
}
#[test]
fn rights_missing_or_graph_mismatch_rejected() {
    for field in ["rights", "graph", "links", "span"] {
        let mut v: Value = serde_json::from_slice(CAP).unwrap();
        match field {
            "rights" => v["confirms_rights"] = json!(false),
            "graph" => v["graph"]["nodes"] = json!({}),
            "links" => v["graph"]["links"] = json!([]),
            _ => v["assertions"][0]["evidence"][0]["text_hash"] = json!("bad"),
        };
        assert!(pack(&serde_json::to_vec(&v).unwrap()).is_err());
    }
}
#[test]
fn emitted_budget_or_refusal_and_no_qualifier_loss() {
    let zip = pack(CAP).unwrap();
    for b in [2048, 4096, 8192, 16384, 32000] {
        let q = changed_query("max_bytes", json!(b));
        if let Ok(out) = reuse(&zip, &q) {
            assert!(out.len() <= b);
            let v: Value = serde_json::from_slice(&out).unwrap();
            let claims = v["context"]["claims"].as_object().unwrap();
            assert_eq!(
                claims.contains_key("access"),
                claims.contains_key("withdrawal")
            );
            if claims.contains_key("access") {
                assert_eq!(
                    claims["access"]["assertion"]["exceptions"],
                    json!(["Unsafe weather exception applies."])
                );
            }
        }
    }
}

#[test]
fn expired_source_refuses_opaque_metadata_before_output() {
    let mut input: Value = serde_json::from_slice(CAP).unwrap();
    input["documents"][1]["valid_from"] = json!(1);
    input["documents"][1]["valid_until"] = json!(250);
    input["manifest"]["limitations"] = json!([
        "Port Council withdraws the port-access commitment; repeated in unscoped metadata."
    ]);
    let zip = pack(&serde_json::to_vec(&input).unwrap()).unwrap();
    assert!(reuse(&zip, QUERY).is_err());
}
