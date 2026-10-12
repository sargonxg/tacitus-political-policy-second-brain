//! Evaluation records distinguish measured automatic checks from pending human judgments.
use serde::{Deserialize, Serialize};
use tacitus_core::Result;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Arm {
    FullDocuments,
    EdgeQuake,
    TacitusContext,
    TacitusReviewedMemory,
    LexicalVector,
    ExternalNotRun,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Run {
    pub arm: Arm,
    pub model_digest: String,
    pub dataset_digest: String,
    pub prompt_version: String,
    pub compiler_version: String,
    pub known_as_of: i64,
    pub evidence_access_digest: String,
    pub prior_notes_digest: String,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub output_budget: u64,
    pub latency_ms: Option<u64>,
    pub ingestion_ms: Option<u64>,
    pub retrieval_ms: Option<u64>,
    pub cache_state: String,
    pub automatic_citation_checks: Option<bool>,
    pub human_critical_errors: Option<u32>,
    pub human_coverage: Option<String>,
    pub human_qualifications: Option<String>,
    pub human_material_omissions: Option<String>,
    pub human_temporal_errors: Option<u32>,
    pub analyst_corrections: Option<u32>,
    pub time_to_acceptable_brief_ms: Option<u64>,
    pub status: String,
}
pub fn comparable(a: &Run, b: &Run) -> Result<()> {
    if a.model_digest.is_empty()
        || a.dataset_digest.is_empty()
        || a.evidence_access_digest.is_empty()
        || a.model_digest != b.model_digest
        || a.dataset_digest != b.dataset_digest
        || a.known_as_of != b.known_as_of
        || a.evidence_access_digest != b.evidence_access_digest
        || a.prior_notes_digest != b.prior_notes_digest
        || a.output_budget != b.output_budget
    {
        return Err(
            "comparison has unequal model/dataset/evidence/time/prior-notes/output access".into(),
        );
    }
    if a.status != "measured"
        || b.status != "measured"
        || a.latency_ms.is_none()
        || b.latency_ms.is_none()
        || a.arm == Arm::ExternalNotRun
        || b.arm == Arm::ExternalNotRun
    {
        return Err("unmeasured runs cannot support benchmark claims".into());
    }
    Ok(())
}
