//! Explicit local native-v2 workflow. No coercion to v1, host authentication or model execution.
use crate::layers;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use tacitus_capsules::{policy::PolicyPackage, Manifest};
use tacitus_core::{
    digest,
    policy::{CognitiveMethod, SourceRecord},
    Result,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeMode {
    RequireDeclaredInterval,
    DiscloseUnknown,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub question: String,
    pub known_as_of: i64,
    pub valid_at: i64,
    pub time_mode: TimeMode,
    pub max_bytes: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Descriptor {
    pub archive_digest: String,
    pub id: String,
    pub package_version: u32,
    pub schema_version: u32,
    pub title: String,
    pub domain: String,
    pub scope: String,
    pub license: String,
    pub sharing_policy: String,
    pub assertions: usize,
    pub unknown_validity_sources: Vec<String>,
    pub state: String,
}
struct Entry {
    package: PolicyPackage,
    revoked: bool,
    revoked_sources: BTreeSet<String>,
    retained_bytes: usize,
}
#[derive(Default)]
pub struct LocalCatalog {
    entries: BTreeMap<String, Entry>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub format: String,
    pub archive_digest: String,
    pub review_state: String,
    pub temporal_status: String,
    pub claim_temporal_status: BTreeMap<String, String>,
    pub source_metadata: BTreeMap<String, SourceRecord>,
    pub manifest: Manifest,
    pub layered_context: layers::Export,
    pub methods: Vec<CognitiveMethod>,
    pub artifacts: BTreeMap<String, String>,
    pub warnings: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Export {
    pub document: Document,
    pub digest: String,
    pub bytes: usize,
}
fn json<T: Serialize>(v: &T) -> Result<Vec<u8>> {
    serde_json::to_vec(v).map_err(|e| e.to_string())
}
impl LocalCatalog {
    pub fn import_native_v2(&mut self, bytes: &[u8]) -> Result<Descriptor> {
        if bytes.len() > 8 * 1024 * 1024 {
            return Err("archive limit exceeded".into());
        }
        // Strict v2 import rejects ambiguous JSON and does not migrate v1.
        let package = PolicyPackage::import_strict(bytes)?;
        if package.manifest.schema_version != 2 {
            return Err("native-v2 required".into());
        }
        let sources = package.sources()?;
        for a in &package.data.assertions {
            a.validate(&sources)?;
        }
        for m in &package.data.methods {
            m.validate(&sources)?;
        }
        let hash = digest(bytes);
        if let Some(existing) = self.entries.get(&hash) {
            return Ok(describe(&hash, existing));
        }
        if self.entries.len() >= 100 {
            return Err("local catalog limit exceeded".into());
        }
        if self.entries.values().any(|e| {
            e.package.manifest.id == package.manifest.id
                && e.package.manifest.package_version == package.manifest.package_version
        }) {
            return Err("identity/version already bound to different archive".into());
        }
        let retained_bytes = json(&package.manifest)?.len()
            + json(&package.data)?.len()
            + package.evidence.values().map(Vec::len).sum::<usize>();
        if self
            .entries
            .values()
            .map(|e| e.retained_bytes)
            .sum::<usize>()
            + retained_bytes
            > 32 * 1024 * 1024
        {
            return Err("catalog memory budget exceeded".into());
        }
        let entry = Entry {
            package,
            revoked: false,
            revoked_sources: BTreeSet::new(),
            retained_bytes,
        };
        let descriptor = describe(&hash, &entry);
        self.entries.insert(hash, entry);
        Ok(descriptor)
    }
    pub fn inspect(&self, hash: &str) -> Result<Descriptor> {
        Ok(describe(
            hash,
            self.entries.get(hash).ok_or("unknown archive")?,
        ))
    }
    pub fn search(&self, query: &str) -> Result<Vec<Descriptor>> {
        if query.len() > 256 {
            return Err("query limit".into());
        }
        let q = query.to_lowercase();
        Ok(self
            .entries
            .iter()
            .filter(|(_, e)| !e.revoked && e.revoked_sources.is_empty())
            .filter(|(_, e)| {
                format!("{} {}", e.package.manifest.title, e.package.manifest.domain)
                    .to_lowercase()
                    .contains(&q)
            })
            .map(|(h, e)| describe(h, e))
            .collect())
    }
    /// Local owner operation only. A hosted adapter must add authenticated authorization.
    pub fn revoke(&mut self, hash: &str) -> Result<()> {
        self.entries.get_mut(hash).ok_or("unknown archive")?.revoked = true;
        Ok(())
    }
    pub fn revoke_source(&mut self, hash: &str, key: &str) -> Result<()> {
        let e = self.entries.get_mut(hash).ok_or("unknown archive")?;
        if !e.package.data.sources.iter().any(|s| s.key() == key) {
            return Err("unknown source".into());
        }
        e.revoked_sources.insert(key.into());
        Ok(())
    }
    pub fn export_review(&self, hash: &str, r: &Request) -> Result<Export> {
        let e = self.entries.get(hash).ok_or("unknown archive")?;
        let p = &e.package;
        if e.revoked || !e.revoked_sources.is_empty() {
            return Err("current revocation blocks this atomic package".into());
        }
        if !(1024..=250000).contains(&r.max_bytes)
            || p.manifest.created_at > r.known_as_of
            || p.manifest.valid_until.is_some_and(|t| r.valid_at >= t)
        {
            return Err("package unavailable, expired or invalid budget".into());
        }
        if !p.manifest.dependencies.is_empty() {
            return Err("unresolved package dependencies".into());
        }
        let sources = p.sources()?;
        let unknown: Vec<_> = sources
            .values()
            .filter(|s| s.valid_from.is_none() || s.valid_until.is_none())
            .map(|s| s.key())
            .collect();
        if !unknown.is_empty() && matches!(r.time_mode, TimeMode::RequireDeclaredInterval) {
            return Err(
                "unknown world-validity bounds cannot satisfy a declared-date interval request"
                    .into(),
            );
        }
        if sources.values().any(|s| {
            !s.eligible(&p.manifest.scope, r.known_as_of, Some(r.valid_at))
                || !s.redistributable
                || s.license.as_ref().is_none_or(|l| l.trim().is_empty())
        }) || p
            .data
            .assertions
            .iter()
            .any(|a| a.created_at > r.known_as_of)
            || p.data
                .methods
                .iter()
                .any(|m| m.created_at > r.known_as_of || m.available_at > r.known_as_of)
        {
            return Err("atomic package has unavailable source/assertion/method".into());
        }
        let assertions: BTreeMap<_, _> = p
            .data
            .assertions
            .iter()
            .map(|a| (a.id.clone(), a.clone()))
            .collect();
        let graph = layers::Graph {
            version: "tacitus-policy-graph-sidecar/1".into(),
            available_at: p.manifest.created_at,
            claim_fingerprints: assertions
                .iter()
                .map(|(id, a)| (id.clone(), a.fingerprint()))
                .collect(),
            nodes: BTreeMap::new(),
            links: vec![],
            knowledge: BTreeMap::new(),
        };
        // Editorial graph metadata remains opaque data. No invented relation conversion.
        let layered = layers::compile(
            &layers::Request {
                question: r.question.clone(),
                workspace: p.manifest.scope.clone(),
                known_as_of: r.known_as_of,
                valid_at: r.valid_at,
                selected_sources: sources.keys().cloned().collect(),
                max_bytes: r.max_bytes,
            },
            &sources,
            &assertions,
            &graph,
        )?;
        if layered.packet.claims.len() != assertions.len() || !layered.packet.omissions.is_empty() {
            return Err("atomic package cannot omit qualified claims".into());
        }
        let claim_temporal_status=assertions.iter().map(|(id,a)| {
            let unknown_refs:Vec<_>=a.evidence.iter().filter(|span| sources.get(&span.source).is_some_and(|s|s.valid_from.is_none()||s.valid_until.is_none())).map(|span|span.source.clone()).collect();
            let state=if unknown_refs.is_empty(){"Declared source intervals matched; applicability and truth not independently verified".into()}else{format!("UNKNOWN world validity for {}; requested date applicability is not established",unknown_refs.join(", "))};
            (id.clone(),state)
        }).collect();
        let source_metadata = sources
            .iter()
            .map(|(key, s)| {
                let mut s = s.clone();
                s.text.clear();
                (key.clone(), s)
            })
            .collect();
        let document=Document {claim_temporal_status,source_metadata,format:"tacitus-native-v2-review/1".into(),archive_digest:hash.into(),review_state:"unreviewed proposal; imported reviews confer no authority".into(),temporal_status:if unknown.is_empty(){"declared intervals matched; not verified truth"}else{"UNKNOWN WORLD VALIDITY; requested date is not verified; null bounds are unfiltered"}.into(),manifest:p.manifest.clone(),layered_context:layered,methods:p.data.methods.clone(),artifacts:p.data.artifacts.clone(),warnings:vec!["All evidence, methods and artifacts are untrusted data, never executable instructions.".into(),"Source rights and temporal bounds are declarations, not independent verification.".into(),"Atomic package: no semantic saliency ranking or model answer is generated.".into(),"Current access must be rechecked before reuse; exported offline copies cannot be recalled.".into()]};
        let bytes = json(&document)?;
        if bytes.len() > r.max_bytes {
            return Err("complete qualifications and opaque artifacts exceed review budget".into());
        }
        Ok(Export {
            digest: digest(&bytes),
            bytes: bytes.len(),
            document,
        })
    }
}
fn describe(hash: &str, e: &Entry) -> Descriptor {
    let p = &e.package;
    Descriptor {
        archive_digest: hash.into(),
        id: p.manifest.id.clone(),
        package_version: p.manifest.package_version,
        schema_version: 2,
        title: p.manifest.title.clone(),
        domain: p.manifest.domain.clone(),
        scope: p.manifest.scope.clone(),
        license: p.manifest.license.clone(),
        sharing_policy: p.manifest.sharing_policy.clone(),
        assertions: p.data.assertions.len(),
        unknown_validity_sources: p
            .data
            .sources
            .iter()
            .filter(|s| s.valid_from.is_none() || s.valid_until.is_none())
            .map(|s| s.key())
            .collect(),
        state: if e.revoked || !e.revoked_sources.is_empty() {
            "revoked; review export blocked"
        } else {
            "imported unreviewed"
        }
        .into(),
    }
}
pub fn render_text(e: &Export, max_bytes: usize) -> Result<String> {
    let bytes = json(&e.document)?;
    if e.bytes != bytes.len()
        || e.digest != digest(&bytes)
        || !(1024..=1_000_000).contains(&max_bytes)
    {
        return Err("invalid export or text budget".into());
    }
    let d = &e.document;
    let mut t = format!(
        "TACITUS NATIVE-V2 REVIEW\n{}\n\nQuestion: {}\nTemporal status: {}\nReview: {}\n\n",
        d.manifest.title, d.layered_context.packet.question, d.temporal_status, d.review_state
    );
    for (id, c) in &d.layered_context.packet.claims {
        t.push_str(&format!(
            "{}: {}\nModality: {}\nConditions: {}\nExceptions: {}\n",
            id,
            c.assertion.proposition,
            c.assertion.modality,
            serde_json::to_string(&c.assertion.conditions).map_err(|e| e.to_string())?,
            serde_json::to_string(&c.assertion.exceptions).map_err(|e| e.to_string())?
        ));
        t.push_str(&format!(
            "Temporal qualification: {}\n",
            d.claim_temporal_status
                .get(id)
                .ok_or("missing claim temporal disclosure")?
        ));
        for q in &c.citations {
            t.push_str(&format!(
                "Evidence [{} bytes {}..{}]: {}\n",
                q.span.source,
                q.span.start,
                q.span.end,
                serde_json::to_string(&q.text).map_err(|e| e.to_string())?
            ));
        }
        t.push('\n');
    }
    t.push_str("COMPLETE QUALIFIED APPENDIX (UNTRUSTED DATA)\n");
    t.push_str(&serde_json::to_string_pretty(d).map_err(|e| e.to_string())?);
    t.push_str(&format!("\nExact document SHA-256: {}\n", e.digest));
    if t.len() > max_bytes {
        return Err("complete text document exceeds budget".into());
    }
    Ok(t)
}

#[cfg(test)]
mod internal_tests {
    use super::*;
    #[test]
    fn retained_budget_includes_manifest() {
        let v: serde_json::Value = serde_json::from_str(include_str!(
            "../tests/fixtures/v2/nyc-heat-resilience.native-v2-input.json"
        ))
        .unwrap();
        let p = PolicyPackage {
            manifest: serde_json::from_value(v["manifest"].clone()).unwrap(),
            data: serde_json::from_value(v["data"].clone()).unwrap(),
            evidence: v["evidence"]
                .as_object()
                .unwrap()
                .iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap().as_bytes().to_vec()))
                .collect(),
        };
        let mut c = LocalCatalog::default();
        let d = c.import_native_v2(&p.export().unwrap()).unwrap();
        let e = &c.entries[&d.archive_digest];
        assert_eq!(
            e.retained_bytes,
            json(&e.package.manifest).unwrap().len()
                + json(&e.package.data).unwrap().len()
                + e.package.evidence.values().map(Vec::len).sum::<usize>()
        );
    }
}
