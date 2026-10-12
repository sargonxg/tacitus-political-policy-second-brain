use std::collections::BTreeSet;
use tacitus_core::{
    ontology::*,
    policy::{PolicyAssertion, RELATIONS},
    EpistemicType,
};

fn assertion(predicate: &str, epistemic_type: EpistemicType) -> PolicyAssertion {
    PolicyAssertion {
        id: "a".into(),
        version: 1,
        workspace: "w".into(),
        speaker: "Minister Ada".into(),
        subject: "Council".into(),
        predicate: predicate.into(),
        object: Some("Programme".into()),
        proposition: "Expenditure remains conditional on appropriation".into(),
        epistemic_type,
        modality: "conditional".into(),
        conditions: vec!["separate appropriation".into()],
        exceptions: vec![],
        evidence: vec![],
        created_at: 10,
    }
}

#[test]
fn registry_is_bounded_and_uses_only_canonical_relations() {
    let packs = packs();
    let tools = tools();
    let definitions = relations();
    assert_eq!(packs.len(), 4);
    assert_eq!(
        packs.iter().map(|p| &p.id).collect::<BTreeSet<_>>().len(),
        packs.len()
    );
    assert_eq!(
        tools.iter().map(|t| &t.id).collect::<BTreeSet<_>>().len(),
        tools.len()
    );
    assert_eq!(
        definitions
            .iter()
            .map(|r| r.predicate.as_str())
            .collect::<BTreeSet<_>>(),
        RELATIONS.iter().copied().collect()
    );
    for pack in packs {
        assert_eq!(pack.version, 1);
        assert!(pack.entities.len() <= 20);
        assert_eq!(
            pack.entities
                .iter()
                .map(|e| &e.extraction_type)
                .collect::<BTreeSet<_>>()
                .len(),
            pack.entities.len()
        );
        assert!(pack
            .entities
            .iter()
            .all(|e| !e.id.is_empty() && !e.definition.is_empty()));
        assert!(pack
            .predicates
            .iter()
            .all(|p| RELATIONS.contains(&p.as_str())));
        assert!(pack.tools.iter().all(|t| find_tool(t).is_some()));
        assert!(!pack.safeguards.is_empty());
    }
    for tool in tools {
        assert!(tool
            .predicates
            .iter()
            .all(|p| RELATIONS.contains(&p.as_str())));
        assert!(!tool.safeguard.is_empty());
    }
    assert!(find_pack("imported-plugin").is_none());
    assert!(find_tool("https://arbitrary-endpoint.example").is_none());
}

#[test]
fn rights_view_preserves_allegation_and_excludes_explanatory_inference() {
    let rights = find_tool("rights").unwrap();
    let alleged = assertion("states", EpistemicType::Allegation);
    assert!(rights.matches(&alleged));
    assert_eq!(alleged.epistemic_type, EpistemicType::Allegation);
    assert!(!rights.matches(&assertion("states", EpistemicType::Inference)));
    assert!(find_tool("hypotheses")
        .unwrap()
        .matches(&assertion("states", EpistemicType::Inference)));
    assert!(!find_tool("hypotheses").unwrap().matches(&alleged));
}

#[test]
fn funding_and_position_views_do_not_promote_conditional_undertakings() {
    let authorization = assertion("authorizes", EpistemicType::ReportedStatement);
    assert!(find_tool("funding").unwrap().matches(&authorization));
    assert!(!find_tool("positions").unwrap().matches(&authorization));
    let undertaking = assertion("conditionally_undertakes", EpistemicType::ReportedStatement);
    assert!(find_tool("positions").unwrap().matches(&undertaking));
    assert_eq!(undertaking.conditions, vec!["separate appropriation"]);
    assert_eq!(undertaking.modality, "conditional");
    assert!(!find_tool("funding").unwrap().matches(&undertaking));
}

#[test]
fn actor_search_matches_subject_or_speaker_without_identity_resolution() {
    let a = assertion("states", EpistemicType::Observation);
    assert!(actor_matches(&a, Some(" ada ")));
    assert!(actor_matches(&a, Some("COUNCIL")));
    assert!(!actor_matches(&a, Some("Programme")));
    assert!(actor_matches(&a, None));
    assert_eq!(a.subject, "Council");
}
