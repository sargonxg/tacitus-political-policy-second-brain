use super::store::{bad, database, Store};
use crate::{
    error::{ApiError, ApiResult},
    handlers::auth::RequestAuthContext,
    middleware::TenantContext,
    state::AppState,
};
use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;
use std::collections::BTreeMap;
use tacitus_core::{digest, policy::SourceRecord};
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    #[serde(default)]
    pub document_ids: Vec<String>,
}
pub async fn authorized_metadata(store: &Store, id: Uuid) -> ApiResult<Value> {
    let mut tx = store.transaction().await?;
    let row=sqlx::query("SELECT to_jsonb(d)-'content' AS metadata FROM public.documents d WHERE id=$1 AND workspace_id=$2 AND (tenant_id=$3 OR tenant_id IS NULL)")
        .bind(id).bind(store.workspace).bind(store.tenant).fetch_optional(&mut *tx).await.map_err(database)?.ok_or_else(||ApiError::NotFound("Selected document unavailable".into()))?;
    Ok(row.get("metadata"))
}
pub async fn snapshot(state: &AppState, store: &Store, id: Uuid) -> ApiResult<SourceRecord> {
    let metadata = authorized_metadata(store, id).await?;
    if !metadata
        .get("status")
        .and_then(Value::as_str)
        .is_some_and(|s| ["completed", "indexed"].contains(&s))
    {
        return Err(ApiError::Conflict(
            "Wait for real document ingestion to complete before policy analysis".into(),
        ));
    }
    let body = crate::services::document_body_loader::load_document_body(
        &state.storage,
        &id.to_string(),
        &metadata,
    )
    .await
    .ok_or_else(|| {
        ApiError::Conflict("Document extraction is not ready; inspect ingestion progress".into())
    })?;
    if body.markdown.len() > 2 * 1024 * 1024 {
        return Err(bad(
            "Policy source exceeds 2 MiB; select a smaller document rather than truncating",
        ));
    }
    let mut tx = store.transaction().await?;
    let canonical:Option<Uuid>=sqlx::query_scalar("SELECT id FROM tacitus_records WHERE tenant_id=$1 AND workspace_id=$2 AND kind='source' AND payload->>'document_id'=$3 ORDER BY seq DESC LIMIT 1").bind(store.tenant).bind(store.workspace).bind(id.to_string()).fetch_optional(&mut *tx).await.map_err(database)?;
    let canonical = canonical.unwrap_or(id);
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("{}:{}:{canonical}", store.tenant, store.workspace))
        .execute(&mut *tx)
        .await
        .map_err(database)?;
    let prior=sqlx::query("SELECT payload FROM tacitus_records WHERE tenant_id=$1 AND workspace_id=$2 AND kind='source' AND id=$3 ORDER BY version DESC LIMIT 1")
        .bind(store.tenant).bind(store.workspace).bind(canonical).fetch_optional(&mut *tx).await.map_err(database)?;
    let previous: Option<SourceRecord> = prior
        .map(|r| serde_json::from_value(r.get::<Value, _>("payload")))
        .transpose()
        .map_err(database)?;
    if previous
        .as_ref()
        .is_some_and(|s| s.document_id.as_deref() != Some(id.to_string().as_str()))
    {
        return Err(ApiError::Conflict("This document was superseded; select its replacement or inspect its pinned source version".into()));
    }
    let hash = digest(body.markdown.as_bytes());
    if let Some(ref old) = previous {
        if old.text_hash == hash
            && !old.withdrawn
            && old.extractor == format!("edgequake-body/{}", body.source.as_str())
            && old.title
                == metadata
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or("Selected document")
        {
            return Ok(old.clone());
        }
    }
    let now = chrono::Utc::now().timestamp();
    let source = SourceRecord {
        id: canonical.to_string(),
        version: previous.as_ref().map_or(1, |s| s.version + 1),
        workspace: store.workspace.to_string(),
        document_id: Some(id.to_string()),
        title: metadata
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("Selected document")
            .into(),
        author: previous.as_ref().and_then(|s| s.author.clone()),
        original_hash: None,
        text_hash: hash,
        text: body.markdown,
        language: previous
            .as_ref()
            .map_or("und".into(), |s| s.language.clone()),
        extractor: format!("edgequake-body/{}", body.source.as_str()),
        published_at: previous.as_ref().and_then(|s| s.published_at),
        retrieved_at: now,
        available_at: now,
        valid_from: previous.as_ref().and_then(|s| s.valid_from),
        valid_until: previous.as_ref().and_then(|s| s.valid_until),
        source_family: previous
            .as_ref()
            .map_or("unverified-origin".into(), |s| s.source_family.clone()),
        withdrawn: false,
        restricted: previous.as_ref().is_some_and(|s| s.restricted),
        redistributable: previous.as_ref().is_some_and(|s| s.redistributable),
        license: previous.as_ref().and_then(|s| s.license.clone()),
    };
    source.validate().map_err(bad)?;
    store
        .insert_in(&mut tx, "source", &source.id, source.version, &source)
        .await?;
    if let Some(old) = previous {
        let eid = Uuid::new_v4().to_string();
        store.insert_in(&mut tx,"event",&eid,1,&json!({"id":eid,"source":old.key(),"replacement":source.key(),"reason":"new source version","action":"review_required","at":now})).await?;
    }
    tx.commit().await.map_err(database)?;
    Ok(source)
}
pub async fn selected(
    state: &AppState,
    store: &Store,
    ids: &[String],
) -> ApiResult<Vec<SourceRecord>> {
    if ids.len() > 32 {
        return Err(bad("Select at most 32 documents"));
    }
    let selected = if ids.is_empty() {
        let mut tx = store.transaction().await?;
        sqlx::query("SELECT d.id FROM public.documents d WHERE workspace_id=$1 AND (tenant_id=$2 OR tenant_id IS NULL) AND status IN ('completed','indexed') AND NOT EXISTS (SELECT 1 FROM tacitus_records older JOIN tacitus_records newer ON newer.tenant_id=older.tenant_id AND newer.workspace_id=older.workspace_id AND newer.kind='source' AND newer.id=older.id AND newer.version>older.version WHERE older.tenant_id=$2 AND older.workspace_id=$1 AND older.kind='source' AND older.payload->>'document_id'=d.id::text AND newer.payload->>'document_id'<>d.id::text) ORDER BY created_at DESC LIMIT 33")
            .bind(store.workspace).bind(store.tenant).fetch_all(&mut *tx).await.map_err(database)?.into_iter().map(|r|r.get::<Uuid,_>("id").to_string()).collect::<Vec<_>>()
    } else {
        ids.to_vec()
    };
    if selected.len() > 32 {
        return Err(bad("More than 32 documents: select a dossier explicitly"));
    }
    let mut sources = Vec::new();
    for id in selected {
        sources.push(snapshot(state, store, Uuid::parse_str(&id).map_err(bad)?).await?);
    }
    Ok(sources)
}
pub async fn live_sources(
    state: &AppState,
    store: &Store,
    known: i64,
    valid: Option<i64>,
) -> ApiResult<BTreeMap<String, SourceRecord>> {
    let records = store.records::<SourceRecord>("source").await?;
    let mut live = BTreeMap::new();
    let mut ids = std::collections::BTreeSet::new();
    for s in records {
        if s.available_at > known || !ids.insert(s.id.clone()) {
            continue;
        }
        if authorize_record(store, &s).await.is_err() {
            continue;
        }
        if let Some(ref doc) = s.document_id {
            if authorized_metadata(store, Uuid::parse_str(doc).map_err(bad)?)
                .await
                .is_err()
            {
                continue;
            }
        }
        if s.eligible(&store.workspace.to_string(), known, valid) {
            s.validate().map_err(bad)?;
            live.insert(s.key(), s);
        }
    }
    let _ = state;
    Ok(live)
}
/// Historical content remains inspectable only while its current source is authorized.
pub async fn authorize_record(store: &Store, source: &SourceRecord) -> ApiResult<()> {
    let latest: SourceRecord = store
        .record("source", Uuid::parse_str(&source.id).map_err(bad)?, None)
        .await?;
    if source.restricted || source.withdrawn || latest.restricted || latest.withdrawn {
        return Err(ApiError::forbidden_reason("Source access withdrawn"));
    }
    if let Some(ref doc) = source.document_id {
        authorized_metadata(store, Uuid::parse_str(doc).map_err(bad)?).await?;
    }
    Ok(())
}
#[utoipa::path(post,path="/api/v1/tacitus/sources",tag="Tacitus policy",request_body=Value,responses((status=200,body=Value)),security(("bearer_auth"=[])))]
pub async fn sync(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(auth): Extension<RequestAuthContext>,
    Json(request): Json<Selection>,
) -> ApiResult<Json<Value>> {
    let store = Store::scoped(&state, &ctx, &auth).await?;
    let sources = selected(&state, &store, &request.document_ids).await?;
    Ok(Json(
        json!({"sources":sources,"validation":"immutable extracted text; statements are unreviewed"}),
    ))
}
#[utoipa::path(get,path="/api/v1/tacitus/sources",tag="Tacitus policy",responses((status=200,body=Value)),security(("bearer_auth"=[])))]
pub async fn list(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(auth): Extension<RequestAuthContext>,
) -> ApiResult<Json<Value>> {
    let store = Store::scoped(&state, &ctx, &auth).await?;
    let sources = live_sources(&state, &store, chrono::Utc::now().timestamp(), None).await?;
    Ok(Json(
        json!({"sources":sources.values().map(|s|json!({"id":s.id,"version":s.version,"key":s.key(),"title":s.title,"language":s.language,"published_at":s.published_at,"source_family":s.source_family,"redistributable":s.redistributable})).collect::<Vec<_>>()}),
    ))
}
#[derive(Deserialize)]
pub struct Inspect {
    pub version: Option<u32>,
    pub start: Option<usize>,
    pub end: Option<usize>,
}
#[utoipa::path(get,path="/api/v1/tacitus/sources/{id}",tag="Tacitus policy",params(("id"=Uuid,Path),("version"=Option<u32>,Query),("start"=Option<usize>,Query),("end"=Option<usize>,Query)),responses((status=200,body=Value)),security(("bearer_auth"=[])))]
pub async fn inspect(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(auth): Extension<RequestAuthContext>,
    Path(id): Path<Uuid>,
    Query(query): Query<Inspect>,
) -> ApiResult<Json<Value>> {
    let store = Store::scoped(&state, &ctx, &auth).await?;
    let source: SourceRecord = store.record("source", id, query.version).await?;
    authorize_record(&store, &source).await?;
    source.validate().map_err(bad)?;
    let start = query.start.unwrap_or(0);
    let end = query.end.unwrap_or(source.text.len());
    let span = tacitus_core::Span {
        source: source.key(),
        text_hash: source.text_hash.clone(),
        start,
        end,
        locator: None,
        original_language: source.language.clone(),
        translation: None,
    };
    let text = source.passage(&span).map_err(bad)?.to_string();
    Ok(Json(
        json!({"source":source,"span":span,"passage":text,"semantic_support":"not_verified"}),
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    pub expected_version: u32,
    pub author: Option<String>,
    pub language: String,
    pub published_at: Option<i64>,
    pub valid_from: Option<i64>,
    pub valid_until: Option<i64>,
    pub source_family: String,
    pub redistributable: bool,
    pub license: Option<String>,
}
#[utoipa::path(patch,path="/api/v1/tacitus/sources/{id}",tag="Tacitus policy",params(("id"=Uuid,Path)),request_body=Value,responses((status=200,body=Value)),security(("bearer_auth"=[])))]
pub async fn metadata(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(auth): Extension<RequestAuthContext>,
    Path(id): Path<Uuid>,
    Json(input): Json<Metadata>,
) -> ApiResult<Json<Value>> {
    let store = Store::scoped(&state, &ctx, &auth).await?;
    let mut tx = store.transaction().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("{}:{}:{id}", store.tenant, store.workspace))
        .execute(&mut *tx)
        .await
        .map_err(database)?;
    let row=sqlx::query("SELECT payload FROM tacitus_records WHERE tenant_id=$1 AND workspace_id=$2 AND kind='source' AND id=$3 ORDER BY version DESC LIMIT 1").bind(store.tenant).bind(store.workspace).bind(id).fetch_optional(&mut *tx).await.map_err(database)?.ok_or_else(||ApiError::NotFound("Source unavailable".into()))?;
    let mut source: SourceRecord = serde_json::from_value(row.get("payload")).map_err(database)?;
    if source.version != input.expected_version {
        return Err(ApiError::Conflict(
            "Source changed; inspect latest version".into(),
        ));
    }
    if let Some(ref doc) = source.document_id {
        authorized_metadata(&store, Uuid::parse_str(doc).map_err(bad)?).await?;
    }
    let old = source.key();
    source.version += 1;
    source.author = input.author;
    source.language = input.language;
    source.published_at = input.published_at;
    source.valid_from = input.valid_from;
    source.valid_until = input.valid_until;
    source.source_family = input.source_family;
    source.redistributable = input.redistributable;
    source.license = input.license;
    source.available_at = chrono::Utc::now().timestamp();
    source.validate().map_err(bad)?;
    store
        .insert_in(&mut tx, "source", &source.id, source.version, &source)
        .await?;
    let eid = Uuid::new_v4().to_string();
    store.insert_in(&mut tx,"event",&eid,1,&json!({"id":eid,"source":old,"reason":"Metadata/rights/provenance revision","action":"review_required","at":source.available_at})).await?;
    tx.commit().await.map_err(database)?;
    Ok(Json(json!({"source":source})))
}
#[utoipa::path(get,path="/api/v1/tacitus/status",tag="Tacitus policy",responses((status=200,body=Value)),security(("bearer_auth"=[])))]
pub async fn status(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(auth): Extension<RequestAuthContext>,
) -> ApiResult<Json<Value>> {
    Store::scoped(&state, &ctx, &auth).await?;
    super::provider_policy::startup().map_err(bad)?;
    Ok(Json(
        json!({"storage":"postgresql","provider":"ollama","local_only_enforced":super::provider_policy::enabled(),"model":super::provider_policy::model(false),"embedding_model":super::provider_policy::model(true),"runtime_acceptance":"pending_cloud_verification","ontology":"tacitus-policy/1"}),
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Replacement {
    pub expected_version: u32,
    pub document_id: Uuid,
}
#[utoipa::path(post,path="/api/v1/tacitus/sources/{id}/replacement",tag="Tacitus policy",params(("id"=Uuid,Path)),request_body=Value,responses((status=200,body=Value)),security(("bearer_auth"=[])))]
pub async fn replace(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(auth): Extension<RequestAuthContext>,
    Path(id): Path<Uuid>,
    Json(input): Json<Replacement>,
) -> ApiResult<Json<Value>> {
    let store = Store::scoped(&state, &ctx, &auth).await?;
    let old: SourceRecord = store.record("source", id, None).await?;
    authorize_record(&store, &old).await?;
    if old.version != input.expected_version {
        return Err(ApiError::Conflict(
            "Source changed; inspect current version".into(),
        ));
    }
    let mut fresh = snapshot(&state, &store, input.document_id).await?;
    if fresh.id == old.id {
        return Err(bad("Replacement must be a different ingested document"));
    }
    let mut tx = store.transaction().await?;
    let full = tacitus_core::Span {
        source: old.key(),
        text_hash: old.text_hash.clone(),
        start: 0,
        end: old.text.len(),
        locator: None,
        original_language: old.language.clone(),
        translation: None,
    };
    store.ensure_evidence_in(&mut tx, &[full]).await?;
    fresh.id = old.id.clone();
    fresh.version = old.version + 1;
    fresh.available_at = chrono::Utc::now().timestamp();
    // New evidence does not inherit redistribution rights, authorship or language attestations.
    fresh.source_family = old.source_family.clone();
    fresh.author = None;
    fresh.redistributable = false;
    fresh.license = None;
    fresh.validate().map_err(bad)?;
    store
        .insert_in(&mut tx, "source", &fresh.id, fresh.version, &fresh)
        .await?;
    let eid = Uuid::new_v4().to_string();
    store.insert_in(&mut tx,"event",&eid,1,&json!({"id":eid,"source":old.key(),"replacement":fresh.key(),"reason":"changed evidence text/document","action":"review_required","at":fresh.available_at})).await?;
    tx.commit().await.map_err(database)?;
    Ok(Json(json!({"source":fresh,"action":"review_required"})))
}
