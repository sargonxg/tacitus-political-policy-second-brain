use std::collections::{BTreeMap, BTreeSet};
use tacitus_analysis::layers::*;
use tacitus_core::{
    digest,
    policy::{PolicyAssertion, SourceRecord},
    EpistemicType, Span,
};
fn fixture() -> (
    Request,
    BTreeMap<String, SourceRecord>,
    BTreeMap<String, PolicyAssertion>,
    Graph,
) {
    let mut sources = BTreeMap::new();
    let mut claims: BTreeMap<String, PolicyAssertion> = BTreeMap::new();
    for (id, text, time) in [
        ("a", "The pilot may proceed subject to cabinet approval.", 1),
        (
            "b",
            "The cabinet declined approval; the proposal is withdrawn.",
            2,
        ),
    ] {
        let hash = digest(text.as_bytes());
        let s = SourceRecord {
            id: id.into(),
            version: 1,
            workspace: "demo".into(),
            document_id: None,
            title: "Fictional statement".into(),
            author: Some("Synthetic delegation".into()),
            original_hash: Some(hash.clone()),
            text_hash: hash.clone(),
            text: text.into(),
            language: "en".into(),
            extractor: "synthetic-test".into(),
            published_at: Some(time),
            retrieved_at: time,
            available_at: time,
            valid_from: Some(time),
            valid_until: None,
            source_family: "synthetic".into(),
            withdrawn: false,
            restricted: false,
            redistributable: true,
            license: Some("CC0 synthetic fixture".into()),
        };
        let a = PolicyAssertion {
            id: id.into(),
            version: 1,
            workspace: "demo".into(),
            speaker: "Synthetic delegation".into(),
            subject: "Delegation".into(),
            predicate: "conditionally_undertakes".into(),
            object: Some("Pilot".into()),
            proposition: text.into(),
            epistemic_type: EpistemicType::ReportedStatement,
            modality: "reported".into(),
            conditions: vec!["Cabinet approval required".into()],
            exceptions: vec![],
            evidence: vec![Span {
                source: s.key(),
                text_hash: hash,
                start: 0,
                end: text.len(),
                locator: None,
                original_language: "en".into(),
                translation: None,
            }],
            created_at: time,
        };
        sources.insert(s.key(), s);
        claims.insert(id.into(), a);
    }
    let request = Request {
        question: "Can the pilot proceed?".into(),
        workspace: "demo".into(),
        known_as_of: 3,
        valid_at: 3,
        selected_sources: sources.keys().cloned().collect(),
        max_bytes: 20000,
    };
    let graph = Graph {
        available_at: 1,
        claim_fingerprints: claims
            .iter()
            .map(|(id, a)| (id.clone(), a.fingerprint()))
            .collect(),
        version: "tacitus-policy-graph-sidecar/1".into(),
        nodes: BTreeMap::from([(
            "delegation".into(),
            Node {
                id: "delegation".into(),
                kind: NodeKind::Actor,
                label: "Synthetic delegation".into(),
                claims: BTreeSet::from(["a".into(), "b".into()]),
                event_time: None,
            },
        )]),
        links: vec![Link {
            from: "b".into(),
            to: "a".into(),
            kind: LinkKind::Supersedes,
            rationale: "Explicit fictional correction".into(),
        }],
        knowledge: BTreeMap::new(),
    };
    (request, sources, claims, graph)
}
fn main() {
    let (request, sources, assertions, graph) = fixture();
    let dossier = serde_json::json!({"request":request,"sources":sources,"assertions":assertions,"graph":graph});
    println!("{}", serde_json::to_string_pretty(&dossier).unwrap());
}
