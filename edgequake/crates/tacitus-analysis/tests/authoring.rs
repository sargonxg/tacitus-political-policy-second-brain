use tacitus_analysis::v2_workflow::{render_text, LocalCatalog, Request, TimeMode};
use tacitus_capsules::{authoring::pack_original_summaries, policy::PolicyPackage};
const HEAT: &str = include_str!("fixtures/v2/nyc-heat-resilience.native-v2-input.json");
const COMPUTE: &str =
    include_str!("fixtures/v2/compute-infrastructure-governance.native-v2-input.json");
const ACCESS: &str = include_str!("fixtures/v2/humanitarian-access-analysis.native-v2-input.json");
#[test]
fn existing_three_author_validate_inspect_review() {
    for raw in [HEAT, COMPUTE, ACCESS] {
        let bytes = pack_original_summaries(raw.as_bytes(), true).unwrap();
        assert_eq!(
            bytes,
            pack_original_summaries(raw.as_bytes(), true).unwrap()
        );
        let p = PolicyPackage::import_strict(&bytes).unwrap();
        assert!(p
            .data
            .sources
            .iter()
            .all(|s| s.valid_from.is_none() && s.valid_until.is_none()));
        let mut c = LocalCatalog::default();
        let d = c.import_native_v2(&bytes).unwrap();
        assert_eq!(c.inspect(&d.archive_digest).unwrap().assertions, 5);
        let mut r = Request {
            question: "What do these authored records say?".into(),
            known_as_of: p.manifest.created_at,
            valid_at: 0,
            time_mode: TimeMode::DiscloseUnknown,
            max_bytes: 200000,
        };
        let e = c.export_review(&d.archive_digest, &r).unwrap();
        assert_eq!(e.document.layered_context.packet.claims.len(), 5);
        assert!(render_text(&e, 500000)
            .unwrap()
            .contains("UNKNOWN WORLD VALIDITY"));
        r.time_mode = TimeMode::RequireDeclaredInterval;
        assert!(c.export_review(&d.archive_digest, &r).is_err());
    }
}
#[test]
fn refuses_unconfirmed_duplicate_and_misleading_declarations() {
    assert!(pack_original_summaries(HEAT.as_bytes(), false).is_err());
    assert!(pack_original_summaries(
        HEAT.replacen("\"manifest\":", "\"manifest\":{},\"manifest\":", 1)
            .as_bytes(),
        true
    )
    .is_err());
    for (path, value) in [
        ("/data/sources/0/valid_from", serde_json::json!(0)),
        ("/data/sources/0/redistributable", serde_json::json!(false)),
        ("/data/sources/0/author", serde_json::json!("Publisher")),
        (
            "/data/sources/0/extractor",
            serde_json::json!("publisher extracted text"),
        ),
        (
            "/data/sources/0/available_at",
            serde_json::json!(9999999999i64),
        ),
        (
            "/data/sources/0/text_hash",
            serde_json::json!("0".repeat(64)),
        ),
        ("/data/assertions/0/conditions", serde_json::json!([])),
    ] {
        let mut v: serde_json::Value = serde_json::from_str(HEAT).unwrap();
        *v.pointer_mut(path).unwrap() = value;
        assert!(
            pack_original_summaries(&serde_json::to_vec(&v).unwrap(), true).is_err(),
            "{path}"
        );
    }
}

#[test]
fn refuses_native_v1_reviews_and_unknown_fields() {
    for (path, value) in [
        ("/manifest/schema_version", serde_json::json!(1)),
        (
            "/manifest/files",
            serde_json::json!({"capsule.json":"0".repeat(64)}),
        ),
        ("/data/reviews", serde_json::json!([{}])),
        ("/data/sources/0/license", serde_json::json!(null)),
    ] {
        let mut v: serde_json::Value = serde_json::from_str(HEAT).unwrap();
        *v.pointer_mut(path).unwrap() = value;
        assert!(pack_original_summaries(&serde_json::to_vec(&v).unwrap(), true).is_err());
    }
    let mut v: serde_json::Value = serde_json::from_str(HEAT).unwrap();
    v["unrecognized"] = serde_json::json!(true);
    assert!(pack_original_summaries(&serde_json::to_vec(&v).unwrap(), true).is_err());
}

#[test]
fn unknown_publication_remains_null() {
    let raw = include_str!("../../../authoring-demo/synthetic-input.json");
    let bytes = pack_original_summaries(raw.as_bytes(), true).unwrap();
    let p = PolicyPackage::import_strict(&bytes).unwrap();
    assert!(p.data.sources[0].published_at.is_none());
    let mut c = LocalCatalog::default();
    let d = c.import_native_v2(&bytes).unwrap();
    let r = Request {
        question: "What is proposed?".into(),
        known_as_of: p.manifest.created_at,
        valid_at: 0,
        time_mode: TimeMode::DiscloseUnknown,
        max_bytes: 32768,
    };
    let e = c.export_review(&d.archive_digest, &r).unwrap();
    let value = serde_json::to_value(&e.document).unwrap();
    assert!(value["source_metadata"]
        .as_object()
        .unwrap()
        .values()
        .all(|s| s["published_at"].is_null()));
}
