//! Versioned policy records. Model proposals and imported attestations grant no authority.
use crate::{digest, EpistemicType, Result, Span};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRecord {
    pub id: String,
    pub version: u32,
    pub workspace: String,
    pub document_id: Option<String>,
    pub title: String,
    pub author: Option<String>,
    pub original_hash: Option<String>,
    pub text_hash: String,
    pub text: String,
    pub language: String,
    pub extractor: String,
    pub published_at: Option<i64>,
    pub retrieved_at: i64,
    pub available_at: i64,
    pub valid_from: Option<i64>,
    pub valid_until: Option<i64>,
    pub source_family: String,
    pub withdrawn: bool,
    pub restricted: bool,
    pub redistributable: bool,
    pub license: Option<String>,
}
impl SourceRecord {
    pub fn key(&self) -> String {
        format!("{}@{}", self.id, self.version)
    }
    pub fn validate(&self) -> Result<()> {
        if self.id.is_empty()
            || self.workspace.is_empty()
            || self.version == 0
            || self.language.is_empty()
            || self.extractor.is_empty()
            || self.source_family.is_empty()
            || self.text.trim().is_empty()
            || self.text.len() > 2 * 1024 * 1024
            || self.text_hash != digest(self.text.as_bytes())
            || self
                .original_hash
                .as_ref()
                .is_some_and(|h| h.len() != 64 || !h.bytes().all(|b| b.is_ascii_hexdigit()))
            || self.available_at < self.retrieved_at
            || self
                .valid_from
                .zip(self.valid_until)
                .is_some_and(|(a, b)| a >= b)
        {
            return Err("invalid policy source/version/digest/interval".into());
        }
        Ok(())
    }
    pub fn eligible(&self, workspace: &str, known: i64, valid: Option<i64>) -> bool {
        self.workspace == workspace
            && !self.withdrawn
            && !self.restricted
            && self.available_at <= known
            && self.published_at.is_none_or(|t| t <= known)
            && valid.is_none_or(|t| {
                self.valid_from.is_none_or(|a| a <= t) && self.valid_until.is_none_or(|b| t < b)
            })
    }
    pub fn passage(&self, span: &Span) -> Result<&str> {
        if span.source != self.key()
            || span.text_hash != self.text_hash
            || span.original_language != self.language
            || span.start >= span.end
        {
            return Err("source passage version/hash/language mismatch".into());
        }
        self.text
            .get(span.start..span.end)
            .ok_or_else(|| "invalid UTF-8 passage boundaries".into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyAssertion {
    pub id: String,
    pub version: u32,
    pub workspace: String,
    pub speaker: String,
    pub subject: String,
    pub predicate: String,
    pub object: Option<String>,
    pub proposition: String,
    pub epistemic_type: EpistemicType,
    pub modality: String,
    pub conditions: Vec<String>,
    pub exceptions: Vec<String>,
    pub evidence: Vec<Span>,
    pub created_at: i64,
}
impl PolicyAssertion {
    pub fn fingerprint(&self) -> String {
        digest(&serde_json::to_vec(self).expect("serializable assertion"))
    }
    pub fn validate(&self, sources: &BTreeMap<String, SourceRecord>) -> Result<()> {
        if self.id.is_empty()
            || self.version == 0
            || self.speaker.trim().is_empty()
            || self.subject.trim().is_empty()
            || self.proposition.trim().is_empty()
            || self.modality.trim().is_empty()
            || self.evidence.is_empty()
            || !RELATIONS.contains(&self.predicate.as_str())
        {
            return Err("attributed source-bound assertion required".into());
        }
        for span in &self.evidence {
            let source = sources
                .get(&span.source)
                .ok_or("missing assertion source")?;
            if source.workspace != self.workspace || source.withdrawn || source.restricted {
                return Err("assertion source unauthorized".into());
            }
            source.passage(span)?;
        }
        Ok(())
    }
}

pub const ONTOLOGY_VERSION: &str = "tacitus-policy/1";
pub const RELATIONS: &[&str] = &[
    "states",
    "expresses_position",
    "conditionally_undertakes",
    "authorizes",
    "constrains",
    "funds",
    "implements",
    "withdraws",
    "contradicts",
    "depends_on",
    "affiliated_with",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CognitiveMethod {
    pub id: String,
    pub version: u32,
    pub workspace: String,
    pub kind: String,
    pub description: String,
    pub scope: String,
    pub prerequisites: Vec<String>,
    pub failure_conditions: Vec<String>,
    pub evidence: Vec<Span>,
    pub created_at: i64,
    pub available_at: i64,
    pub ontology_version: String,
}
impl CognitiveMethod {
    pub fn fingerprint(&self) -> String {
        digest(&serde_json::to_vec(self).expect("serializable method"))
    }
    pub fn validate(&self, sources: &BTreeMap<String, SourceRecord>) -> Result<()> {
        if self.id.is_empty()
            || self.version == 0
            || self.description.trim().is_empty()
            || self.description.len() > 4000
            || self.scope.trim().is_empty()
            || self.failure_conditions.is_empty()
            || self.failure_conditions.iter().any(|s| s.trim().is_empty())
            || self.prerequisites.is_empty()
            || self.prerequisites.iter().any(|s| s.trim().is_empty())
            || self.evidence.is_empty()
            || self.available_at < self.created_at
            || self.ontology_version != ONTOLOGY_VERSION
            || ![
                "distinction",
                "cue",
                "heuristic",
                "pattern",
                "exception",
                "red_flag",
                "language_rule",
                "procedure",
                "case",
                "unknown",
                "revision_trigger",
            ]
            .contains(&self.kind.as_str())
        {
            return Err("scoped method with provenance/failure conditions required".into());
        }
        for span in &self.evidence {
            let source = sources
                .get(&span.source)
                .ok_or("missing method provenance")?;
            if source.workspace != self.workspace || source.restricted || source.withdrawn {
                return Err("method provenance unavailable".into());
            }
            source.passage(span)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HumanReview {
    pub id: String,
    pub target_id: String,
    pub target_kind: String,
    pub target_fingerprint: String,
    pub evidence_hashes: BTreeMap<String, String>,
    pub reviewer: String,
    pub decision: String,
    pub rationale: String,
    pub reviewed_at: i64,
    pub origin: String,
}
impl HumanReview {
    pub fn accepted(&self, fingerprint: &str, now: i64, evidence: &[Span]) -> bool {
        self.origin == "authenticated_human"
            && !self.reviewer.is_empty()
            && !self.rationale.trim().is_empty()
            && ["accept", "caveat"].contains(&self.decision.as_str())
            && self.target_fingerprint == fingerprint
            && self.reviewed_at <= now
            && evidence
                .iter()
                .all(|e| self.evidence_hashes.get(&e.source) == Some(&e.text_hash))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Applicability {
    pub state: String,
    pub reasons: Vec<String>,
}
pub fn applicability(
    method: &CognitiveMethod,
    review: Option<&HumanReview>,
    sources: &BTreeMap<String, SourceRecord>,
    workspace: &str,
    known: i64,
    facts: &BTreeMap<String, bool>,
) -> Applicability {
    let result = |state: &str, reason: &str| Applicability {
        state: state.into(),
        reasons: vec![reason.into()],
    };
    if method.workspace != workspace || method.available_at > known {
        return result("blocked", "scope or knowledge cutoff");
    }
    if method.validate(sources).is_err()
        || method.evidence.iter().any(|e| {
            sources
                .get(&e.source)
                .is_none_or(|s| !s.eligible(workspace, known, None))
        })
    {
        return result("stale", "provenance changed or unavailable");
    }
    if !review.is_some_and(|r| {
        r.target_id == method.id
            && r.target_kind == "method"
            && r.accepted(&method.fingerprint(), known, &method.evidence)
    }) {
        return result("guarded", "authenticated review required");
    }
    for prerequisite in &method.prerequisites {
        match facts.get(prerequisite) {
            Some(false) => return result("blocked", "false prerequisite"),
            None => return result("guarded", "unknown prerequisite"),
            _ => {}
        }
    }
    result(
        "active",
        "current reviewed method; applicability explicitly declared",
    )
}
