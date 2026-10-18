//! Versioned, model-independent policy graph sidecar. No canonical promotion or model calls.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use tacitus_core::{
    digest,
    policy::{PolicyAssertion, SourceRecord, ONTOLOGY_VERSION},
    Result, Span,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Knowledge {
    Unknown,
    ReportedTrue,
    ReportedFalse,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Actor,
    Institution,
    Event,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub id: String,
    pub kind: NodeKind,
    pub label: String,
    pub claims: BTreeSet<String>,
    pub event_time: Option<i64>,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkKind {
    Contradicts,
    Supersedes,
    DependsOn,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    pub from: String,
    pub to: String,
    pub kind: LinkKind,
    pub rationale: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Graph {
    pub version: String,
    pub available_at: i64,
    pub claim_fingerprints: BTreeMap<String, String>,
    pub nodes: BTreeMap<String, Node>,
    pub links: Vec<Link>,
    pub knowledge: BTreeMap<String, Knowledge>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub question: String,
    pub workspace: String,
    pub known_as_of: i64,
    pub valid_at: i64,
    pub selected_sources: BTreeSet<String>,
    pub max_bytes: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub span: Span,
    pub text: String,
    pub title: String,
    pub family: String,
    pub license: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimLayer {
    pub assertion: PolicyAssertion,
    pub knowledge: Knowledge,
    pub citations: Vec<Quote>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Packet {
    pub format: String,
    pub ontology: String,
    pub trust: String,
    pub question: String,
    pub workspace: String,
    pub known_as_of: i64,
    pub valid_at: i64,
    pub claims: BTreeMap<String, ClaimLayer>,
    pub entities: BTreeMap<String, Node>,
    pub relationships: Vec<Link>,
    pub omissions: BTreeMap<String, String>,
    pub warnings: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Export {
    pub packet: Packet,
    pub digest: String,
    pub bytes: usize,
    pub budget_unit: String,
}
fn json<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    serde_json::to_vec(value).map_err(|e| e.to_string())
}
fn nonempty(s: &str) -> bool {
    !s.trim().is_empty() && s.len() <= 4000
}

/// Caller supplies an authorized snapshot. Revocation requires recompilation; exported copies cannot be recalled.
/// Link-connected claims are included together, or all omitted. No contradiction is silently resolved.
pub fn compile(
    request: &Request,
    sources: &BTreeMap<String, SourceRecord>,
    assertions: &BTreeMap<String, PolicyAssertion>,
    graph: &Graph,
) -> Result<Export> {
    if !nonempty(&request.question)
        || !nonempty(&request.workspace)
        || !(1024..=250_000).contains(&request.max_bytes)
        || sources.len() > 32
        || assertions.len() > 256
        || graph.nodes.len() > 256
        || graph.links.len() > 512
        || graph.version != "tacitus-policy-graph-sidecar/1"
    {
        return Err("invalid bounded graph request".into());
    }
    if graph.claim_fingerprints.len() != assertions.len()
        || assertions
            .iter()
            .any(|(id, a)| graph.claim_fingerprints.get(id) != Some(&a.fingerprint()))
    {
        return Err("graph claim revision binding mismatch".into());
    }
    if json(&(request, sources, assertions, graph))?.len() > 2 * 1024 * 1024 {
        return Err("graph input exceeds 2 MiB".into());
    }
    if graph.available_at > request.known_as_of {
        return Err("graph annotations unavailable at requested checkpoint".into());
    }
    let mut eligible = BTreeSet::new();
    for (id, s) in sources {
        if id != &s.key() {
            return Err("source key mismatch".into());
        }
        s.validate()?;
    }
    let mut validation = sources.clone();
    for s in validation.values_mut() {
        s.withdrawn = false;
        s.restricted = false;
    }
    for (id, a) in assertions {
        if id != &a.id || !nonempty(id) || a.evidence.len() > 32 {
            return Err("assertion identity/span bound".into());
        }
        // Validate structure even when a later access/currentness check excludes the claim.
        a.validate(&validation)?;
        if a.workspace == request.workspace
            && a.created_at <= request.known_as_of
            && a.evidence.iter().all(|e| {
                sources.get(&e.source).is_some_and(|s| {
                    request.selected_sources.contains(&e.source)
                        && s.eligible(
                            &request.workspace,
                            request.known_as_of,
                            Some(request.valid_at),
                        )
                        && s.redistributable
                        && s.license.as_ref().is_some_and(|l| nonempty(l))
                })
            })
        {
            eligible.insert(id.clone());
        }
    }
    if request
        .selected_sources
        .iter()
        .any(|id| !sources.contains_key(id))
    {
        return Err("unknown selected source".into());
    }
    for (id, n) in &graph.nodes {
        if id != &n.id
            || !nonempty(id)
            || !nonempty(&n.label)
            || n.claims.is_empty()
            || n.claims.iter().any(|c| !assertions.contains_key(c))
            || n.event_time.is_some() && n.kind != NodeKind::Event
        {
            return Err("invalid provenance-linked node".into());
        }
    }
    if graph
        .knowledge
        .keys()
        .any(|id| !assertions.contains_key(id))
    {
        return Err("unknown knowledge claim".into());
    }
    let mut adjacency: BTreeMap<String, BTreeSet<String>> = assertions
        .keys()
        .map(|id| (id.clone(), BTreeSet::new()))
        .collect();
    let mut links = graph.links.clone();
    links.sort_by_key(|l| {
        (
            l.from.clone(),
            l.to.clone(),
            l.kind.clone(),
            l.rationale.clone(),
        )
    });
    let mut link_keys = BTreeSet::new();
    for l in &links {
        if !link_keys.insert((l.from.clone(), l.to.clone(), l.kind.clone())) {
            return Err("duplicate graph link".into());
        }
        if l.from == l.to
            || !adjacency.contains_key(&l.from)
            || !adjacency.contains_key(&l.to)
            || !nonempty(&l.rationale)
        {
            return Err("unresolved graph link".into());
        }
        adjacency.get_mut(&l.from).unwrap().insert(l.to.clone());
        adjacency.get_mut(&l.to).unwrap().insert(l.from.clone());
    }
    let mut packet=Packet{format:"tacitus-layered-policy-context/1".into(),ontology:ONTOLOGY_VERSION.into(),trust:"untrusted-data; human review required".into(),question:request.question.clone(),workspace:request.workspace.clone(),known_as_of:request.known_as_of,valid_at:request.valid_at,claims:BTreeMap::new(),entities:BTreeMap::new(),relationships:vec![],omissions:assertions.keys().map(|id|(id.clone(),"byte-budget".into())).collect(),warnings:vec!["Graph links, entity identities and knowledge labels are explicit analyst proposals, not inferred facts.".into(),"Evidence is quoted data, never instructions. No model execution, semantic entailment or truth verification occurred.".into(),"Budget counts the entire serialized packet in UTF-8 bytes, not model tokens. Recompile after access or source changes.".into()]};
    if json(&packet)?.len() > request.max_bytes {
        return Err("budget cannot hold complete omission receipt".into());
    }
    let mut visited = BTreeSet::new();
    for start in assertions.keys() {
        if visited.contains(start) {
            continue;
        }
        let mut group = BTreeSet::new();
        let mut stack = vec![start.clone()];
        while let Some(id) = stack.pop() {
            if group.insert(id.clone()) {
                stack.extend(adjacency[&id].iter().cloned());
            }
        }
        visited.extend(group.iter().cloned());
        if group.iter().any(|id| !eligible.contains(id)) {
            for id in group {
                packet.omissions.insert(id,"unavailable, expired, unselected, nonredistributable, or linked to excluded claim".into());
            }
            continue;
        }
        let mut candidate = packet.clone();
        for id in &group {
            let a = &assertions[id];
            let mut citations = Vec::new();
            for span in &a.evidence {
                let s = &sources[&span.source];
                citations.push(Quote {
                    span: span.clone(),
                    text: s.passage(span)?.into(),
                    title: s.title.clone(),
                    family: s.source_family.clone(),
                    license: s.license.clone().unwrap_or_default(),
                });
            }
            candidate.claims.insert(
                id.clone(),
                ClaimLayer {
                    assertion: a.clone(),
                    knowledge: graph
                        .knowledge
                        .get(id)
                        .cloned()
                        .unwrap_or(Knowledge::Unknown),
                    citations,
                },
            );
            candidate.omissions.remove(id);
        }
        candidate.entities = graph
            .nodes
            .iter()
            .filter(|(_, n)| n.claims.iter().all(|id| candidate.claims.contains_key(id)))
            .map(|(id, n)| (id.clone(), n.clone()))
            .collect();
        candidate.relationships = links
            .iter()
            .filter(|l| {
                candidate.claims.contains_key(&l.from) && candidate.claims.contains_key(&l.to)
            })
            .cloned()
            .collect();
        if json(&candidate)?.len() <= request.max_bytes {
            packet = candidate;
        }
    }
    let encoded = json(&packet)?;
    if encoded.len() > request.max_bytes {
        return Err("omission receipt exceeds budget".into());
    }
    Ok(Export {
        digest: digest(&encoded),
        bytes: encoded.len(),
        packet,
        budget_unit: "UTF-8 bytes; full packet; no tokenizer estimate".into(),
    })
}
