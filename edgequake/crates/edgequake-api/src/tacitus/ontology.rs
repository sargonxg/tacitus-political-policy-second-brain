use super::store::{bad, Store};
use crate::{
    error::{ApiError, ApiResult},
    handlers::auth::RequestAuthContext,
    middleware::TenantContext,
    state::AppState,
};
use axum::{extract::State, Extension, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use tacitus_core::{digest, ontology};
use uuid::Uuid;
fn config(ws: &edgequake_core::Workspace) -> Value {
    let keys = [
        "entity_types",
        "entity_types_strict",
        "relation_types",
        "relation_types_strict",
        "relation_edges",
        "kg_schema_preset",
    ];
    let mut result = serde_json::Map::new();
    for key in keys {
        result.insert(
            key.into(),
            ws.metadata.get(key).cloned().unwrap_or(Value::Null),
        );
    }
    Value::Object(result)
}
fn registry() -> Value {
    json!({"version":ontology::REGISTRY_VERSION,"packs":ontology::packs(),"relations":ontology::relations(),"tools":ontology::tools(),"base_ontology":"tacitus-policy/1","meaning":"Domain extraction vocabulary and graph filters; not automatic legal/causal classification"})
}
#[utoipa::path(get,path="/api/v1/tacitus/ontologies",tag="Tacitus policy",responses((status=200,body=Value)),security(("bearer_auth"=[])))]
pub async fn list(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(auth): Extension<RequestAuthContext>,
) -> ApiResult<Json<Value>> {
    let store = Store::scoped(&state, &ctx, &auth).await?;
    let ws = state
        .workspace_service
        .get_workspace(store.workspace)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound("Workspace unavailable".into()))?;
    let current = config(&ws);
    let recorded = store
        .records::<Value>("event")
        .await?
        .into_iter()
        .find(|e| e["action"] == "domain_profile_applied");
    let matches = recorded
        .as_ref()
        .is_some_and(|e| e["applied_configuration"] == current);
    let data = registry();
    let hash = digest(&serde_json::to_vec(&data).map_err(bad)?);
    Ok(Json(
        json!({"registry":data,"registry_digest":hash,"workspace_configuration":current,"recorded_profile":recorded,"configuration_matches_recorded_profile":matches,"review_required":true}),
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Apply {
    pub id: String,
    pub version: u32,
    pub registry_digest: String,
    pub replace_extraction_configuration: bool,
}
#[utoipa::path(post,path="/api/v1/tacitus/ontologies/apply",tag="Tacitus policy",request_body=Value,responses((status=200,body=Value)),security(("bearer_auth"=[])))]
pub async fn apply(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(auth): Extension<RequestAuthContext>,
    Json(input): Json<Apply>,
) -> ApiResult<Json<Value>> {
    let store = Store::scoped(&state, &ctx, &auth).await?;
    let role = crate::services::request_authorization::membership_role(
        &state,
        &auth.user_id,
        ctx.tenant_id.as_deref(),
        ctx.workspace_id.as_deref(),
    )
    .await?;
    if !matches!(
        role,
        edgequake_core::MembershipRole::Owner | edgequake_core::MembershipRole::Admin
    ) {
        return Err(ApiError::forbidden_reason(
            "Applying extraction vocabulary requires owner/admin membership",
        ));
    }
    let pack =
        ontology::find_pack(&input.id).ok_or_else(|| bad("Unknown native domain profile"))?;
    if !input.replace_extraction_configuration
        || input.version != pack.version
        || input.registry_digest != digest(&serde_json::to_vec(&registry()).map_err(bad)?)
    {
        return Err(ApiError::Conflict("Inspect the current registry/version and confirm replacement of future extraction settings".into()));
    }
    let before = state
        .workspace_service
        .get_workspace(store.workspace)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound("Workspace unavailable".into()))?;
    let request = edgequake_core::UpdateWorkspaceRequest {
        entity_types: Some(
            pack.entities
                .iter()
                .map(|e| e.extraction_type.clone())
                .collect(),
        ),
        entity_types_strict: Some(false),
        relation_types: Some(
            pack.predicates
                .iter()
                .map(|p| p.to_ascii_uppercase())
                .collect(),
        ),
        relation_types_strict: Some(false),
        kg_schema_preset: Some("none".into()),
        relation_edges: Some(vec![]),
        ..Default::default()
    };
    let updated = state
        .workspace_service
        .update_workspace(store.workspace, request)
        .await
        .map_err(ApiError::from)?;
    let applied = config(&updated);
    let id = Uuid::new_v4().to_string();
    store.insert("event",&id,1,&json!({"id":id,"action":"domain_profile_applied","profile":pack.id,"profile_version":pack.version,"registry_version":ontology::REGISTRY_VERSION,"registry_digest":input.registry_digest,"reviewer":store.actor,"before_configuration":config(&before),"applied_configuration":applied,"at":chrono::Utc::now().timestamp(),"effect":"future ingestion only; no retroactive relabeling or truth/review promotion"})).await?;
    Ok(Json(
        json!({"profile":pack,"workspace_configuration":applied,"effect":"future ingestion only","existing_assertions_changed":false}),
    ))
}
