use std::collections::{BTreeMap, BTreeSet};
use tacitus_analysis::*;
use tacitus_capsules::compiler::EvidencePacket;
use tacitus_core::{digest, DecisionContract, Result, Span};

fn decision_inputs() -> (DecisionContract, EvidencePacket) {
    let contract = DecisionContract {
        workspace: "synthetic-test".into(),
        question: "Can the pilot proceed?".into(),
        decision_maker: "fictional council".into(),
        audience: "analyst".into(),
        mandate: "pilot authorization".into(),
        objectives: vec!["feasible pilot".into()],
        constraints: vec!["funding".into()],
        time_horizon: "one month".into(),
        known_as_of: 20,
        valid_at: 20,
        selected_sources: BTreeSet::from(["s@1".into()]),
        selected_capsules: BTreeSet::new(),
        permitted_tools: BTreeSet::new(),
        output_format: "brief".into(),
        context_tokens: 8192,
        output_tokens: 512,
        overhead_tokens: 1024,
    };
    let text = "SYNTHETIC. Operational launch requires appropriation.";
    let packet = EvidencePacket {
        compiler_version: "0.1.0".into(),
        contract_digest: digest(&serde_json::to_vec(&contract).unwrap()),
        sources: BTreeMap::from([("s@1".into(), text.into())]),
        citations: BTreeMap::from([(
            "E1".into(),
            Span {
                source: "s@1".into(),
                text_hash: digest(text.as_bytes()),
                start: 0,
                end: text.len(),
                locator: None,
                original_language: "en".into(),
                translation: None,
            },
        )]),
        omissions: vec![],
        source_families: BTreeMap::new(),
        evidence_byte_budget: 6656,
        used_bytes: text.len(),
        warnings: vec![],
    };
    (contract, packet)
}
/// Deterministic unit stub only; this is never a product runtime provider.
struct UnitStub {
    json: String,
}
impl ModelExecution for UnitStub {
    fn generate_json(&self, prompt: &str, budget: usize) -> Result<String> {
        assert!(prompt.contains("untrusted DATA"));
        assert_eq!(budget, 512);
        Ok(self.json.clone())
    }
}
fn unit_output(citation: &str) -> String {
    let j = serde_json::json!({"text":"Appropriation remains required.", "citations":[citation], "qualification":"Statement establishes requirement, not fulfillment."});
    serde_json::json!({"question":"Can the pilot proceed?", "judgments":[j.clone()],"alternatives":[j.clone()],"options":[j.clone()],"implementation_constraints":[j.clone()],"distributional_effects":[j],"uncertainties":["Affected-group impacts unknown."],"indicators":["Recorded appropriation."]}).to_string()
}
#[test]
fn decision_brief_is_cited_proposal_not_accepted_knowledge() {
    let (contract, packet) = decision_inputs();
    let proposal = decision_brief(
        &contract,
        &packet,
        &UnitStub {
            json: unit_output("E1"),
        },
    )
    .unwrap();
    assert!(proposal.state.contains("human review required"));
    assert!(proposal
        .validation
        .contains("semantic entailment unverified"));
}
#[test]
fn decision_brief_rejects_unknown_citation_and_malformed_output() {
    let (contract, packet) = decision_inputs();
    for json in [
        unit_output("other-workspace-citation"),
        "not JSON".into(),
        "{}".into(),
    ] {
        assert!(decision_brief(&contract, &packet, &UnitStub { json }).is_err());
    }
}
#[test]
fn packet_tampering_and_unselected_evidence_are_rejected() {
    let (contract, mut packet) = decision_inputs();
    packet
        .sources
        .insert("other@1".into(), "Unauthorized".into());
    assert!(decision_brief(
        &contract,
        &packet,
        &UnitStub {
            json: unit_output("E1")
        }
    )
    .is_err());
    packet.sources.remove("other@1");
    packet
        .sources
        .get_mut("s@1")
        .unwrap()
        .push_str(" Changed after compile.");
    assert!(decision_brief(
        &contract,
        &packet,
        &UnitStub {
            json: unit_output("E1")
        }
    )
    .is_err());
}
#[test]
fn missing_model_propagates_without_mock_or_cloud_fallback() {
    struct Unavailable;
    impl ModelExecution for Unavailable {
        fn generate_json(&self, _: &str, _: usize) -> Result<String> {
            Err("local model unavailable; initialize selected Ollama model".into())
        }
    }
    let (contract, packet) = decision_inputs();
    assert!(decision_brief(&contract, &packet, &Unavailable)
        .unwrap_err()
        .contains("local model unavailable"));
}
#[test]
fn serialized_prompt_budget_and_contract_binding_are_checked() {
    let (mut contract, mut packet) = decision_inputs();
    contract.question.push_str(" changed");
    assert!(decision_brief(
        &contract,
        &packet,
        &UnitStub {
            json: unit_output("E1")
        }
    )
    .is_err());
    packet.contract_digest = digest(&serde_json::to_vec(&contract).unwrap());
    contract.context_tokens = 1025;
    contract.output_tokens = 512;
    contract.overhead_tokens = 1;
    packet.contract_digest = digest(&serde_json::to_vec(&contract).unwrap());
    assert!(decision_brief(
        &contract,
        &packet,
        &UnitStub {
            json: unit_output("E1")
        }
    )
    .unwrap_err()
    .contains("prompt exceeds"));
}
#[test]
fn hypothesis_matrix_preserves_unknown_without_probability() {
    let hypotheses = vec![
        Hypothesis {
            id: "h1".into(),
            explanation: "conditional support".into(),
        },
        Hypothesis {
            id: "h2".into(),
            explanation: "delay tactic".into(),
        },
    ];
    let evidence = BTreeSet::from(["statement@1".into()]);
    let mut cells: Vec<_> = hypotheses
        .iter()
        .map(|h| MatrixCell {
            hypothesis: h.id.clone(),
            evidence: "statement@1".into(),
            consistency: "unknown".into(),
            diagnosticity_note: "Wording alone does not establish motive.".into(),
        })
        .collect();
    validate_hypothesis_matrix(&hypotheses, &cells, &evidence).unwrap();
    cells[0].consistency = "0.75 posterior".into();
    assert!(validate_hypothesis_matrix(&hypotheses, &cells, &evidence).is_err());
    cells.pop();
    assert!(validate_hypothesis_matrix(&hypotheses, &cells, &evidence).is_err());
}
#[test]
fn rhetoric_is_not_delivery() {
    let mut c = Commitment {
        actor: "Fictional Alder".into(),
        original_wording: "may discuss participation".into(),
        action: "participation".into(),
        modality: "may".into(),
        conditions: vec!["approval".into()],
        authority: None,
        deadline: None,
        implementation_evidence: vec![],
        unresolved_dependencies: vec!["appropriation".into()],
        source: "statement@1".into(),
    };
    c.validate(&BTreeSet::from(["statement@1".into()])).unwrap();
    assert!(!c.has_implementation_references());
    c.implementation_evidence.push("execution@1".into());
    assert!(c.has_implementation_references());
    assert!(c.validate(&BTreeSet::from(["statement@1".into()])).is_err());
}
#[test]
fn changed_source_traces_cycle_safe_transitive_impact() {
    let dependencies = BTreeMap::from([
        ("assertion".into(), BTreeSet::from(["source@1".into()])),
        (
            "hypothesis".into(),
            BTreeSet::from(["assertion".into(), "brief".into()]),
        ),
        ("brief".into(), BTreeSet::from(["hypothesis".into()])),
        ("unrelated".into(), BTreeSet::from(["source@2".into()])),
    ]);
    let impact = change_impact(BTreeSet::from(["source@1".into()]), &dependencies).unwrap();
    assert_eq!(
        impact.affected,
        BTreeSet::from([
            "source@1".into(),
            "assertion".into(),
            "hypothesis".into(),
            "brief".into()
        ])
    );
    assert_eq!(impact.unaffected, BTreeSet::from(["unrelated".into()]));
    assert!(impact.replacement_state.contains("new review"));
}
