//! Native v2 extends v1 explicitly; no imported payload is executable or review authority.
use super::{zip_entry_count, Manifest, Package, MAX_ARCHIVE, MAX_ENTRY, MAX_FILES};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Cursor, Read, Write},
};
use tacitus_core::{
    digest,
    policy::{CognitiveMethod, HumanReview, PolicyAssertion, SourceRecord, ONTOLOGY_VERSION},
    Result, Span,
};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Data {
    pub sources: Vec<SourceRecord>,
    pub assertions: Vec<PolicyAssertion>,
    pub methods: Vec<CognitiveMethod>,
    pub reviews: Vec<HumanReview>,
    pub artifacts: BTreeMap<String, String>,
}
#[derive(Debug, Clone)]
pub struct PolicyPackage {
    pub manifest: Manifest,
    pub data: Data,
    pub evidence: BTreeMap<String, Vec<u8>>,
}
pub fn evidence_name(s: &SourceRecord) -> String {
    format!("evidence/{}.txt", digest(s.key().as_bytes()))
}
impl PolicyPackage {
    pub fn sources(&self) -> Result<BTreeMap<String, SourceRecord>> {
        self.data
            .sources
            .iter()
            .map(|s| {
                let mut hydrated = s.clone();
                hydrated.text = String::from_utf8(
                    self.evidence
                        .get(&evidence_name(s))
                        .ok_or("evidence unavailable; import authorized original")?
                        .clone(),
                )
                .map_err(|_| "invalid UTF-8")?;
                hydrated.validate()?;
                Ok((s.key(), hydrated))
            })
            .collect()
    }
    pub fn validate(&self) -> Result<()> {
        let m = &self.manifest;
        if m.schema_version != 2
            || m.package_version == 0
            || m.id.is_empty()
            || m.author.is_empty()
            || m.title.is_empty()
            || m.scope.is_empty()
            || m.purpose.is_empty()
            || m.license.is_empty()
            || m.sharing_policy.is_empty()
            || !["context", "cognitive_method", "output_artifact"].contains(&m.profile.as_str())
            || !["private", "public", "restricted"].contains(&m.confidentiality.as_str())
            || !["references_only", "evidence_included"].contains(&m.evidence_mode.as_str())
            || m.dependencies.len() > 16
            || self.data.sources.len() > 32
            || self.data.assertions.len() > 256
            || self.data.methods.len() > 32
            || self.data.reviews.len() > 256
        {
            return Err("Unsupported/incomplete native v2 capsule".into());
        }
        if m.evidence_mode == "references_only"
            && (!self.evidence.is_empty() || m.limitations.is_empty())
        {
            return Err(
                "References-only export needs explicit missing-evidence limitations".into(),
            );
        }
        let mut sources = BTreeMap::new();
        let mut allowed = BTreeSet::new();
        for s in &self.data.sources {
            if !s.text.is_empty()
                || s.workspace != m.scope
                || s.withdrawn
                || s.restricted
                || s.version == 0
                || s.id.is_empty()
                || s.text_hash.len() != 64
                || !s.text_hash.bytes().all(|b| b.is_ascii_hexdigit())
                || sources.insert(s.key(), s.clone()).is_some()
            {
                return Err("Source export scope/identity/hash rejected".into());
            }
            let name = evidence_name(s);
            allowed.insert(name.clone());
            if let Some(bytes) = self.evidence.get(&name) {
                if !s.redistributable
                    || s.license.as_ref().is_none_or(|v| v.trim().is_empty())
                    || digest(bytes) != s.text_hash
                {
                    return Err("Evidence rights/hash rejected".into());
                }
                let mut h = s.clone();
                h.text = String::from_utf8(bytes.clone()).map_err(|_| "UTF-8 required")?;
                h.validate()?;
            } else if m.evidence_mode == "evidence_included" {
                return Err("Included evidence missing".into());
            }
        }
        if self
            .evidence
            .iter()
            .any(|(n, b)| !allowed.contains(n) || b.len() > MAX_ENTRY)
        {
            return Err("Unexpected/oversized evidence payload".into());
        }
        let mut ids = BTreeSet::new();
        for a in &self.data.assertions {
            if a.workspace != m.scope || !ids.insert(a.id.clone()) || a.evidence.is_empty() {
                return Err("Invalid/duplicate assertion scope".into());
            }
            self.check_spans(&a.evidence, &sources)?;
        }
        let mut methods = BTreeSet::new();
        for method in &self.data.methods {
            if method.workspace != m.scope
                || !methods.insert(method.id.clone())
                || method.description.trim().is_empty()
                || method.failure_conditions.is_empty()
                || method.prerequisites.is_empty()
                || method.ontology_version != ONTOLOGY_VERSION
                || method.evidence.is_empty()
            {
                return Err("Invalid declarative method".into());
            }
            self.check_spans(&method.evidence, &sources)?;
        }
        for d in &m.dependencies {
            if d.id.is_empty()
                || d.version == 0
                || d.digest.len() != 64
                || !d.digest.bytes().all(|b| b.is_ascii_hexdigit())
            {
                return Err("Versioned dependency digest required".into());
            }
        }
        for review in &self.data.reviews {
            let target = match review.target_kind.as_str() {
                "assertion" => ids.contains(&review.target_id),
                "method" => methods.contains(&review.target_id),
                _ => false,
            };
            if !target
                || review.id.is_empty()
                || !["accept", "reject", "caveat"].contains(&review.decision.as_str())
                || review.target_fingerprint.len() != 64
                || !review
                    .target_fingerprint
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit())
                || review
                    .evidence_hashes
                    .iter()
                    .any(|(key, hash)| sources.get(key).is_none_or(|s| s.text_hash != *hash))
            {
                return Err(
                    "Review metadata refers to an unexported target/evidence version".into(),
                );
            }
        }
        if m.evidence_mode == "evidence_included" {
            let hydrated = self.sources()?;
            for a in &self.data.assertions {
                a.validate(&hydrated)?;
            }
            for method in &self.data.methods {
                method.validate(&hydrated)?;
            }
        }
        Ok(())
    }
    fn check_spans(&self, spans: &[Span], sources: &BTreeMap<String, SourceRecord>) -> Result<()> {
        for span in spans {
            let s = sources
                .get(&span.source)
                .ok_or("Unexported citation source")?;
            if span.text_hash != s.text_hash
                || span.original_language != s.language
                || span.start >= span.end
                || span.end > MAX_ENTRY
            {
                return Err("Invalid citation identity/bounds".into());
            }
            if let Some(bytes) = self.evidence.get(&evidence_name(s)) {
                let mut h = s.clone();
                h.text = String::from_utf8(bytes.clone()).map_err(|_| "UTF-8 required")?;
                h.passage(span)?;
            }
        }
        Ok(())
    }
    pub fn preview(&self) -> Result<BTreeMap<String, Vec<u8>>> {
        self.validate()?;
        let mut files = self.evidence.clone();
        files.insert(
            "capsule.json".into(),
            serde_json::to_vec(&self.data).map_err(|e| e.to_string())?,
        );
        let mut m = self.manifest.clone();
        m.files = files.iter().map(|(n, b)| (n.clone(), digest(b))).collect();
        files.insert(
            "manifest.json".into(),
            serde_json::to_vec(&m).map_err(|e| e.to_string())?,
        );
        if files.len() > MAX_FILES
            || files.values().any(|b| b.len() > MAX_ENTRY)
            || files.values().map(Vec::len).sum::<usize>() > MAX_ARCHIVE
        {
            return Err("Capsule resource bound exceeded".into());
        }
        Ok(files)
    }
    pub fn export(&self) -> Result<Vec<u8>> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        for (name, bytes) in self.preview()? {
            writer
                .start_file(
                    name,
                    SimpleFileOptions::default()
                        .compression_method(CompressionMethod::Stored)
                        .unix_permissions(0o600),
                )
                .map_err(|e| e.to_string())?;
            writer.write_all(&bytes).map_err(|e| e.to_string())?;
        }
        let bytes = writer.finish().map_err(|e| e.to_string())?.into_inner();
        if bytes.len() > MAX_ARCHIVE {
            return Err("Archive too large".into());
        }
        Ok(bytes)
    }
    pub fn import(bytes: &[u8]) -> Result<Self> {
        Self::import_inner(bytes, false)
    }
    /// Opt-in preflight before any JSON normalization; legacy import stays compatible.
    pub fn import_strict(bytes: &[u8]) -> Result<Self> {
        Self::import_inner(bytes, true)
    }
    fn import_inner(bytes: &[u8], strict: bool) -> Result<Self> {
        if bytes.len() > MAX_ARCHIVE {
            return Err("Archive too large".into());
        }
        let count = zip_entry_count(bytes)?;
        let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
        if count != archive.len() || archive.offset() != 0 {
            return Err("Duplicate/prefixed archive rejected".into());
        }
        let mut files = BTreeMap::new();
        let mut total = 0;
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index).map_err(|e| e.to_string())?;
            let name = entry.name().to_string();
            let mode = entry.unix_mode().unwrap_or(0o100600) & 0o170000;
            if !super::safe_name(&name)
                || entry.is_dir()
                || !matches!(mode, 0 | 0o100000)
                || entry.size() > MAX_ENTRY as u64
                || files.contains_key(&name)
            {
                return Err("Unsafe capsule file".into());
            }
            let mut body = Vec::new();
            entry
                .by_ref()
                .take((MAX_ENTRY + 1) as u64)
                .read_to_end(&mut body)
                .map_err(|e| e.to_string())?;
            total += body.len();
            if total > MAX_ARCHIVE || body.len() > MAX_ENTRY {
                return Err("Decompression budget exceeded".into());
            }
            files.insert(name, body);
        }
        if strict {
            super::strict_json::validate(files.get("manifest.json").ok_or("Manifest missing")?)?;
            super::strict_json::validate(files.get("capsule.json").ok_or("Data missing")?)?;
        }
        let manifest: Manifest =
            serde_json::from_slice(files.get("manifest.json").ok_or("Manifest missing")?)
                .map_err(|e| e.to_string())?;
        if strict && manifest.schema_version != 2 {
            return Err("strict import requires native-v2; no implicit migration".into());
        }
        if manifest.schema_version == 1 {
            return Self::migrate_v1(Package::import(bytes)?);
        }
        let mut data: Data =
            serde_json::from_slice(files.get("capsule.json").ok_or("Data missing")?)
                .map_err(|e| e.to_string())?;
        files.remove("manifest.json");
        if files.len() != manifest.files.len()
            || files
                .iter()
                .any(|(n, b)| manifest.files.get(n) != Some(&digest(b)))
        {
            return Err("Capsule integrity failed".into());
        }
        files.remove("capsule.json");
        for review in &mut data.reviews {
            review.origin = "imported_attestation".into();
        }
        let package = Self {
            manifest,
            data,
            evidence: files,
        };
        package.validate()?;
        Ok(package)
    }
    fn migrate_v1(old: Package) -> Result<Self> {
        let original_data = serde_json::to_string(&old.data).map_err(|e| e.to_string())?;
        let mut manifest = old.manifest;
        manifest.schema_version = 2;
        manifest.limitations.push("Migrated from native v1: legacy methods lack explicit prerequisites; scope must be confirmed. All reviews are untrusted.".into());
        let sources: Vec<_> = old
            .data
            .sources
            .into_iter()
            .map(|s| SourceRecord {
                id: s.id,
                version: s.version,
                workspace: s.workspace,
                document_id: None,
                title: s.title,
                author: Some(s.author),
                original_hash: Some(s.original_hash),
                text_hash: s.text_hash,
                text: String::new(),
                language: s.language,
                extractor: s.extraction_tool,
                published_at: Some(s.published_at),
                retrieved_at: s.retrieved_at,
                available_at: s.available.start,
                valid_from: Some(s.valid.start),
                valid_until: s.valid.end,
                source_family: s.source_family,
                withdrawn: s.withdrawn,
                restricted: s.restricted,
                redistributable: s.redistributable,
                license: Some(s.license),
            })
            .collect();
        let assertions: Vec<_> = old
            .data
            .assertions
            .into_iter()
            .map(|a| PolicyAssertion {
                id: a.id,
                version: a.version,
                workspace: a.workspace,
                speaker: a.speaker,
                subject: a.subject,
                predicate: "states".into(),
                object: a.object,
                proposition: a.proposition,
                epistemic_type: a.epistemic_type,
                modality: a.modality,
                conditions: a.conditions,
                exceptions: a.exceptions,
                evidence: a.evidence,
                created_at: a.available.start,
            })
            .collect();
        let original_scope = manifest.scope.clone();
        if let Some(source) = sources.first() {
            manifest.scope = source.workspace.clone();
        }
        let mut methods = Vec::new();
        for m in old.data.methods {
            let mut spans = Vec::new();
            for key in &m.provenance {
                let s = sources
                    .iter()
                    .find(|s| s.key() == *key)
                    .ok_or("Legacy method provenance absent")?;
                let text=old.evidence.get(&evidence_name(s)).ok_or("Legacy references-only method cannot migrate exact provenance; import sources and review manually")?;
                spans.push(Span {
                    source: key.clone(),
                    text_hash: s.text_hash.clone(),
                    start: 0,
                    end: text.len(),
                    locator: None,
                    original_language: s.language.clone(),
                    translation: None,
                });
            }
            methods.push(CognitiveMethod {
                id: m.id,
                version: m.version,
                workspace: manifest.scope.clone(),
                kind: m.kind,
                description: m.description,
                scope: m.scope,
                prerequisites: vec!["scope_confirmed".into()],
                failure_conditions: m.failure_conditions,
                evidence: spans,
                created_at: manifest.created_at,
                available_at: manifest.created_at,
                ontology_version: ONTOLOGY_VERSION.into(),
            });
        }
        let mut artifacts = old.data.artifacts;
        artifacts.insert("v1-original-records-untrusted".into(), original_data);
        manifest.limitations.push("V1 descriptive profile, polarity, audience, speech-act, detailed temporal/dependency/confidence fields are retained in original-records artifacts; v2 assertions are conservative unreviewed projections, not a fidelity-equivalent replacement.".into());
        artifacts.insert("v1-original-scope".into(), original_scope);
        artifacts.insert(
            "v1-review-attestations".into(),
            serde_json::to_string(&old.data.reviews).map_err(|e| e.to_string())?,
        );
        let migrated = Self {
            manifest,
            data: Data {
                sources,
                assertions,
                methods,
                reviews: vec![],
                artifacts,
            },
            evidence: old.evidence,
        };
        migrated.validate()?;
        Ok(migrated)
    }
}
