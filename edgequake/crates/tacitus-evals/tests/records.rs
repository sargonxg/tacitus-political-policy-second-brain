use tacitus_evals::*;
fn run() -> Run {
    Run {
        arm: Arm::EdgeQuake,
        model_digest: "test-model".into(),
        dataset_digest: "test-dataset".into(),
        prompt_version: "1".into(),
        compiler_version: "1".into(),
        known_as_of: 20,
        evidence_access_digest: "same-evidence".into(),
        prior_notes_digest: "same-notes".into(),
        input_tokens: None,
        output_tokens: None,
        output_budget: 512,
        latency_ms: Some(10),
        ingestion_ms: None,
        retrieval_ms: None,
        cache_state: "warm".into(),
        automatic_citation_checks: Some(true),
        human_critical_errors: None,
        human_coverage: None,
        human_qualifications: None,
        human_material_omissions: None,
        human_temporal_errors: None,
        analyst_corrections: None,
        time_to_acceptable_brief_ms: None,
        status: "measured".into(),
    }
}
#[test]
fn null_human_ratings_stay_pending() {
    let restored: Run = serde_json::from_str(&serde_json::to_string(&run()).unwrap()).unwrap();
    assert!(restored.human_critical_errors.is_none());
    assert!(restored.human_coverage.is_none());
}
#[test]
fn unequal_access_and_not_run_cannot_be_compared() {
    let a = run();
    let mut b = run();
    b.arm = Arm::TacitusContext;
    comparable(&a, &b).unwrap();
    b.prior_notes_digest = "withheld-correction".into();
    assert!(comparable(&a, &b).is_err());
    b = run();
    b.known_as_of += 1;
    assert!(comparable(&a, &b).is_err());
    b = run();
    b.status = "NOT RUN".into();
    assert!(comparable(&a, &b).is_err());
    b = run();
    b.arm = Arm::ExternalNotRun;
    assert!(comparable(&a, &b).is_err());
}
