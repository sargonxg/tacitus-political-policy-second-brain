use super::{
    sources,
    store::{bad, database, Store},
};
use crate::{
    error::{ApiError, ApiResult},
    handlers::auth::RequestAuthContext,
    middleware::TenantContext,
    state::AppState,
};
use axum::{extract::State, Extension, Json};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use tacitus_capsules::{
    policy::{evidence_name, Data, PolicyPackage},
    Manifest,
};
use tacitus_core::{
    digest,
    policy::{CognitiveMethod, HumanReview, PolicyAssertion, SourceRecord},
};
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub title: String,
    pub source_keys: Vec<String>,
    #[serde(default)]
    pub assertion_ids: Vec<String>,
    #[serde(default)]
    pub method_ids: Vec<String>,
    pub profile: String,
    pub evidence_mode: String,
    pub expected_preview_hash: Option<String>,
}
async fn package(
    state: &AppState,
    store: &Store,
    selection: &Selection,
) -> ApiResult<PolicyPackage> {
    if selection.title.trim().is_empty()
        || selection.title.len() > 200
        || selection.source_keys.is_empty()
        || selection.source_keys.len() > 32
        || selection.method_ids.len() > 32
        || selection.assertion_ids.len() > 256
    {
        return Err(bad("Bounded explicit capsule selection required"));
    }
    let live = sources::live_sources(state, store, chrono::Utc::now().timestamp(), None).await?;
    let mut selected = Vec::new();
    let mut evidence = BTreeMap::new();
    let mut keys = BTreeSet::new();
    for key in &selection.source_keys {
        if !keys.insert(key.clone()) {
            return Err(bad("Duplicate source selection"));
        }
        let mut source = live
            .get(key)
            .ok_or_else(|| ApiError::forbidden_reason("Export source not current/authorized"))?
            .clone();
        if selection.evidence_mode == "evidence_included" {
            evidence.insert(evidence_name(&source), source.text.as_bytes().to_vec());
        }
        source.text.clear();
        selected.push(source);
    }
    let assertions = store
        .records::<PolicyAssertion>("assertion")
        .await?
        .into_iter()
        .filter(|a| selection.assertion_ids.contains(&a.id))
        .collect::<Vec<_>>();
    let methods = store
        .records::<CognitiveMethod>("method")
        .await?
        .into_iter()
        .filter(|m| selection.method_ids.contains(&m.id))
        .collect::<Vec<_>>();
    if assertions.len() != selection.assertion_ids.len()
        || methods.len() != selection.method_ids.len()
    {
        return Err(bad("Capsule target absent or duplicate"));
    }
    for a in &assertions {
        a.validate(&live).map_err(bad)?;
        if a.evidence.iter().any(|e| !keys.contains(&e.source)) {
            return Err(bad("Select every assertion dependency"));
        }
    }
    for m in &methods {
        m.validate(&live).map_err(bad)?;
        if m.evidence.iter().any(|e| !keys.contains(&e.source)) {
            return Err(bad("Select every method dependency"));
        }
    }
    let target_ids = selection
        .assertion_ids
        .iter()
        .chain(&selection.method_ids)
        .cloned()
        .collect::<BTreeSet<_>>();
    let reviews = store
        .records::<HumanReview>("review")
        .await?
        .into_iter()
        .filter(|r| target_ids.contains(&r.target_id))
        .collect();
    // Deterministic package identity/time makes preview bind to the subsequent exact export.
    let identity = digest(
        serde_json::to_vec(&(
            selection.title.clone(),
            &selection.source_keys,
            &selection.assertion_ids,
            &selection.method_ids,
            &selection.profile,
            &selection.evidence_mode,
        ))
        .map_err(bad)?
        .as_slice(),
    );
    let created_at = selected.iter().map(|s| s.available_at).max().unwrap_or(0);
    let package=PolicyPackage {manifest:Manifest {schema_version:2,package_version:1,id:identity,author:store.actor.to_string(),title:selection.title.clone(),domain:"political-policy-analysis".into(),scope:store.workspace.to_string(),purpose:"Selected context and reviewed/scoped method proposals; integrity is not truth".into(),profile:selection.profile.clone(),created_at,valid_until:None,supersedes:None,dependencies:vec![],license:"Per-source rights; no new rights granted".into(),confidentiality:"private".into(),sharing_policy:"Review every field before sharing; references still disclose assertions/methods/reviewer metadata".into(),evidence_mode:selection.evidence_mode.clone(),limitations:vec!["Imported attestations are untrusted; no executable tools or permissions are imported".into(),"Native Tacitus v2; private Praxis compatibility unverified".into(),if selection.evidence_mode=="references_only"{"Original text absent; fresh workspace needs authorized originals".into()}else{"Only explicitly redistributable evidence included".into()}],files:BTreeMap::new()},data:Data {sources:selected,assertions,methods,reviews,artifacts:BTreeMap::new()},evidence};
    package.validate().map_err(bad)?;
    Ok(package)
}
#[utoipa::path(post,path="/api/v1/tacitus/capsules/preview",tag="Tacitus policy",request_body=Value,responses((status=200,body=Value)),security(("bearer_auth"=[])))]
pub async fn preview(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(auth): Extension<RequestAuthContext>,
    Json(selection): Json<Selection>,
) -> ApiResult<Json<Value>> {
    let store = Store::scoped(&state, &ctx, &auth).await?;
    let package = package(&state, &store, &selection).await?;
    let files = package.preview().map_err(bad)?;
    let hash = digest(&serde_json::to_vec(&files).map_err(bad)?);
    let content = files
        .into_iter()
        .map(|(n, b)| String::from_utf8(b).map(|s| (n, s)))
        .collect::<Result<BTreeMap<_, _>, _>>()
        .map_err(bad)?;
    Ok(Json(
        json!({"preview_hash":hash,"files":content,"warning":"References-only may still contain sensitive quotations, assertions, methods and reviewer metadata. Inspect the whole package."}),
    ))
}
#[utoipa::path(post,path="/api/v1/tacitus/capsules/export",tag="Tacitus policy",request_body=Value,responses((status=200,body=Value)),security(("bearer_auth"=[])))]
pub async fn export(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(auth): Extension<RequestAuthContext>,
    Json(selection): Json<Selection>,
) -> ApiResult<Json<Value>> {
    let store = Store::scoped(&state, &ctx, &auth).await?;
    let package = package(&state, &store, &selection).await?;
    let hash = digest(&serde_json::to_vec(&package.preview().map_err(bad)?).map_err(bad)?);
    if selection.expected_preview_hash.as_deref() != Some(&hash) {
        return Err(ApiError::Conflict(
            "Export differs from preview; inspect a new exact preview first".into(),
        ));
    }
    let mut spans = Vec::new();
    for source in &package.data.sources {
        let s: SourceRecord = store
            .record(
                "source",
                Uuid::parse_str(&source.id).map_err(bad)?,
                Some(source.version),
            )
            .await?;
        spans.push(tacitus_core::Span {
            source: s.key(),
            text_hash: s.text_hash,
            start: 0,
            end: s.text.len(),
            locator: None,
            original_language: s.language,
            translation: None,
        });
    }
    let mut tx = store.transaction().await?;
    store.ensure_evidence_in(&mut tx, &spans).await?;
    let bytes = package.export().map_err(bad)?;
    tx.commit().await.map_err(database)?;
    Ok(Json(
        json!({"filename":"tacitus-policy.capsule","capsule_base64":STANDARD.encode(bytes),"preview_hash":hash,"schema_version":2}),
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Import {
    pub capsule_base64: String,
}
#[utoipa::path(post,path="/api/v1/tacitus/capsules/import",tag="Tacitus policy",request_body=Value,responses((status=200,body=Value)),security(("bearer_auth"=[])))]
pub async fn import(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(auth): Extension<RequestAuthContext>,
    Json(input): Json<Import>,
) -> ApiResult<Json<Value>> {
    let store = Store::scoped(&state, &ctx, &auth).await?;
    if input.capsule_base64.len() > 12 * 1024 * 1024 {
        return Err(bad("Capsule exceeds bound"));
    }
    let bytes = STANDARD.decode(input.capsule_base64).map_err(bad)?;
    let imported = PolicyPackage::import(&bytes).map_err(bad)?;
    // References-only cannot silently fabricate evidence in a fresh workspace.
    let hydrated = imported.sources().map_err(bad)?;
    let mut source_map = BTreeMap::new();
    let mut fresh_sources = Vec::new();
    let mut documents = Vec::new();
    for source in hydrated.values() {
        let request:crate::handlers::documents_types::UploadDocumentRequest=serde_json::from_value(json!({"content":source.text,"title":source.title,"metadata":{"imported_capsule":imported.manifest.id},"async_processing":true,"enable_gleaning":false,"max_gleaning":1,"use_llm_summarization":true})).map_err(bad)?;
        let (_, Json(response)) = crate::handlers::documents::upload_document(
            State(state.clone()),
            ctx.clone(),
            Json(request),
        )
        .await?;
        let doc_id = response
            .duplicate_of
            .as_ref()
            .unwrap_or(&response.document_id)
            .clone();
        let mut fresh = source.clone();
        fresh.id = doc_id.clone();
        fresh.document_id = Some(doc_id.clone());
        fresh.workspace = store.workspace.to_string();
        fresh.available_at = chrono::Utc::now().timestamp();
        let prior = store
            .records::<SourceRecord>("source")
            .await?
            .into_iter()
            .find(|s| s.id == doc_id);
        if let Some(old) = prior {
            if old.text_hash == fresh.text_hash {
                fresh = old;
            } else {
                fresh.version = old.version + 1;
            }
        } else {
            fresh.version = 1;
        }
        fresh.validate().map_err(bad)?;
        source_map.insert(source.key(), fresh.key());
        fresh_sources.push(fresh);
        documents.push(response);
    }
    let remap = |spans: &mut Vec<tacitus_core::Span>| -> ApiResult<()> {
        for span in spans {
            span.source = source_map
                .get(&span.source)
                .ok_or_else(|| bad("Imported evidence reference missing"))?
                .clone();
        }
        Ok(())
    };
    let mut assertions = imported.data.assertions;
    let mut assertion_ids = Vec::new();
    for assertion in &mut assertions {
        assertion.id = Uuid::new_v4().to_string();
        assertion.version = 1;
        assertion.workspace = store.workspace.to_string();
        remap(&mut assertion.evidence)?;
        assertion_ids.push(assertion.id.clone());
    }
    let mut methods = imported.data.methods;
    let mut method_ids = Vec::new();
    for method in &mut methods {
        method.id = Uuid::new_v4().to_string();
        method.version = 1;
        method.workspace = store.workspace.to_string();
        method.available_at = chrono::Utc::now().timestamp();
        remap(&mut method.evidence)?;
        method_ids.push(method.id.clone());
    }
    let map = fresh_sources
        .iter()
        .map(|s| (s.key(), s.clone()))
        .collect::<BTreeMap<_, _>>();
    for a in &assertions {
        a.validate(&map).map_err(bad)?;
    }
    for m in &methods {
        m.validate(&map).map_err(bad)?;
    }
    let mut tx = store.transaction().await?;
    for source in &fresh_sources {
        // Duplicate imports do not replace existing canonical records or review history.
        sqlx::query("INSERT INTO tacitus_records(tenant_id,workspace_id,kind,id,version,payload) VALUES($1,$2,'source',$3,$4,$5) ON CONFLICT DO NOTHING")
            .bind(store.tenant).bind(store.workspace).bind(Uuid::parse_str(&source.id).map_err(bad)?).bind(source.version as i32).bind(serde_json::to_value(source).map_err(bad)?).execute(&mut *tx).await.map_err(database)?;
    }
    for a in &assertions {
        store
            .insert_in(&mut tx, "assertion", &a.id, a.version, a)
            .await?;
    }
    for m in &methods {
        store
            .insert_in(&mut tx, "method", &m.id, m.version, m)
            .await?;
    }
    let eid = Uuid::new_v4().to_string();
    store.insert_in(&mut tx,"event",&eid,1,&json!({"id":eid,"capsule":imported.manifest.id,"action":"imported_proposals_require_review","untrusted_attestation_count":imported.data.reviews.len(),"at":chrono::Utc::now().timestamp()})).await?;
    tx.commit().await.map_err(database)?;
    Ok(Json(
        json!({"documents":documents,"assertion_ids":assertion_ids,"method_ids":method_ids,"source_map":source_map,"review_state":"unreviewed_proposals","ingestion":"queued through existing EdgeQuake pipeline; wait for completion before reuse","limitations":imported.manifest.limitations}),
    ))
}
