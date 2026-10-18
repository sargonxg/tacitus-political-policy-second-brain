//! Explicit archive-bound extractive projection. Imported mappings grant no review authority.
use crate::{layers, v2_workflow as v2};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use tacitus_capsules::policy::PolicyPackage;
use tacitus_core::{digest, policy::SourceRecord, Result};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Global,
    Claim,
    Attribution,
    Supplement,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Item {
    pub artifact: String,
    pub pointer: String,
    pub value_hash: String,
    pub role: Role,
    pub claims: BTreeSet<String>,
    pub reason: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mapping {
    pub profile_version: String,
    pub format: String,
    pub archive_digest: String,
    pub available_at: i64,
    pub artifact_hashes: BTreeMap<String, String>,
    pub claim_fingerprints: BTreeMap<String, String>,
    pub items: Vec<Item>,
    pub graph: layers::Graph,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub review: v2::Request,
    pub priority: Vec<String>,
    pub revoked_sources: BTreeSet<String>,
    pub package_revoked: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemValue {
    pub artifact: String,
    pub pointer: String,
    pub value: Value,
    pub original_value_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exact_text_reference: Option<TextReference>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextReference {
    pub kind: String,
    pub key: String,
    pub text_hash: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStatus {
    SelectedEvidence,
    OmissionOnly,
}
const NO_EVIDENCE_WARNING: &str =
    "NO EVIDENCE SELECTED: this is an omission receipt, not an answer or usable evidence review.";
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Document {
    pub format: String,
    /// Additive field: older documents may omit it; derive from claims in that case.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_status: Option<EvidenceStatus>,
    pub archive_digest: String,
    pub mapping_digest: String,
    pub question: String,
    pub known_as_of: i64,
    pub valid_at: i64,
    pub trust: String,
    pub package_metadata: tacitus_capsules::Manifest,
    pub claims: BTreeMap<String, layers::ClaimLayer>,
    pub sources: BTreeMap<String, SourceRecord>,
    pub temporal: BTreeMap<String, String>,
    pub relationships: Vec<layers::Link>,
    pub items: Vec<ItemValue>,
    pub omissions: BTreeMap<String, String>,
    pub methods_notice: String,
    pub warnings: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Export {
    pub document: Document,
    pub digest: String,
    pub bytes: usize,
}
fn bytes<T: Serialize>(v: &T) -> Result<Vec<u8>> {
    serde_json::to_vec(v).map_err(|e| e.to_string())
}
fn leaves(v: &Value, p: &str, out: &mut Vec<String>) {
    match v {
        Value::Object(m) if !m.is_empty() => {
            for (k, v) in m {
                leaves(
                    v,
                    &format!("{}/{}", p, k.replace('~', "~0").replace('/', "~1")),
                    out,
                )
            }
        }
        Value::Array(a) if !a.is_empty() => {
            for (i, v) in a.iter().enumerate() {
                leaves(v, &format!("{p}/{i}"), out)
            }
        }
        _ => out.push(p.into()),
    }
}
fn contains(prefix: &str, path: &str) -> bool {
    prefix == path || path.starts_with(&format!("{prefix}/"))
}
fn validate_profile_fields(
    value: &Value,
    path: &str,
    fields: &BTreeMap<String, BTreeSet<String>>,
) -> Result<()> {
    match value {
        Value::Object(object) => {
            let allowed = fields
                .get(path)
                .ok_or("unrecognized object path requires explicit profile migration")?;
            for (key, value) in object {
                if !allowed.contains(key) {
                    return Err(
                        "unrecognized artifact field requires explicit profile migration".into(),
                    );
                }
                validate_profile_fields(
                    value,
                    &format!("{}/{}", path, key.replace('~', "~0").replace('/', "~1")),
                    fields,
                )?;
            }
        }
        Value::Array(array) => {
            for value in array {
                validate_profile_fields(value, &format!("{path}/*"), fields)?
            }
        }
        _ => {}
    }
    Ok(())
}
/// Raw untrusted JSON boundary. Typed callers must not pre-normalize ambiguous input.
pub fn compile_json(archive: &[u8], mapping: &[u8], request: &[u8]) -> Result<Export> {
    if mapping.len() > 1_000_000 || request.len() > 65536 {
        return Err("mapping/request input byte limit exceeded".into());
    }
    let m = tacitus_capsules::strict_json::from_slice(mapping)?;
    let r = tacitus_capsules::strict_json::from_slice(request)?;
    compile(archive, &m, &r)
}
/// Local current-state inputs must be supplied by the host, not obtained from an archive.
pub fn compile(archive: &[u8], mapping: &Mapping, request: &Request) -> Result<Export> {
    if request.package_revoked
        || request.review.max_bytes < 1024
        || request.review.max_bytes > 250000
        || mapping.format != "tacitus-scoped-artifacts/1"
        || mapping.profile_version != "tacitus-policy-research-r2/1"
        || mapping.archive_digest != digest(archive)
        || mapping.available_at > request.review.known_as_of
        || mapping.items.len() > 256
        || bytes(mapping)?.len() > 1000000
    {
        return Err("mapping, current state or resource bounds invalid".into());
    }
    let mut catalog = v2::LocalCatalog::default();
    let desc = catalog.import_native_v2(archive)?;
    for key in &request.revoked_sources {
        catalog.revoke_source(&desc.archive_digest, key)?;
    }
    let mut full_request = request.review.clone();
    full_request.max_bytes = 250000;
    // Retain the full workflow's conservative rights, revocation and temporal gates.
    let full = catalog.export_review(&desc.archive_digest, &full_request)?;
    let p = PolicyPackage::import(archive)?;
    let sources = p.sources()?;
    let assertions = p
        .data
        .assertions
        .iter()
        .map(|a| (a.id.clone(), a.clone()))
        .collect::<BTreeMap<_, _>>();
    let fingerprints = assertions
        .iter()
        .map(|(id, a)| (id.clone(), a.fingerprint()))
        .collect::<BTreeMap<_, _>>();
    if mapping.claim_fingerprints != fingerprints
        || mapping.graph.claim_fingerprints != fingerprints
    {
        return Err("stale claim mapping".into());
    }
    let expected_hashes = p
        .data
        .artifacts
        .iter()
        .map(|(k, v)| (k.clone(), digest(v.as_bytes())))
        .collect::<BTreeMap<_, _>>();
    if mapping.artifact_hashes != expected_hashes {
        return Err("unmapped or changed opaque artifact".into());
    }
    let artifacts = p
        .data
        .artifacts
        .iter()
        .map(|(k, v)| {
            Ok((
                k.clone(),
                tacitus_capsules::strict_json::from_slice::<Value>(v.as_bytes())
                    .map_err(|_| "non-JSON opaque artifact requires atomic workflow")?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    let fields: BTreeMap<String, BTreeSet<String>> =
        serde_json::from_str(include_str!("compact_profile_fields.json"))
            .map_err(|e| e.to_string())?;
    for (key, value) in &artifacts {
        validate_profile_fields(value, key, &fields)?;
    }
    for (key, field) in [
        ("tacitus.policy-research", "revision"),
        ("tacitus.research-summary-provenance", "contentRevision"),
        ("tacitus.native-adaptation", "sourceContentRevision"),
    ] {
        if let Some(v) = artifacts.get(key) {
            if v.get(field).and_then(Value::as_str) != Some("0.1.1") {
                return Err(
                    "unsupported research content revision; explicit profile migration required"
                        .into(),
                );
            }
        }
    }
    let allowed = [
        "tacitus.policy-research",
        "tacitus.research-summary-provenance",
        "tacitus.native-adaptation",
    ];
    if artifacts.keys().any(|k| !allowed.contains(&k.as_str())) {
        return Err("unknown opaque artifact requires atomic workflow".into());
    }
    for key in [
        "limitations",
        "rightsNote",
        "contextPolicy",
        "temporalContract",
        "reviewStatus",
        "constraints",
        "updates",
    ] {
        if artifacts.contains_key("tacitus.policy-research")
            && !mapping.items.iter().any(|i| {
                i.artifact == "tacitus.policy-research"
                    && i.pointer == format!("/{key}")
                    && i.role == Role::Global
            })
        {
            return Err("mandatory global qualification missing".into());
        }
    }
    if artifacts.contains_key("tacitus.native-adaptation")
        && !mapping.items.iter().any(|i| {
            i.artifact == "tacitus.native-adaptation"
                && i.pointer.is_empty()
                && i.role == Role::Global
        })
    {
        return Err("native adaptation caveats must remain global".into());
    }
    // The known research profile must bind each complete record to its declared exact claim.
    for (artifact, field, id_field) in [
        ("tacitus.policy-research", "claims", "id"),
        (
            "tacitus.research-summary-provenance",
            "summaryRecords",
            "claimId",
        ),
    ] {
        if let Some(value) = artifacts.get(artifact) {
            let records = value
                .get(field)
                .and_then(Value::as_array)
                .ok_or("missing scoped records")?;
            let mut record_ids = BTreeSet::new();
            for (n, record) in records.iter().enumerate() {
                let id = record
                    .get(id_field)
                    .and_then(Value::as_str)
                    .ok_or("missing record claim ID")?;
                if !record_ids.insert(id.to_string())
                    || !assertions.contains_key(id)
                    || !mapping.items.iter().any(|i| {
                        i.artifact == artifact
                            && i.pointer == format!("/{field}/{n}")
                            && i.role == Role::Claim
                            && i.claims == BTreeSet::from([id.to_string()])
                    })
                {
                    return Err("record scope cannot remove or misbind claim qualifications".into());
                }
                if field == "claims" {
                    for dependency in record
                        .get("analyticalDependencies")
                        .and_then(Value::as_array)
                        .ok_or("missing explicit analytical dependencies")?
                    {
                        let target = dependency.as_str().ok_or("invalid analytical dependency")?;
                        if !mapping.graph.links.iter().any(|l| {
                            l.from == id && l.to == target && l.kind == layers::LinkKind::DependsOn
                        }) {
                            return Err(
                                "analytical dependency missing from closed claim graph".into()
                            );
                        }
                    }
                }
                if field == "summaryRecords" {
                    let sid = record
                        .get("id")
                        .and_then(Value::as_str)
                        .ok_or("summary source ID")?;
                    let version = record
                        .get("version")
                        .and_then(Value::as_u64)
                        .ok_or("summary version")?;
                    let key = format!("{sid}@{version}");
                    let source = sources.get(&key).ok_or("summary source revision missing")?;
                    if record.get("text").and_then(Value::as_str) != Some(source.text.as_str())
                        || !assertions[id].evidence.iter().any(|e| e.source == key)
                    {
                        return Err(
                            "summary does not match exact source bytes and assertion".into()
                        );
                    }
                }
            }
            if record_ids != assertions.keys().cloned().collect() {
                return Err("missing complete per-claim qualification records".into());
            }
        }
    }
    if let Some(research) = artifacts.get("tacitus.policy-research") {
        let records = research["claims"]
            .as_array()
            .ok_or("research claims missing")?;
        let official = research["sources"]
            .as_array()
            .ok_or("official sources missing")?;
        let mut known = BTreeSet::new();
        for (n, source) in official.iter().enumerate() {
            let id = source["id"].as_str().ok_or("official source ID missing")?;
            if !known.insert(id) {
                return Err("duplicate official source ID".into());
            }
            let bound = records
                .iter()
                .filter(|c| {
                    c["sourceIds"]
                        .as_array()
                        .is_some_and(|a| a.iter().any(|s| s.as_str() == Some(id)))
                })
                .map(|c| c["id"].as_str().unwrap().to_string())
                .collect::<BTreeSet<_>>();
            if !bound.is_empty()
                && !mapping.items.iter().any(|i| {
                    i.artifact == "tacitus.policy-research"
                        && i.pointer == format!("/sources/{n}")
                        && i.role == Role::Attribution
                        && i.claims == bound
                })
            {
                return Err("selected source attribution scope missing".into());
            }
        }
        for edge in research["graph"]["edges"]
            .as_array()
            .ok_or("declared graph edges missing")?
        {
            let from = edge["from"].as_str().ok_or("graph edge source missing")?;
            let to = edge["to"].as_str().ok_or("graph edge target missing")?;
            if assertions.contains_key(from) && assertions.contains_key(to) {
                let kind = match edge["type"].as_str() {
                    Some("analytically_depends_on" | "depends_on") => layers::LinkKind::DependsOn,
                    Some("contradicts") => layers::LinkKind::Contradicts,
                    Some("supersedes") => layers::LinkKind::Supersedes,
                    _ => return Err("unknown claim relationship requires atomic workflow".into()),
                };
                if !mapping
                    .graph
                    .links
                    .iter()
                    .any(|l| l.from == from && l.to == to && l.kind == kind)
                {
                    return Err("declared relationship missing from closure graph".into());
                }
            }
        }
        for claim in records {
            for id in claim["sourceIds"]
                .as_array()
                .ok_or("source references missing")?
            {
                if !known.contains(id.as_str().ok_or("source reference invalid")?) {
                    return Err("unresolved official source".into());
                }
            }
        }
    }
    let mut values = Vec::new();
    let mut keys = BTreeSet::new();
    for item in &mapping.items {
        let v = artifacts
            .get(&item.artifact)
            .and_then(|v| v.pointer(&item.pointer))
            .ok_or("unresolved artifact item")?;
        if !keys.insert((item.artifact.clone(), item.pointer.clone()))
            || digest(&bytes(v)?) != item.value_hash
            || item.reason.trim().is_empty()
            || item.reason.len() > 512
            || item.claims.iter().any(|id| !assertions.contains_key(id))
            || (matches!(item.role, Role::Claim | Role::Attribution) != !item.claims.is_empty())
        {
            return Err("invalid item binding, scope or rationale".into());
        }
        values.push(v.clone());
    }
    // Exact exhaustive partition, including empty containers. No hidden unclassified leaves.
    for (key, value) in &artifacts {
        let mut paths = Vec::new();
        leaves(value, "", &mut paths);
        for path in paths {
            if mapping
                .items
                .iter()
                .filter(|i| i.artifact == *key && contains(&i.pointer, &path))
                .count()
                != 1
            {
                return Err("artifact partition incomplete or overlapping".into());
            }
        }
    }
    let layer_request = layers::Request {
        question: request.review.question.clone(),
        workspace: p.manifest.scope.clone(),
        known_as_of: request.review.known_as_of,
        valid_at: request.review.valid_at,
        selected_sources: sources.keys().cloned().collect(),
        max_bytes: 250000,
    };
    let packet = layers::compile(&layer_request, &sources, &assertions, &mapping.graph)?;
    if packet.packet.claims.len() != assertions.len() {
        return Err("graph contains unavailable claims; atomic fallback required".into());
    }
    let unique = request.priority.iter().cloned().collect::<BTreeSet<_>>();
    if unique.len() != request.priority.len() || unique != assertions.keys().cloned().collect() {
        return Err("priority must list each exact claim once".into());
    }
    let mut adjacency = assertions
        .keys()
        .map(|id| (id.clone(), BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    for link in &mapping.graph.links {
        adjacency
            .get_mut(&link.from)
            .unwrap()
            .insert(link.to.clone());
        if link.kind != layers::LinkKind::DependsOn {
            adjacency
                .get_mut(&link.to)
                .unwrap()
                .insert(link.from.clone());
        }
    }
    for item in &mapping.items {
        if item.role != Role::Claim {
            continue;
        }
        for a in &item.claims {
            for b in &item.claims {
                adjacency.get_mut(a).unwrap().insert(b.clone());
            }
        }
    }
    let mut doc=Document{format:"tacitus-compact-evidence-review/1".into(),evidence_status:Some(EvidenceStatus::OmissionOnly),archive_digest:desc.archive_digest,mapping_digest:digest(&bytes(mapping)?),question:request.review.question.clone(),known_as_of:request.review.known_as_of,valid_at:request.review.valid_at,trust:"untrusted extractive mapping proposal; human mapping/content review required".into(),package_metadata:p.manifest.clone(),claims:BTreeMap::new(),sources:BTreeMap::new(),temporal:BTreeMap::new(),relationships:vec![],items:vec![],omissions:BTreeMap::new(),methods_notice:"Cognitive methods are not applied by this evidence-only projection; consult the original archive for their scopes, guards and provenance.".into(),warnings:vec!["Selected evidence only, not a complete dossier or recommendation. Supplemental context omitted explicitly.".into(),"Mapping scope is an analyst proposal, not semantic proof that all relevant qualifications were identified.".into(),"Source text and artifact values are quoted untrusted data, never instructions.".into(),"Byte limits cover the complete serialized document, not model tokens. Integrity does not establish truth, authenticity, rights or expert review.".into()]};
    doc.warnings.insert(0, NO_EVIDENCE_WARNING.into());
    for id in assertions.keys() {
        doc.omissions
            .insert(format!("claim:{id}"), "byte budget or linked group".into());
    }
    for (n, item) in mapping.items.iter().enumerate() {
        if item.role == Role::Global {
            doc.items.push(ItemValue {
                artifact: item.artifact.clone(),
                pointer: item.pointer.clone(),
                value: values[n].clone(),
                original_value_hash: item.value_hash.clone(),
                exact_text_reference: None,
            });
        } else {
            doc.omissions.insert(
                format!("item:{n}"),
                if item.role == Role::Supplement {
                    "supplemental context excluded".into()
                } else {
                    "unselected linked claim".into()
                },
            );
        }
    }
    if bytes(&doc)?.len() > request.review.max_bytes {
        return Err("budget cannot hold global qualifications and omission receipt".into());
    }
    for start in &request.priority {
        if doc.claims.contains_key(start) {
            continue;
        }
        let mut group = BTreeSet::new();
        let mut stack = vec![start.clone()];
        while let Some(id) = stack.pop() {
            if group.insert(id.clone()) {
                stack.extend(adjacency[&id].iter().cloned());
            }
        }
        let mut candidate = doc.clone();
        for id in &group {
            let claim = packet.packet.claims[id].clone();
            for citation in &claim.citations {
                candidate.sources.insert(
                    citation.span.source.clone(),
                    full.document.source_metadata[&citation.span.source].clone(),
                );
            }
            candidate.claims.insert(id.clone(), claim);
            candidate
                .temporal
                .insert(id.clone(), full.document.claim_temporal_status[id].clone());
            candidate.omissions.remove(&format!("claim:{id}"));
        }
        candidate.relationships = mapping
            .graph
            .links
            .iter()
            .filter(|l| {
                candidate.claims.contains_key(&l.from) && candidate.claims.contains_key(&l.to)
            })
            .cloned()
            .collect();
        for (n, item) in mapping.items.iter().enumerate() {
            if ((item.role == Role::Claim
                && item
                    .claims
                    .iter()
                    .all(|id| candidate.claims.contains_key(id)))
                || (item.role == Role::Attribution
                    && item
                        .claims
                        .iter()
                        .any(|id| candidate.claims.contains_key(id))))
                && candidate.omissions.remove(&format!("item:{n}")).is_some()
            {
                candidate
                    .items
                    .push(project_item(item, &values[n], &candidate.claims));
            }
        }
        candidate.evidence_status = Some(EvidenceStatus::SelectedEvidence);
        candidate
            .warnings
            .retain(|warning| warning != NO_EVIDENCE_WARNING);
        if bytes(&candidate)?.len() <= request.review.max_bytes {
            doc = candidate;
        }
    }
    for item in &doc.items {
        reconstruct_item(item, &doc.claims)?;
    }
    let encoded = bytes(&doc)?;
    Ok(Export {
        bytes: encoded.len(),
        digest: digest(&encoded),
        document: doc,
    })
}
fn project_item(
    item: &Item,
    value: &Value,
    claims: &BTreeMap<String, layers::ClaimLayer>,
) -> ItemValue {
    let mut out = ItemValue {
        artifact: item.artifact.clone(),
        pointer: item.pointer.clone(),
        value: value.clone(),
        original_value_hash: item.value_hash.clone(),
        exact_text_reference: None,
    };
    if item.role != Role::Claim {
        return out;
    }
    if let Some(text) = value.get("text").and_then(Value::as_str) {
        if item.artifact == "tacitus.policy-research" {
            if let Some((id, _)) = claims
                .iter()
                .find(|(id, c)| item.claims.contains(*id) && c.assertion.proposition == text)
            {
                out.exact_text_reference = Some(TextReference {
                    kind: "assertion_proposition/1".into(),
                    key: id.clone(),
                    text_hash: digest(text.as_bytes()),
                });
            }
        }
        if item.artifact == "tacitus.research-summary-provenance" {
            for claim in claims.values() {
                if let Some(c) = claim
                    .citations
                    .iter()
                    .find(|c| c.span.start == 0 && c.span.end == text.len() && c.text == text)
                {
                    out.exact_text_reference = Some(TextReference {
                        kind: "full_source_quote/1".into(),
                        key: c.span.source.clone(),
                        text_hash: c.span.text_hash.clone(),
                    });
                    break;
                }
            }
        }
        if out.exact_text_reference.is_some() {
            out.value.as_object_mut().unwrap().remove("text");
        }
    }
    out
}
/// Reconstruct original item bytes semantically (canonical serde JSON), without fetching anything.
pub fn reconstruct_item(
    item: &ItemValue,
    claims: &BTreeMap<String, layers::ClaimLayer>,
) -> Result<Value> {
    let mut value = item.value.clone();
    if let Some(reference) = &item.exact_text_reference {
        let text = match reference.kind.as_str() {
            "assertion_proposition/1" => claims
                .get(&reference.key)
                .map(|c| c.assertion.proposition.as_str()),
            "full_source_quote/1" => claims
                .values()
                .flat_map(|c| c.citations.iter())
                .find(|c| {
                    c.span.source == reference.key
                        && c.span.start == 0
                        && c.span.end == c.text.len()
                })
                .map(|c| c.text.as_str()),
            _ => None,
        }
        .ok_or("unresolved exact text reference")?;
        if digest(text.as_bytes()) != reference.text_hash {
            return Err("exact text reference hash mismatch".into());
        }
        let object = value
            .as_object_mut()
            .ok_or("text reference requires object")?;
        if object
            .insert("text".into(), Value::String(text.into()))
            .is_some()
        {
            return Err("text reference overlaps inline text".into());
        }
    }
    if digest(&bytes(&value)?) != item.original_value_hash {
        return Err("projected item reconstruction hash mismatch".into());
    }
    Ok(value)
}
/// Status derives from retained findings; a supplied contradictory label is invalid.
pub fn evidence_status(document: &Document) -> Result<EvidenceStatus> {
    let derived = if document.claims.is_empty() {
        EvidenceStatus::OmissionOnly
    } else {
        EvidenceStatus::SelectedEvidence
    };
    if document
        .evidence_status
        .is_some_and(|status| status != derived)
    {
        return Err("evidence status contradicts retained findings".into());
    }
    Ok(derived)
}
/// Deterministic, extractive review text. No summarization or model execution.
pub fn render_text(export: &Export, max_bytes: usize) -> Result<String> {
    let raw = bytes(&export.document)?;
    if export.bytes != raw.len() || export.digest != digest(&raw) || max_bytes > 1000000 {
        return Err("invalid export digest or render bound".into());
    }
    let d = &export.document;
    for item in &d.items {
        reconstruct_item(item, &d.claims)?;
    }
    if d.claims.keys().any(|id| !d.temporal.contains_key(id)) {
        return Err("selected claim temporal disclosure missing".into());
    }
    let status = evidence_status(d)?;
    let heading = if status == EvidenceStatus::OmissionOnly {
        "OMISSION RECEIPT - NO EVIDENCE SELECTED"
    } else {
        "SELECTED EVIDENCE REVIEW"
    };
    let status_line = if status == EvidenceStatus::OmissionOnly {
        NO_EVIDENCE_WARNING
    } else {
        "Selected findings are unreviewed evidence, not a complete answer."
    };
    let mut out=format!("{heading}\nStatus: {status:?}; findings: {}\n{status_line}\n{}\nArchive: {}\nMapping: {}\nQuestion: {}\nKnown at: {}; requested date: {}\nLicense: {}\nSharing: {}\n{}\n",d.claims.len(),d.trust,d.archive_digest,d.mapping_digest,d.question,d.known_as_of,d.valid_at,d.package_metadata.license,d.package_metadata.sharing_policy,d.methods_notice);
    out.push_str(&format!(
        "Package metadata: {}\n",
        String::from_utf8(bytes(&d.package_metadata)?).unwrap()
    ));
    for warning in &d.warnings {
        out.push_str(&format!("WARNING: {warning}\n"));
    }
    for (id, claim) in &d.claims {
        out.push_str(&format!("\nReported proposition [{id}]: {}\nTemporal status: {}\nExact qualified assertion and citations: {}\n",claim.assertion.proposition,d.temporal[id],String::from_utf8(bytes(claim)?).unwrap()));
    }
    out.push_str(&format!("\nSource metadata: {}\nRetained artifact items: {}\nRelationships: {}\nExplicit omissions: {}\n",String::from_utf8(bytes(&d.sources)?).unwrap(),String::from_utf8(bytes(&d.items)?).unwrap(),String::from_utf8(bytes(&d.relationships)?).unwrap(),String::from_utf8(bytes(&d.omissions)?).unwrap()));
    if out.len() > max_bytes {
        return Err("complete rendered review exceeds byte budget".into());
    }
    Ok(out)
}
