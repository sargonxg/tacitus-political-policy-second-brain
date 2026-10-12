//! Tacitus canonical records. Retrieval indexes are derived projections.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub type Result<T> = std::result::Result<T, String>;
pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Timestamps are integer UTC Unix seconds. Intervals are [start, end).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Interval {
    pub start: i64,
    pub end: Option<i64>,
}
impl Interval {
    pub fn contains(&self, time: i64) -> bool {
        time >= self.start && self.end.is_none_or(|end| time < end)
    }
    pub fn validate(&self) -> Result<()> {
        if self.end.is_some_and(|end| end <= self.start) {
            return Err("interval end must follow start".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpistemicType {
    ReportedStatement,
    Observation,
    Allegation,
    Inference,
    Hypothesis,
    Forecast,
    ValueJudgment,
    Recommendation,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewDecision {
    Accept,
    Reject,
    Caveat,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewOrigin {
    LocalHuman,
    ImportedAttestation,
    ModelSuggestion,
    SyntheticTest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceVersion {
    pub id: String,
    pub version: u32,
    pub workspace: String,
    pub title: String,
    pub author: String,
    pub original_hash: String,
    pub text_hash: String,
    pub text: String,
    pub language: String,
    pub extraction_tool: String,
    pub published_at: i64,
    pub retrieved_at: i64,
    pub available: Interval,
    pub valid: Interval,
    pub source_family: String,
    pub withdrawn: bool,
    pub restricted: bool,
    pub redistributable: bool,
    pub license: String,
}
impl SourceVersion {
    pub fn key(&self) -> String {
        format!("{}@{}", self.id, self.version)
    }
    pub fn validate(&self) -> Result<()> {
        self.available.validate()?;
        self.valid.validate()?;
        if self.id.is_empty()
            || self.workspace.is_empty()
            || self.version == 0
            || self.author.is_empty()
            || self.extraction_tool.is_empty()
            || self.language.is_empty()
            || self.source_family.is_empty()
            || self.text.trim().is_empty()
            || digest(self.text.as_bytes()) != self.text_hash
            || self.original_hash.len() != 64
            || !self.original_hash.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err("invalid source identity, extraction, or digest".into());
        }
        if self.available.start < self.published_at || self.available.start < self.retrieved_at {
            return Err("knowledge availability precedes publication or retrieval".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SelectedTextDocument {
    /// Original bytes are private runtime data, never automatically exported.
    pub original: Vec<u8>,
    pub source: SourceVersion,
}
impl SelectedTextDocument {
    /// Explicitly selected TXT/Markdown/pasted UTF-8 only. No filesystem crawling.
    /// Metadata must be supplied by the caller, never invented by the extractor.
    pub fn ingest(original: Vec<u8>, mut metadata: SourceVersion) -> Result<Self> {
        if original.len() > 2 * 1024 * 1024 {
            return Err("selected text exceeds 2 MiB limit".into());
        }
        metadata.text = String::from_utf8(original.clone())
            .map_err(|_| "malformed UTF-8; select a verified local extractor")?;
        metadata.original_hash = digest(&original);
        metadata.text_hash = digest(metadata.text.as_bytes());
        metadata.extraction_tool = "tacitus-utf8/0.1.0".into();
        metadata.validate()?;
        Ok(Self {
            original,
            source: metadata,
        })
    }
}
pub trait SourceImport {
    fn import_selected(
        &self,
        bytes: &[u8],
        metadata: SourceVersion,
    ) -> Result<SelectedTextDocument>;
}
pub trait CanonicalStorage {
    /// Implementations must authenticate access and atomically compare expected versions.
    fn append_review(
        &self,
        authenticated_actor: &str,
        expected_fingerprint: &str,
        receipt: ReviewReceipt,
    ) -> Result<()>;
    fn withdraw_source(
        &self,
        authenticated_actor: &str,
        source_version: &str,
    ) -> Result<Vec<String>>;
}

/// UTF-8 byte offsets in immutable extracted text, half-open [start, end).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Span {
    pub source: String,
    pub text_hash: String,
    pub start: usize,
    pub end: usize,
    pub locator: Option<String>,
    pub original_language: String,
    pub translation: Option<String>,
}
impl Span {
    pub fn passage<'a>(&self, source: &'a SourceVersion) -> Result<&'a str> {
        if self.source != source.key()
            || self.text_hash != source.text_hash
            || self.original_language != source.language
            || self.start >= self.end
        {
            return Err("citation identity/version mismatch".into());
        }
        source
            .text
            .get(self.start..self.end)
            .ok_or_else(|| "invalid UTF-8 citation offsets".into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Assertion {
    pub id: String,
    pub version: u32,
    pub workspace: String,
    pub profile: String,
    pub speaker: String,
    pub proposition: String,
    pub subject: String,
    pub object: Option<String>,
    pub epistemic_type: EpistemicType,
    pub polarity: String,
    pub modality: String,
    pub conditions: Vec<String>,
    pub exceptions: Vec<String>,
    pub audience: String,
    pub speech_act: String,
    pub valid: Interval,
    pub available: Interval,
    pub evidence: Vec<Span>,
    pub dependencies: Vec<String>,
    pub challenges: Vec<String>,
    pub supersedes: Option<String>,
    pub confidence: String,
    pub confidence_rationale: String,
}
impl Assertion {
    pub fn fingerprint(&self) -> String {
        digest(&serde_json::to_vec(self).expect("fixed serializable assertion"))
    }
    pub fn validate(&self, sources: &BTreeMap<String, SourceVersion>) -> Result<()> {
        self.valid.validate()?;
        self.available.validate()?;
        if self.id.is_empty()
            || self.version == 0
            || self.proposition.trim().is_empty()
            || self.speaker.is_empty()
            || self.evidence.is_empty()
            || self.confidence_rationale.is_empty()
        {
            return Err("substantive assertions need attribution, evidence, and rationale".into());
        }
        for span in &self.evidence {
            let source = sources.get(&span.source).ok_or("missing source version")?;
            if source.workspace != self.workspace || source.withdrawn || source.restricted {
                return Err("assertion evidence not accessible".into());
            }
            span.passage(source)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewReceipt {
    pub assertion_id: String,
    pub assertion_fingerprint: String,
    pub evidence_hashes: BTreeMap<String, String>,
    pub reviewer: String,
    pub rationale: String,
    pub reviewed_at: i64,
    pub decision: ReviewDecision,
    pub origin: ReviewOrigin,
}
impl ReviewReceipt {
    /// Only locally authenticated human actions can earn the UI label.
    /// The server must set origin and reviewer; client payloads are not authority.
    pub fn is_current_human_review(
        &self,
        assertion: &Assertion,
        sources: &BTreeMap<String, SourceVersion>,
    ) -> bool {
        self.origin == ReviewOrigin::LocalHuman
            && !self.reviewer.is_empty()
            && !self.rationale.is_empty()
            && self.assertion_id == assertion.id
            && self.assertion_fingerprint == assertion.fingerprint()
            && assertion.validate(sources).is_ok()
            && assertion
                .evidence
                .iter()
                .all(|span| self.evidence_hashes.get(&span.source) == Some(&span.text_hash))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionContract {
    pub workspace: String,
    pub question: String,
    pub decision_maker: String,
    pub audience: String,
    pub mandate: String,
    pub objectives: Vec<String>,
    pub constraints: Vec<String>,
    pub time_horizon: String,
    pub known_as_of: i64,
    pub valid_at: i64,
    pub selected_sources: BTreeSet<String>,
    pub selected_capsules: BTreeSet<String>,
    pub permitted_tools: BTreeSet<String>,
    pub output_format: String,
    pub context_tokens: usize,
    pub output_tokens: usize,
    pub overhead_tokens: usize,
}
impl DecisionContract {
    pub fn evidence_budget(&self) -> Result<usize> {
        if self.workspace.is_empty()
            || self.question.trim().is_empty()
            || self.decision_maker.is_empty()
            || self.audience.is_empty()
            || self.mandate.is_empty()
            || self.objectives.is_empty()
            || self.time_horizon.is_empty()
            || self.output_format.is_empty()
            || self.selected_sources.is_empty()
            || self.output_tokens == 0
        {
            return Err("incomplete decision contract".into());
        }
        self.context_tokens
            .checked_sub(self.output_tokens)
            .and_then(|n| n.checked_sub(self.overhead_tokens))
            .filter(|n| *n > 0)
            .ok_or_else(|| "context too small; retain input AND output budgets".into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalModelPolicy {
    pub endpoint: String,
    pub chat_model: String,
    pub embedding_model: String,
}
impl LocalModelPolicy {
    pub fn validate(
        &self,
        provider: &str,
        endpoint: &str,
        model: &str,
        embedding: bool,
        connection: Option<&str>,
    ) -> Result<()> {
        if ![
            "http://ollama:11434",
            "http://host.docker.internal:11434",
            "http://127.0.0.1:11434",
        ]
        .contains(&self.endpoint.as_str())
            || provider != "ollama"
            || endpoint != self.endpoint
            || connection.is_some()
            || model
                != if embedding {
                    &self.embedding_model
                } else {
                    &self.chat_model
                }
        {
            return Err("local-only policy: approved Ollama identity required; saved connections/fallbacks forbidden".into());
        }
        Ok(())
    }
}

/// Provisional adapter boundary; no private Praxis schema is included.
pub trait PraxisAdapter {
    fn compatibility(&self) -> &'static str {
        "provisional; compatibility unverified"
    }
    fn encode_proposal(&self, assertions: &[Assertion]) -> Result<Vec<u8>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn digest_known_vector() {
        assert_eq!(
            digest(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
    #[test]
    fn half_open_interval() {
        let interval = Interval {
            start: 10,
            end: Some(20),
        };
        assert!(interval.contains(10));
        assert!(!interval.contains(20));
        assert!(Interval {
            start: 20,
            end: Some(10)
        }
        .validate()
        .is_err());
    }
    #[test]
    fn local_identity_is_exact() {
        let p = LocalModelPolicy {
            endpoint: "http://ollama:11434".into(),
            chat_model: "qwen3:4b".into(),
            embedding_model: "qwen3-embedding:0.6b".into(),
        };
        assert!(p
            .validate("ollama", &p.endpoint, &p.chat_model, false, None)
            .is_ok());
        for endpoint in [
            "https://api.openai.com",
            "http://ollama:11434.evil",
            "http://ollama:11434/path",
        ] {
            assert!(p
                .validate("ollama", endpoint, &p.chat_model, false, None)
                .is_err());
        }
        assert!(p
            .validate("ollama", &p.endpoint, &p.chat_model, false, Some("saved"))
            .is_err());
        assert!(p
            .validate("mock", &p.endpoint, &p.chat_model, false, None)
            .is_err());
        assert!(p
            .validate("ollama", &p.endpoint, "different-vector-space", true, None)
            .is_err());
    }
}
