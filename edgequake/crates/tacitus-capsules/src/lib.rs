//! Portable data packages and a constrained evidence compiler. No imported code executes.
pub mod compiler;
pub mod policy;
pub mod strict_json;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{Cursor, Read, Write};
use tacitus_core::{digest, Assertion, Result, ReviewOrigin, ReviewReceipt, SourceVersion};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

const MAX_ARCHIVE: usize = 8 * 1024 * 1024;
const MAX_ENTRY: usize = 2 * 1024 * 1024;
const MAX_FILES: usize = 64;

// zip 4.6 indexes names in a map, silently collapsing duplicate central entries.
// Check the bounded ZIP32 envelope first and require its raw count to survive parsing.
// v1 deliberately rejects multi-disk, ZIP64 and archive comments/trailers.
fn zip_entry_count(bytes: &[u8]) -> Result<usize> {
    let end = bytes
        .len()
        .checked_sub(22)
        .ok_or("truncated ZIP envelope")?;
    let record = &bytes[end..];
    if &record[..4] != b"PK\x05\x06" || record[4..8] != [0; 4] || record[20..22] != [0; 2] {
        return Err("unsupported ZIP envelope: ZIP32, single disk, no comments required".into());
    }
    let disk_count = u16::from_le_bytes([record[8], record[9]]) as usize;
    let count = u16::from_le_bytes([record[10], record[11]]) as usize;
    let size = u32::from_le_bytes(record[12..16].try_into().map_err(|_| "ZIP size")?) as usize;
    let offset = u32::from_le_bytes(record[16..20].try_into().map_err(|_| "ZIP offset")?) as usize;
    if count != disk_count || count > MAX_FILES || offset.checked_add(size) != Some(end) {
        return Err("ZIP entry/directory resource bounds invalid".into());
    }
    Ok(count)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dependency {
    pub id: String,
    pub version: u32,
    pub digest: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Method {
    pub id: String,
    pub version: u32,
    pub kind: String,
    pub description: String,
    pub scope: String,
    pub failure_conditions: Vec<String>,
    pub provenance: Vec<String>,
    pub review_state: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapsuleData {
    pub assertions: Vec<Assertion>,
    /// Metadata only: `text` must be empty. Evidence is an explicit optional file.
    pub sources: Vec<SourceVersion>,
    pub methods: Vec<Method>,
    pub reviews: Vec<ReviewReceipt>,
    pub artifacts: BTreeMap<String, String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema_version: u32,
    pub package_version: u32,
    pub id: String,
    pub author: String,
    pub title: String,
    pub domain: String,
    pub scope: String,
    pub purpose: String,
    pub profile: String,
    pub created_at: i64,
    pub valid_until: Option<i64>,
    pub supersedes: Option<String>,
    pub dependencies: Vec<Dependency>,
    pub license: String,
    pub confidentiality: String,
    pub sharing_policy: String,
    pub evidence_mode: String,
    pub limitations: Vec<String>,
    pub files: BTreeMap<String, String>,
}
#[derive(Debug, Clone)]
pub struct Package {
    pub manifest: Manifest,
    pub data: CapsuleData,
    pub evidence: BTreeMap<String, Vec<u8>>,
}

fn safe_name(name: &str) -> bool {
    matches!(name, "manifest.json" | "capsule.json") || {
        let Some(rest) = name.strip_prefix("evidence/") else {
            return false;
        };
        !rest.is_empty()
            && rest
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
            && !rest.contains("..")
            && [".txt", ".md", ".pdf", ".html"]
                .iter()
                .any(|ext| rest.ends_with(ext))
    }
}
fn json<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    serde_json::to_vec(value).map_err(|e| e.to_string())
}
pub fn evidence_name(source: &SourceVersion) -> String {
    format!("evidence/{}.txt", digest(source.key().as_bytes()))
}

impl Package {
    /// Export preview is the exact list of serialized fields and files.
    pub fn preview(&self) -> Result<BTreeMap<String, Vec<u8>>> {
        self.validate()?;
        let mut files = self.evidence.clone();
        files.insert("capsule.json".into(), json(&self.data)?);
        let mut manifest = self.manifest.clone();
        manifest.files = files
            .iter()
            .map(|(name, bytes)| (name.clone(), digest(bytes)))
            .collect();
        files.insert("manifest.json".into(), json(&manifest)?);
        Ok(files)
    }
    pub fn validate(&self) -> Result<()> {
        let m = &self.manifest;
        if m.schema_version != 1
            || m.package_version == 0
            || m.id.is_empty()
            || m.author.is_empty()
            || m.title.is_empty()
            || m.scope.is_empty()
            || m.purpose.is_empty()
            || m.domain.is_empty()
            || m.license.is_empty()
            || m.sharing_policy.is_empty()
            || !["context", "cognitive_method", "output_artifact"].contains(&m.profile.as_str())
            || !["public", "private", "restricted"].contains(&m.confidentiality.as_str())
            || !["references_only", "evidence_included"].contains(&m.evidence_mode.as_str())
            || m.valid_until.is_some_and(|end| end <= m.created_at)
            || m.dependencies.len() > 16
            || self.data.sources.len() > 32
            || self.data.assertions.len() > 256
            || self.data.methods.len() > 32
        {
            return Err("unsupported or incomplete capsule manifest/resource limits".into());
        }
        for dependency in &m.dependencies {
            if dependency.id.is_empty()
                || dependency.version == 0
                || dependency.digest.len() != 64
                || !dependency.digest.bytes().all(|b| b.is_ascii_hexdigit())
            {
                return Err("dependencies must pin identity/version/SHA-256".into());
            }
        }
        if m.evidence_mode == "references_only"
            && (!self.evidence.is_empty() || m.limitations.is_empty())
        {
            return Err(
                "references-only needs explicit unavailable-evidence limitations and no evidence"
                    .into(),
            );
        }
        let mut keys = BTreeSet::new();
        let mut allowed_files = BTreeSet::new();
        for source in &self.data.sources {
            if !source.text.is_empty()
                || source.version == 0
                || source.id.is_empty()
                || source.text_hash.len() != 64
                || source.original_hash.len() != 64
                || !source.text_hash.bytes().all(|b| b.is_ascii_hexdigit())
                || !source.original_hash.bytes().all(|b| b.is_ascii_hexdigit())
                || source.withdrawn
                || source.restricted
                || !keys.insert(source.key())
            {
                return Err("invalid, duplicate, withdrawn or restricted capsule source".into());
            }
            source.available.validate()?;
            source.valid.validate()?;
            let name = evidence_name(source);
            allowed_files.insert(name.clone());
            if let Some(bytes) = self.evidence.get(&name) {
                if !source.redistributable
                    || source.license.is_empty()
                    || digest(bytes) != source.text_hash
                {
                    return Err("evidence disclosure/rights/hash check failed".into());
                }
                let mut hydrated = source.clone();
                hydrated.text =
                    String::from_utf8(bytes.clone()).map_err(|_| "invalid UTF-8 evidence")?;
                hydrated.validate()?;
            } else if m.evidence_mode == "evidence_included" {
                return Err("included-evidence source file missing".into());
            }
        }
        let mut assertion_ids = BTreeSet::new();
        for assertion in &self.data.assertions {
            if assertion.id.is_empty()
                || assertion.version == 0
                || assertion.proposition.trim().is_empty()
                || assertion.speaker.is_empty()
                || assertion.confidence_rationale.is_empty()
                || assertion.evidence.is_empty()
                || !assertion_ids.insert(assertion.id.clone())
            {
                return Err("invalid or duplicate capsule assertion".into());
            }
            assertion.valid.validate()?;
            assertion.available.validate()?;
            for span in &assertion.evidence {
                let source = self
                    .data
                    .sources
                    .iter()
                    .find(|s| s.key() == span.source)
                    .ok_or("citation source absent")?;
                if span.text_hash != source.text_hash
                    || assertion.workspace != source.workspace
                    || span.start >= span.end
                    || span.original_language != source.language
                {
                    return Err("invalid reference span".into());
                }
                if let Some(bytes) = self.evidence.get(&evidence_name(source)) {
                    let mut hydrated = source.clone();
                    hydrated.text =
                        String::from_utf8(bytes.clone()).map_err(|_| "invalid evidence")?;
                    span.passage(&hydrated)?;
                }
            }
        }
        let kinds = [
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
        ];
        let mut method_ids = BTreeSet::new();
        for method in &self.data.methods {
            if !kinds.contains(&method.kind.as_str())
                || method.version == 0
                || method.id.is_empty()
                || method.scope.is_empty()
                || method.description.trim().is_empty()
                || !method_ids.insert(method.id.clone())
                || method.failure_conditions.is_empty()
                || method
                    .failure_conditions
                    .iter()
                    .any(|s| s.trim().is_empty())
                || method.provenance.is_empty()
                || method.provenance.iter().any(|key| !keys.contains(key))
            {
                return Err("invalid declarative cognitive method".into());
            }
        }
        if self.evidence.iter().any(|(name, bytes)| {
            !safe_name(name) || !allowed_files.contains(name) || bytes.len() > MAX_ENTRY
        }) {
            return Err("unexpected or oversized evidence file".into());
        }
        Ok(())
    }
    pub fn export(&self) -> Result<Vec<u8>> {
        let files = self.preview()?;
        if files.len() > MAX_FILES
            || files.values().any(|b| b.len() > MAX_ENTRY)
            || files.values().map(Vec::len).sum::<usize>() > MAX_ARCHIVE
        {
            return Err("export exceeds package resource limits".into());
        }
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Stored)
            .unix_permissions(0o600);
        for (name, bytes) in files {
            writer
                .start_file(name, options)
                .map_err(|e| e.to_string())?;
            writer.write_all(&bytes).map_err(|e| e.to_string())?;
        }
        let bytes = writer.finish().map_err(|e| e.to_string())?.into_inner();
        if bytes.len() > MAX_ARCHIVE {
            return Err("archive too large".into());
        }
        Ok(bytes)
    }
    /// Parse in memory, never extract paths; never fetch dependencies or run payloads.
    pub fn import(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_ARCHIVE {
            return Err("archive too large".into());
        }
        let raw_count = zip_entry_count(bytes)?;
        let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
        if raw_count != archive.len() || archive.offset() != 0 {
            return Err("duplicate central-directory names or prefixed archive rejected".into());
        }
        if archive.len() > MAX_FILES {
            return Err("too many archive entries".into());
        }
        let mut files = BTreeMap::new();
        let mut total = 0usize;
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
            let name = entry.name().to_string();
            let mode = entry.unix_mode().unwrap_or(0o100600) & 0o170000;
            if !safe_name(&name)
                || entry.is_dir()
                || !matches!(mode, 0 | 0o100000)
                || entry.size() > MAX_ENTRY as u64
                || files.contains_key(&name)
            {
                return Err("unsafe/duplicate/oversized archive entry".into());
            }
            let mut content = Vec::new();
            entry
                .by_ref()
                .take((MAX_ENTRY + 1) as u64)
                .read_to_end(&mut content)
                .map_err(|e| e.to_string())?;
            total = total.checked_add(content.len()).ok_or("size overflow")?;
            if content.len() > MAX_ENTRY || total > MAX_ARCHIVE {
                return Err("decompression budget exceeded".into());
            }
            files.insert(name, content);
        }
        let manifest: Manifest =
            serde_json::from_slice(&files.remove("manifest.json").ok_or("missing manifest")?)
                .map_err(|e| e.to_string())?;
        if manifest.files.len() != files.len()
            || files
                .iter()
                .any(|(name, bytes)| manifest.files.get(name) != Some(&digest(bytes)))
        {
            return Err("manifest file hashes mismatch".into());
        }
        let mut data: CapsuleData =
            serde_json::from_slice(&files.remove("capsule.json").ok_or("missing data")?)
                .map_err(|e| e.to_string())?;
        for review in &mut data.reviews {
            review.origin = ReviewOrigin::ImportedAttestation;
        }
        // A foreign label grants no local review authority, even on an intact bundle.
        for method in &mut data.methods {
            method.review_state = "unreviewed_proposal".into();
        }
        let package = Self {
            manifest,
            data,
            evidence: files,
        };
        package.validate()?;
        Ok(package)
    }
    pub fn proposals_for_workspace(&self, workspace: &str) -> Result<Vec<Assertion>> {
        let sources = self.sources_for_workspace(workspace)?;
        self.data
            .assertions
            .iter()
            .map(|original| {
                let mut proposal = original.clone();
                proposal.workspace = workspace.into();
                proposal.validate(&sources)?;
                Ok(proposal)
            })
            .collect()
    }
    /// Explicit new-workspace reuse. References-only cannot fabricate evidence.
    pub fn sources_for_workspace(
        &self,
        workspace: &str,
    ) -> Result<BTreeMap<String, SourceVersion>> {
        self.validate()?;
        if workspace.is_empty() {
            return Err("workspace required".into());
        }
        self.data
            .sources
            .iter()
            .map(|metadata| {
                let mut source = metadata.clone();
                source.workspace = workspace.into();
                source.text = String::from_utf8(
                    self.evidence
                        .get(&evidence_name(metadata))
                        .ok_or("evidence unavailable; supply authorized original")?
                        .clone(),
                )
                .map_err(|_| "invalid text")?;
                source.validate()?;
                Ok((source.key(), source))
            })
            .collect()
    }
}
