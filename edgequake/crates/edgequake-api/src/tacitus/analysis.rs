use super::{
    provider_policy, sources,
    store::{bad, database, Store},
};
use crate::{
    error::{ApiError, ApiResult},
    handlers::auth::RequestAuthContext,
    middleware::TenantContext,
    state::AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};
use tacitus_analysis::harness::{self, Context, Task};
use tacitus_core::{
    digest,
    policy::{applicability, CognitiveMethod, HumanReview},
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub question: String,
    #[serde(default)]
    pub document_ids: Vec<String>,
    #[serde(default = "auto")]
    pub task: Task,
    #[serde(default)]
    pub deepen: bool,
    pub known_as_of: Option<i64>,
    pub valid_at: Option<i64>,
    #[serde(default)]
    pub facts: BTreeMap<String, bool>,
    #[serde(default)]
    pub method_ids: Vec<String>,
}
fn auto() -> Task {
    Task::Auto
}
#[utoipa::path(post,path="/api/v1/tacitus/analyses",tag="Tacitus policy",request_body=Value,responses((status=202,body=Value)),security(("bearer_auth"=[])))]
pub async fn submit(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(auth): Extension<RequestAuthContext>,
    Json(request): Json<Request>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    provider_policy::startup().map_err(bad)?;
    if !provider_policy::enabled() {
        return Err(bad("Enable TACITUS_LOCAL_ONLY=1 for the policy runtime"));
    }
    if request.question.trim().is_empty()
        || request.question.len() > 4000
        || request.document_ids.len() > 32
        || request.facts.len() > 32
        || request.method_ids.len() > 32
    {
        return Err(bad("Question/selection exceeds limits"));
    }
    let store = Store::scoped(&state, &ctx, &auth).await?;
    // Validate explicit document ownership before accepting a job or touching retrieval caches.
    for id in &request.document_ids {
        sources::authorized_metadata(&store, Uuid::parse_str(id).map_err(bad)?).await?;
    }
    let id = Uuid::new_v4();
    let mut tx = store.transaction().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("analysis:{}:{}", store.tenant, store.workspace))
        .execute(&mut *tx)
        .await
        .map_err(database)?;
    let active:i64=sqlx::query_scalar("SELECT count(*) FROM tacitus_analyses WHERE tenant_id=$1 AND workspace_id=$2 AND status IN ('queued','retrieving','generating')").bind(store.tenant).bind(store.workspace).fetch_one(&mut *tx).await.map_err(database)?;
    if active >= 2 {
        return Err(ApiError::Conflict(
            "Two analyses already active; cancel or wait".into(),
        ));
    }
    sqlx::query("INSERT INTO tacitus_analyses(id,tenant_id,workspace_id,user_id,status,request) VALUES($1,$2,$3,$4,'queued',$5)")
        .bind(id).bind(store.tenant).bind(store.workspace).bind(store.actor).bind(serde_json::to_value(&request).map_err(bad)?).execute(&mut *tx).await.map_err(database)?;
    tx.commit().await.map_err(database)?;
    tokio::spawn(async move {
        if let Err(error) = run(&state, &ctx, &store, id, request).await {
            let _ = store
                .transition(id, "failed", None, None, Some(&error.to_string()))
                .await;
        }
    });
    Ok((
        StatusCode::ACCEPTED,
        Json(json!({"id":id,"status":"queued"})),
    ))
}
async fn cancellable<T>(
    store: &Store,
    id: Uuid,
    future: impl std::future::Future<Output = ApiResult<T>>,
) -> ApiResult<T> {
    tokio::pin!(future);
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    loop {
        tokio::select! {result=&mut future=>return result,_=interval.tick()=>{
            if store.analysis(id).await?["status"]=="cancelled" {return Err(ApiError::Conflict("Analysis cancelled".into()));}
        }}
    }
}
pub async fn generate(prompt: &str, deepen: bool) -> ApiResult<(String, Value)> {
    provider_policy::validate_identity("ollama", &provider_policy::model(false), false)
        .map_err(bad)?;
    let context_tokens = if deepen { 16384 } else { 8192 };
    let output_tokens = if deepen { 4096 } else { 2048 };
    if prompt.len() + output_tokens > context_tokens {
        return Err(bad(
            "Serialized prompt exceeds conservative context budget; narrow selection or deepen",
        ));
    }
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(600))
        .build()
        .map_err(database)?;
    let host = provider_policy::host();
    let model = provider_policy::model(false);
    let tags: Value = client
        .get(format!("{host}/api/tags"))
        .send()
        .await
        .map_err(|_| ApiError::Internal("Ollama unavailable; no cloud/mock fallback".into()))?
        .error_for_status()
        .map_err(|_| bad("Cannot inspect installed Ollama models"))?
        .json()
        .await
        .map_err(database)?;
    let installed = tags["models"]
        .as_array()
        .and_then(|models| models.iter().find(|m| m["name"].as_str() == Some(&model)))
        .ok_or_else(|| {
            bad("Configured local model is not installed; download on the cloud host")
        })?;
    let started = Instant::now();
    let response=client.post(format!("{host}/api/generate")).json(&json!({"model":model,"prompt":prompt,"format":"json","stream":false,"think":false,"options":{"temperature":0,"num_ctx":context_tokens,"num_predict":output_tokens}})).send().await.map_err(|_|ApiError::Internal("Local inference failed; no substitute model".into()))?;
    if !response.status().is_success() {
        return Err(ApiError::Internal(
            "Ollama rejected generation; inspect model/service status".into(),
        ));
    }
    if response.content_length().is_some_and(|n| n > 128 * 1024) {
        return Err(bad("Model response exceeds bound"));
    }
    let mut response = response;
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(database)? {
        if bytes.len() + chunk.len() > 128 * 1024 {
            return Err(bad("Model response exceeds bound"));
        }
        bytes.extend_from_slice(&chunk);
    }
    let output: Value = serde_json::from_slice(&bytes).map_err(bad)?;
    if output["done_reason"] == "length" || output["done"] != true {
        return Err(bad(
            "Local model response incomplete; narrow evidence or deepen",
        ));
    }
    let raw = output["response"]
        .as_str()
        .ok_or_else(|| bad("Ollama structured response absent"))?
        .to_string();
    Ok((
        raw,
        json!({"provider":"ollama","model":model,"model_digest":installed["digest"],"prompt_tokens":output["prompt_eval_count"],"output_tokens":output["eval_count"],"duration_ms":started.elapsed().as_millis(),"context_capacity":context_tokens,"source":"real_ollama_response"}),
    ))
}
async fn run(
    state: &AppState,
    ctx: &TenantContext,
    store: &Store,
    id: Uuid,
    request: Request,
) -> ApiResult<()> {
    crate::services::request_authorization::membership_role(
        state,
        &store.actor.to_string(),
        Some(&store.tenant.to_string()),
        Some(&store.workspace.to_string()),
    )
    .await?;
    if !store.transition(id, "retrieving", None, None, None).await? {
        return Ok(());
    }
    let mut selected = sources::selected(state, store, &request.document_ids).await?;
    let known = request
        .known_as_of
        .unwrap_or_else(|| chrono::Utc::now().timestamp());
    // Historical selection uses the immutable revision available at that cutoff.
    let live = sources::live_sources(state, store, known, request.valid_at).await?;
    let ids: std::collections::BTreeSet<_> = selected.iter().map(|s| s.id.clone()).collect();
    selected = live
        .values()
        .filter(|s| ids.contains(&s.id) || s.document_id.is_none())
        .cloned()
        .collect();
    if selected.is_empty() {
        return Err(bad(
            "No authorized evidence available at the requested dates",
        ));
    }
    let doc_ids: Vec<_> = selected
        .iter()
        .filter_map(|s| s.document_id.clone())
        .collect();
    let retrieval = if doc_ids.is_empty() {
        None
    } else {
        let retrieval_request:crate::handlers::context_types::ContextRetrievalRequest=serde_json::from_value(json!({"query":request.question,"mode":"mix","max_results":16,"content_granularity":"agent","document_filter":{"document_ids":doc_ids},"enable_rerank":false,"include_lineage":true,"include_subgraph":true})).map_err(bad)?;
        Some(
            cancellable(
                store,
                id,
                crate::services::query_context::retrieve_context(
                    state,
                    ctx,
                    retrieval_request,
                    None,
                ),
            )
            .await?,
        )
    };
    let mut candidates = Vec::new();
    if let Some(ref r) = retrieval {
        for chunk in &r.bundle.chunks {
            let doc = chunk
                .lineage
                .as_ref()
                .and_then(|l| l.document_id.as_ref())
                .ok_or_else(|| bad("Retrieval chunk has no authorized document lineage"))?;
            if !doc_ids.contains(doc) {
                return Err(bad("Retrieval crossed selected document scope"));
            }
            candidates.push((doc.clone(), chunk.content.clone()));
        }
    }
    let methods = store.records::<CognitiveMethod>("method").await?;
    let reviews = store.records::<HumanReview>("review").await?;
    let mut active = Vec::new();
    let mut method_receipts = Vec::new();
    let mut method_ids = std::collections::BTreeSet::new();
    for method in methods {
        if !method_ids.insert(method.id.clone()) {
            continue;
        }
        let review = reviews
            .iter()
            .find(|r| r.target_id == method.id && r.target_kind == "method");
        let applicability = applicability(
            &method,
            review,
            &live,
            &store.workspace.to_string(),
            known,
            &request.facts,
        );
        if request.method_ids.contains(&method.id) && applicability.state != "active" {
            return Err(bad(format!(
                "Selected method {} is {}; review its scope/provenance/prerequisites",
                method.id, applicability.state
            )));
        }
        method_receipts.push(
            json!({"id":method.id,"state":applicability.state,"reasons":applicability.reasons}),
        );
        if applicability.state == "active" && request.method_ids.contains(&method.id) {
            active.push(method);
        }
    }
    if request.method_ids.iter().any(|id| !method_ids.contains(id)) {
        return Err(bad("Selected method unavailable in this workspace"));
    }
    let task = harness::route(&request.question, request.task);
    let context = harness::compile(
        &request.question,
        task,
        &selected,
        &candidates,
        active,
        if request.deepen { 10000 } else { 4000 },
    )
    .map_err(bad)?;
    if !store
        .transition(
            id,
            "generating",
            Some(serde_json::to_value(&context).map_err(bad)?),
            None,
            None,
        )
        .await?
    {
        return Ok(());
    }
    let prompt = harness::prompt(&context).map_err(bad)?;
    let (raw, model) = cancellable(store, id, generate(&prompt, request.deepen)).await?;
    let output = harness::validate(&raw, &context).map_err(bad)?;
    let assertion_ids = output
        .claims
        .iter()
        .map(|_| Uuid::new_v4().to_string())
        .collect::<Vec<_>>();
    let assertions = harness::assertions(
        &output,
        &context,
        &store.workspace.to_string(),
        chrono::Utc::now().timestamp(),
        &assertion_ids,
    )
    .map_err(bad)?;
    let map = selected
        .into_iter()
        .map(|s| (s.key(), s))
        .collect::<BTreeMap<_, _>>();
    for a in &assertions {
        a.validate(&map).map_err(bad)?;
    }
    let context_digest = digest(&serde_json::to_vec(&context).map_err(bad)?);
    let retrieval_chunks = retrieval.as_ref().map_or(0, |r| r.bundle.chunks.len());
    let current = sources::live_sources(
        state,
        store,
        chrono::Utc::now().timestamp(),
        request.valid_at,
    )
    .await?;
    let stale = context
        .passages
        .iter()
        .any(|p| !current.contains_key(&p.span.source));
    crate::services::request_authorization::membership_role(
        state,
        &store.actor.to_string(),
        Some(&store.tenant.to_string()),
        Some(&store.workspace.to_string()),
    )
    .await?;
    let result = json!({"answer":output,"assertions":assertions,"model":model,"context_digest":context_digest,"task":task,"routing":"bounded rule-based suggestion; user may override","methods":method_receipts,"stale":stale,"review_state":"proposed","citation_validation":"exact identity/quote checked; semantic support requires human review","retrieval":retrieval.as_ref().map(|r|json!({"fingerprint":r.retrieval_fingerprint,"stats":r.stats,"quality":r.retrieval_quality}))});
    let mut result = result;
    result["retrieval_chunks"] = json!(retrieval_chunks);
    store.complete(id, result, &assertions).await
}
#[utoipa::path(get,path="/api/v1/tacitus/analyses/{id}",tag="Tacitus policy",params(("id"=Uuid,Path)),responses((status=200,body=Value)),security(("bearer_auth"=[])))]
pub async fn get(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(auth): Extension<RequestAuthContext>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Value>> {
    let store = Store::scoped(&state, &ctx, &auth).await?;
    let mut value = store.analysis(id).await?;
    if let Some(context) = value.get("context").filter(|v| !v.is_null()) {
        let context: Context = serde_json::from_value(context.clone()).map_err(database)?;
        let records = store
            .records::<tacitus_core::policy::SourceRecord>("source")
            .await?;
        for passage in &context.passages {
            let s = records
                .iter()
                .find(|s| s.key() == passage.span.source)
                .ok_or_else(|| ApiError::forbidden_reason("Answer evidence unavailable"))?;
            sources::authorize_record(&store, s).await?;
        }
        let live =
            sources::live_sources(&state, &store, chrono::Utc::now().timestamp(), None).await?;
        value["freshness"] = json!(if context
            .passages
            .iter()
            .all(|p| live.contains_key(&p.span.source))
        {
            "current"
        } else {
            "review_required"
        });
    }
    Ok(Json(value))
}
#[utoipa::path(get,path="/api/v1/tacitus/analyses",tag="Tacitus policy",responses((status=200,body=Value)),security(("bearer_auth"=[])))]
pub async fn list(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(auth): Extension<RequestAuthContext>,
) -> ApiResult<Json<Value>> {
    let store = Store::scoped(&state, &ctx, &auth).await?;
    let mut tx = store.transaction().await?;
    let rows=sqlx::query("SELECT id,status,request->>'question' AS question FROM tacitus_analyses WHERE tenant_id=$1 AND workspace_id=$2 ORDER BY created_at DESC LIMIT 50").bind(store.tenant).bind(store.workspace).fetch_all(&mut *tx).await.map_err(database)?;
    Ok(Json(
        json!({"analyses":rows.into_iter().map(|r|json!({"id":r.get::<Uuid,_>("id"),"status":r.get::<String,_>("status"),"question":r.get::<Option<String>,_>("question")})).collect::<Vec<_>>()}),
    ))
}
#[utoipa::path(post,path="/api/v1/tacitus/analyses/{id}/cancel",tag="Tacitus policy",params(("id"=Uuid,Path)),responses((status=200,body=Value)),security(("bearer_auth"=[])))]
pub async fn cancel(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(auth): Extension<RequestAuthContext>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Value>> {
    let store = Store::scoped(&state, &ctx, &auth).await?;
    store.analysis(id).await?;
    let changed = store.transition(id, "cancelled", None, None, None).await?;
    Ok(Json(json!({"id":id,"cancelled":changed})))
}
