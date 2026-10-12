use tacitus_analysis::harness::{self, Task};
use tacitus_core::{digest, policy::SourceRecord};
fn source() -> SourceRecord {
    let text="The Council authorized the programme.\nHowever, expenditure requires a separate appropriation.\n\nThe ministry discussed participation but did not accept it.".to_string();
    SourceRecord {
        id: "s".into(),
        version: 1,
        workspace: "w".into(),
        document_id: Some("d".into()),
        title: "Council record".into(),
        author: Some("Council".into()),
        original_hash: None,
        text_hash: digest(text.as_bytes()),
        text,
        language: "en".into(),
        extractor: "test".into(),
        published_at: Some(1),
        retrieved_at: 2,
        available_at: 2,
        valid_from: None,
        valid_until: None,
        source_family: "council-record".into(),
        withdrawn: false,
        restricted: false,
        redistributable: true,
        license: Some("CC0".into()),
    }
}
#[test]
fn qualification_stays_with_authorization() {
    let context = harness::compile(
        "What did the Council authorize?",
        Task::Answer,
        &[source()],
        &[],
        vec![],
        4000,
    )
    .unwrap();
    let passage = context
        .passages
        .iter()
        .find(|p| p.text.contains("authorized"))
        .unwrap();
    assert!(passage.text.contains("separate appropriation"));
    let s = source();
    assert_eq!(s.passage(&passage.span).unwrap(), passage.text);
}
#[test]
fn duplicate_common_origin_is_not_confirmation() {
    let a = source();
    let mut b = a.clone();
    b.id = "second".into();
    let c = harness::compile(
        "Council programme",
        Task::Answer,
        &[a, b],
        &[],
        vec![],
        8000,
    )
    .unwrap();
    assert!(c
        .omissions
        .iter()
        .any(|o| o.contains("not independent corroboration")));
    assert_eq!(
        c.passages
            .iter()
            .filter(|p| p.text.contains("authorized"))
            .count(),
        1
    );
}
#[test]
fn hallucinated_citation_and_missing_brief_sections_fail() {
    let c = harness::compile(
        "Council programme",
        Task::Brief,
        &[source()],
        &[],
        vec![],
        8000,
    )
    .unwrap();
    let raw = serde_json::json!({"question":c.question,"sections":[{"heading":"Judgments","text":"Funding available","citations":["E99"],"qualification":"uncertain"}],"claims":[],"hypotheses":[],"matrix":[],"positions":[],"uncertainties":["appropriation unknown"]});
    assert!(harness::validate(&raw.to_string(), &c).is_err());
    let mut valid_id = raw;
    valid_id["sections"][0]["citations"] = serde_json::json!([c.passages[0].id]);
    assert!(harness::validate(&valid_id.to_string(), &c)
        .unwrap_err()
        .contains("Missing brief section"));
}
#[test]
fn explicit_task_overrides_suggestion() {
    assert_eq!(
        harness::route("why did they withdraw?", Task::Commitments),
        Task::Commitments
    );
    assert_eq!(
        harness::route("what options should we consider?", Task::Auto),
        Task::Brief
    );
}
