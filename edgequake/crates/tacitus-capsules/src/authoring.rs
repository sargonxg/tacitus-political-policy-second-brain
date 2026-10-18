//! Conservative authoring entrypoint for original research records with unknown world validity.
//! The envelope reuses native manifest/data/evidence. Declarations are not proof of ownership.
use crate::{
    policy::{Data, PolicyPackage},
    strict_json, Manifest,
};
use serde::Deserialize;
use std::collections::BTreeMap;
use tacitus_core::Result;

pub const SUMMARY_EXTRACTOR: &str =
    "Original assistant-authored paraphrase/analysis; no publisher-text extraction";
pub const UNKNOWN_WARNING: &str = "World-validity interval is unknown";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    manifest: Manifest,
    data: Data,
    evidence: BTreeMap<String, String>,
}

/// Author-declared original summary bytes only. Caller must separately authorize sharing.
/// No guessed dates, rights, hashes, spans, publisher attribution, or review attestations.
pub fn pack_original_summaries(raw: &[u8], confirms_original_evidence: bool) -> Result<Vec<u8>> {
    if !confirms_original_evidence {
        return Err("explicit original-evidence declaration required".into());
    }
    let input: Input = strict_json::from_slice(raw)?;
    let p = PolicyPackage {
        manifest: input.manifest,
        data: input.data,
        evidence: input
            .evidence
            .into_iter()
            .map(|(k, v)| (k, v.into_bytes()))
            .collect(),
    };
    if p.manifest.schema_version != 2
        || p.manifest.evidence_mode != "evidence_included"
        || !p.manifest.files.is_empty()
        || !p.manifest.dependencies.is_empty()
        || !p.data.reviews.is_empty()
        || p.data.sources.is_empty()
        || p.data.assertions.is_empty()
        || !p
            .manifest
            .limitations
            .iter()
            .any(|s| s.contains(UNKNOWN_WARNING))
    {
        return Err("authoring requires native-v2 original summaries, explicit unknown validity, no imported files/dependencies/reviews".into());
    }
    for s in &p.data.sources {
        if s.author.as_deref() != Some(p.manifest.author.as_str())
            || s.extractor != SUMMARY_EXTRACTOR
            || !s.source_family.starts_with("tacitus-authored-summary:")
            || s.original_hash.as_deref() != Some(s.text_hash.as_str())
            || s.valid_from.is_some()
            || s.valid_until.is_some()
            || s.available_at > p.manifest.created_at
            || s.published_at.is_some_and(|t| t > s.available_at)
            || !s.redistributable
            || s.license.as_ref().is_none_or(|v| v.trim().is_empty())
        {
            return Err("source must declare original summary authorship/rights and unknown world validity; chronology must be explicit".into());
        }
    }
    p.validate()?;
    let sources = p.sources()?;
    for a in &p.data.assertions {
        if a.created_at > p.manifest.created_at
            || !a.conditions.iter().any(|s| s.contains(UNKNOWN_WARNING))
        {
            return Err("assertion chronology/unknown-validity disclosure required".into());
        }
        a.validate(&sources)?;
    }
    for m in &p.data.methods {
        if m.created_at > p.manifest.created_at || m.available_at > p.manifest.created_at {
            return Err("method chronology exceeds authored record".into());
        }
        m.validate(&sources)?;
    }
    let bytes = p.export()?;
    let restored = PolicyPackage::import_strict(&bytes)?;
    if serde_json::to_value(&restored.data).map_err(|e| e.to_string())?
        != serde_json::to_value(&p.data).map_err(|e| e.to_string())?
        || restored.evidence != p.evidence
    {
        return Err("native-v2 authoring roundtrip changed content".into());
    }
    Ok(bytes)
}
