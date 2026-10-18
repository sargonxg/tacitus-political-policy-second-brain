//! Extractive native-v1 brief. Explicit caller priority, no model or semantic ranking.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use tacitus_capsules::{Manifest, Method, Package};
use tacitus_core::{digest, Assertion, Result, SourceVersion, Span};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub question: String,
    pub workspace: String,
    pub known_as_of: i64,
    pub valid_at: i64,
    pub priority: Vec<String>,
    pub max_bytes: usize,
    /// Current host access state, never obtained from the imported archive.
    pub revoked_sources: BTreeSet<String>,
    pub package_revoked: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Citation {
    pub span: Span,
    pub excerpt: String,
    /// Metadata text remains empty; excerpt is bound to the original text hash.
    pub source: SourceVersion,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub assertion: Assertion,
    pub citations: Vec<Citation>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Brief {
    pub format: String,
    pub trust: String,
    pub question: String,
    pub workspace: String,
    pub known_as_of: i64,
    pub valid_at: i64,
    pub manifest: Manifest,
    /// Original attribution/limitations artifacts, treated as untrusted data.
    pub artifacts: BTreeMap<String, String>,
    pub methods: Vec<Method>,
    pub entries: Vec<Entry>,
    pub omissions: BTreeMap<String, String>,
    pub warnings: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Export {
    pub brief: Brief,
    pub digest: String,
    pub bytes: usize,
}
fn bytes(brief: &Brief) -> Result<Vec<u8>> {
    serde_json::to_vec(brief).map_err(|e| e.to_string())
}
pub fn compile(package: &Package, request: &Request) -> Result<Export> {
    package.validate()?;
    if request.question.trim().is_empty()
        || request.question.len() > 4096
        || request.workspace.is_empty()
        || request.workspace.len() > 256
        || !(1024..=250_000).contains(&request.max_bytes)
        || request.priority.is_empty()
        || request.priority.len() > 256
        || request.revoked_sources.len() > 32
        || request.package_revoked
    {
        return Err("invalid request, resource bound or revoked package".into());
    }
    if package.manifest.created_at > request.known_as_of
        || package
            .manifest
            .valid_until
            .is_some_and(|end| request.valid_at >= end)
        || !package.manifest.dependencies.is_empty()
    {
        return Err("package unavailable, expired or unresolved dependencies".into());
    }
    // v1 relations require resolution before independent selection can be sound.
    if package
        .data
        .assertions
        .iter()
        .any(|a| !a.dependencies.is_empty() || !a.challenges.is_empty() || a.supersedes.is_some())
    {
        return Err("related assertions need a grouped native-v1 resolver".into());
    }
    let sources = package.sources_for_workspace(&request.workspace)?;
    let assertions: BTreeMap<_, _> = package
        .data
        .assertions
        .iter()
        .map(|a| (a.id.clone(), a))
        .collect();
    let mut seen = BTreeSet::new();
    for id in &request.priority {
        if !assertions.contains_key(id) || !seen.insert(id) {
            return Err("unknown or duplicate priority assertion".into());
        }
    }
    // Opaque prose may repeat any claim or evidence. Without a dependency graph,
    // it can only accompany an intact, fully eligible package.
    let atomic = !package.data.artifacts.is_empty() || !package.data.methods.is_empty();
    if atomic
        && (seen.len() != assertions.len()
            || sources.values().any(|s| {
                request.revoked_sources.contains(&s.key())
                    || s.withdrawn
                    || s.restricted
                    || !s.redistributable
                    || s.license.trim().is_empty()
                    || !s.available.contains(request.known_as_of)
                    || !s.valid.contains(request.valid_at)
            }))
    {
        return Err("opaque artifacts or methods require all claims and eligible sources".into());
    }
    let mut brief = Brief {
        format: "tacitus-native-extractive-brief/1".into(),
        trust: "untrusted-data; unreviewed extractive proposal".into(),
        question: request.question.clone(), workspace: request.workspace.clone(),
        known_as_of: request.known_as_of, valid_at: request.valid_at,
        manifest: package.manifest.clone(), methods: package.data.methods.iter().cloned().map(|mut method| { method.review_state="unreviewed_proposal".into(); method }).collect(), artifacts: package.data.artifacts.clone(),
        entries: vec![], omissions: assertions.keys().map(|id| (id.clone(), "not selected".into())).collect(),
        warnings: vec!["Caller-selected priority; relevance, truth and entailment are unverified.".into(),
            "Exact excerpts may be authored research summaries; inspect source author and extraction metadata.".into(),
            "No answer or recommendation is generated. Recheck current access and revocations before reuse.".into()],
    };
    for id in &request.priority {
        let original = assertions[id];
        let eligible = original.available.contains(request.known_as_of)
            && original.valid.contains(request.valid_at)
            && original.evidence.iter().all(|span| {
                sources.get(&span.source).is_some_and(|s| {
                    !request.revoked_sources.contains(&s.key())
                        && !s.withdrawn
                        && !s.restricted
                        && s.redistributable
                        && !s.license.trim().is_empty()
                        && s.available.contains(request.known_as_of)
                        && s.valid.contains(request.valid_at)
                })
            });
        if !eligible && atomic {
            return Err("opaque package contains unavailable claim".into());
        }
        if !eligible {
            brief
                .omissions
                .insert(id.clone(), "time, access or rights exclusion".into());
            continue;
        }
        let mut assertion = original.clone();
        assertion.workspace = request.workspace.clone();
        assertion.validate(&sources)?;
        let citations = assertion
            .evidence
            .iter()
            .map(|span| {
                let s = &sources[&span.source];
                let excerpt = span.passage(s)?.to_string();
                let mut source = s.clone();
                source.text.clear();
                Ok(Citation {
                    span: span.clone(),
                    excerpt,
                    source,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        brief.omissions.remove(id);
        brief.entries.push(Entry {
            assertion,
            citations,
        });
        if bytes(&brief)?.len() > request.max_bytes {
            if atomic {
                return Err("opaque package cannot be partially budgeted".into());
            }
            brief.entries.pop();
            brief.omissions.insert(
                id.clone(),
                "whole qualified claim exceeds byte budget".into(),
            );
        }
    }
    let data = bytes(&brief)?;
    if data.len() > request.max_bytes {
        return Err("attribution and omission receipt cannot fit byte budget".into());
    }
    Ok(Export {
        digest: digest(&data),
        bytes: data.len(),
        brief,
    })
}

/// A deterministic plain-text review document. Its independent byte bound includes
/// the complete JSON audit appendix, so a readable view cannot erase qualifications.
pub fn render_text(export: &Export, max_bytes: usize) -> Result<String> {
    let raw = bytes(&export.brief)?;
    if export.bytes != raw.len()
        || export.digest != digest(&raw)
        || !(1024..=1_000_000).contains(&max_bytes)
    {
        return Err("export integrity or document budget invalid".into());
    }
    let b = &export.brief;
    let mut text = format!("TACITUS EXTRACTIVE POLICY REVIEW\n{}\n\nQuestion: {}\nKnown as of (Unix seconds): {}\nValid at (Unix seconds): {}\nState: {}\n\n",b.manifest.title,b.question,b.known_as_of,b.valid_at,b.trust);
    for (i, entry) in b.entries.iter().enumerate() {
        let a = &entry.assertion;
        text.push_str(&format!("{}. {}\nReported by: {}\nModality: {}\nConditions: {}\nExceptions: {}\nQualification: {}\n",i+1,a.proposition,a.speaker,a.modality,serde_json::to_string(&a.conditions).map_err(|e|e.to_string())?,serde_json::to_string(&a.exceptions).map_err(|e|e.to_string())?,a.confidence_rationale));
        for c in &entry.citations {
            text.push_str(&format!(
                "Evidence [{} bytes {}..{}]: {}\nAuthor: {}; extraction: {}\nEXCERPT DATA: {}\n",
                c.span.source,
                c.span.start,
                c.span.end,
                c.source.title,
                c.source.author,
                c.source.extraction_tool,
                serde_json::to_string(&c.excerpt).map_err(|e| e.to_string())?
            ));
        }
        text.push('\n');
    }
    text.push_str("Complete qualified evidence and omission appendix (untrusted JSON data):\n");
    text.push_str(&serde_json::to_string_pretty(b).map_err(|e| e.to_string())?);
    text.push_str(&format!("\nPacket exact-byte SHA-256: {}\n", export.digest));
    if text.len() > max_bytes {
        return Err("complete review document exceeds byte budget".into());
    }
    Ok(text)
}
