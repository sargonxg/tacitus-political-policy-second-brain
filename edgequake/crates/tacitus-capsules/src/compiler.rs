use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use tacitus_core::{digest, DecisionContract, Result, SourceVersion, Span};

#[derive(Debug, Clone)]
pub struct Candidate {
    pub source: String,
    pub semantic_relevance: f64,
}
/// An adapter must constrain vector AND graph expansion to `allowed` BEFORE retrieval.
pub trait Retrieval {
    fn retrieve(&self, question: &str, allowed: &BTreeSet<String>) -> Result<Vec<Candidate>>;
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Omission {
    pub source: String,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidencePacket {
    pub compiler_version: String,
    pub contract_digest: String,
    pub sources: BTreeMap<String, String>,
    pub citations: BTreeMap<String, Span>,
    pub omissions: Vec<Omission>,
    pub source_families: BTreeMap<String, Vec<String>>,
    pub evidence_byte_budget: usize,
    pub used_bytes: usize,
    pub warnings: Vec<String>,
}
fn eligible(source: &SourceVersion, contract: &DecisionContract) -> bool {
    source.workspace == contract.workspace
        && !source.withdrawn
        && !source.restricted
        && contract.selected_sources.contains(&source.key())
        && source.available.contains(contract.known_as_of)
        && source.valid.contains(contract.valid_at)
        && source.published_at <= contract.known_as_of
        && source.retrieved_at <= contract.known_as_of
}
pub fn compile(
    contract: &DecisionContract,
    sources: &BTreeMap<String, SourceVersion>,
    retrieval: &dyn Retrieval,
) -> Result<EvidencePacket> {
    let budget = contract.evidence_budget()?;
    if !contract.selected_capsules.is_empty() {
        return Err("capsule selection resolution not integrated; explicitly import authorized evidence first".into());
    }
    let mut allowed = BTreeSet::new();
    let mut omissions = Vec::new();
    for key in &contract.selected_sources {
        match sources.get(key) {
            Some(source) if source.key() == *key && eligible(source, contract) => {
                source.validate()?;
                allowed.insert(key.clone());
            }
            _ => omissions.push(Omission {
                source: key.clone(),
                reason:
                    "unavailable, unselected, unauthorized, withdrawn, or outside temporal scope"
                        .into(),
            }),
        }
    }
    if allowed.is_empty() {
        return Err("no authorized evidence at requested dates".into());
    }
    let results = retrieval.retrieve(&contract.question, &allowed)?;
    if results.len() > 4096 {
        return Err("retrieval exceeded candidate bound".into());
    }
    let mut ranks = BTreeMap::<String, f64>::new();
    for candidate in results {
        let Some(source) = sources.get(&candidate.source) else {
            return Err("retrieval returned unknown evidence".into());
        };
        if !allowed.contains(&candidate.source) || !eligible(source, contract) {
            return Err("retrieval scope violation; packet refused".into());
        }
        if !candidate.semantic_relevance.is_finite()
            || !(0.0..=1.0).contains(&candidate.semantic_relevance)
        {
            return Err("invalid semantic relevance".into());
        }
        ranks
            .entry(candidate.source)
            .and_modify(|s| *s = s.max(candidate.semantic_relevance))
            .or_insert(candidate.semantic_relevance);
    }
    // Lexical fallback has the same authorized input. Whole versions retain qualifiers.
    let words: BTreeSet<_> = contract
        .question
        .split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|w| w.len() > 2)
        .collect();
    let mut ranked: Vec<_> = allowed
        .iter()
        .map(|key| {
            let text = sources[key].text.to_lowercase();
            let lexical = words.iter().filter(|w| text.contains(w.as_str())).count() as f64
                / words.len().max(1) as f64;
            (
                key.clone(),
                ranks.get(key).copied().unwrap_or(0.0) + lexical,
            )
        })
        .collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let mut packet = EvidencePacket { compiler_version: "0.1.0".into(), contract_digest: digest(&serde_json::to_vec(contract).map_err(|e| e.to_string())?), sources: BTreeMap::new(), citations: BTreeMap::new(), omissions, source_families: BTreeMap::new(), evidence_byte_budget: budget, used_bytes: 0,
        warnings: vec!["Conservative UTF-8 byte budget, not a measured model tokenizer. Semantic support and decision-critical coverage require review.".into(), "Full-source lexical/semantic ranking; graph expansion and reviewed-method/dependency prioritization not yet implemented.".into()] };
    for (key, _) in ranked {
        let source = &sources[&key];
        // Bound text plus identifiers/framing; never silently cut a source mid-qualification.
        let cost = source
            .text
            .len()
            .checked_add(key.len())
            .and_then(|n| n.checked_add(64))
            .ok_or("size overflow")?;
        if cost > budget - packet.used_bytes {
            packet.omissions.push(Omission { source: key, reason: "whole source omitted: input/output budget; enlarge budget or select a smaller version".into() });
            continue;
        }
        packet.used_bytes += cost;
        let citation = format!("E{}", packet.citations.len() + 1);
        packet.citations.insert(
            citation,
            Span {
                source: key.clone(),
                text_hash: source.text_hash.clone(),
                start: 0,
                end: source.text.len(),
                locator: None,
                original_language: source.language.clone(),
                translation: None,
            },
        );
        packet
            .source_families
            .entry(source.source_family.clone())
            .or_default()
            .push(key.clone());
        packet.sources.insert(key, source.text.clone());
    }
    if packet.sources.is_empty() {
        return Err("no whole source fits context; evidence not truncated".into());
    }
    Ok(packet)
}
