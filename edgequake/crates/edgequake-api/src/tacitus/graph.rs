//! Authorized, temporal views of the canonical attributed-assertion graph.
use super::{
    sources,
    store::{bad, Store},
};
use crate::{
    error::ApiResult, handlers::auth::RequestAuthContext, middleware::TenantContext,
    state::AppState,
};
use axum::{
    extract::{Query, State},
    Extension, Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use tacitus_core::{
    ontology::{self, GraphTool},
    policy::{HumanReview, PolicyAssertion, SourceRecord, ONTOLOGY_VERSION},
};
use uuid::Uuid;

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphFilter {
    pub tool: Option<String>,
    pub actor: Option<String>,
    pub review_state: Option<String>,
    pub known_as_of: Option<i64>,
    pub valid_at: Option<i64>,
}
impl GraphFilter {
    fn validate(&self, now: i64) -> ApiResult<(i64, Option<GraphTool>)> {
        let known = self.known_as_of.unwrap_or(now);
        if known > now {
            return Err(bad("Knowledge cutoff cannot be in the future"));
        }
        if self.actor.as_ref().is_some_and(|s| s.len() > 200) {
            return Err(bad("Actor label search is bounded to 200 bytes"));
        }
        if self.review_state.as_deref().is_some_and(|s| {
            !["proposed", "human_reviewed", "rejected", "review_required"].contains(&s)
        }) {
            return Err(bad("Unknown review state"));
        }
        let tool = self
            .tool
            .as_deref()
            .map(|id| ontology::find_tool(id).ok_or_else(|| bad("Unknown native graph tool")))
            .transpose()?;
        Ok((known, tool))
    }
}
#[utoipa::path(get,path="/api/v1/tacitus/graph",tag="Tacitus policy",params(("tool"=Option<String>,Query),("actor"=Option<String>,Query),("review_state"=Option<String>,Query),("known_as_of"=Option<i64>,Query),("valid_at"=Option<i64>,Query)),responses((status=200,body=Value)),security(("bearer_auth"=[])))]
pub async fn graph(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(auth): Extension<RequestAuthContext>,
    Query(filter): Query<GraphFilter>,
) -> ApiResult<Json<Value>> {
    let store = Store::scoped(&state, &ctx, &auth).await?;
    let now = chrono::Utc::now().timestamp();
    let (known, tool) = filter.validate(now)?;
    // Freshness uses current provenance even when displaying historical selections.
    let current_sources = sources::live_sources(&state, &store, now, None).await?;
    let reviews = store.records::<HumanReview>("review").await?;
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut ids = BTreeSet::new();
    for a in store.records::<PolicyAssertion>("assertion").await? {
        if a.created_at > known {
            continue;
        }
        if !ids.insert(a.id.clone()) {
            continue;
        }
        if tool.as_ref().is_some_and(|t| !t.matches(&a))
            || !ontology::actor_matches(&a, filter.actor.as_deref())
        {
            continue;
        }
        let stale = a.validate(&current_sources).is_err();
        let mut authorized = true;
        let mut eligible = true;
        for span in &a.evidence {
            let (id, version) = span
                .source
                .rsplit_once('@')
                .ok_or_else(|| bad("Invalid source reference"))?;
            let source: SourceRecord = store
                .record(
                    "source",
                    Uuid::parse_str(id).map_err(bad)?,
                    Some(version.parse().map_err(bad)?),
                )
                .await?;
            if super::sources::authorize_record(&store, &source)
                .await
                .is_err()
            {
                authorized = false;
                break;
            }
            eligible &= source.eligible(&store.workspace.to_string(), known, filter.valid_at);
        }
        if !authorized || !eligible {
            continue;
        }
        let receipt = reviews.iter().find(|r| {
            r.target_id == a.id && r.target_kind == "assertion" && r.reviewed_at <= known
        });
        let reviewed =
            !stale && receipt.is_some_and(|r| r.accepted(&a.fingerprint(), known, &a.evidence));
        let rejected = receipt.is_some_and(|r| {
            r.origin == "authenticated_human"
                && r.decision == "reject"
                && r.target_fingerprint == a.fingerprint()
        });
        let review_state = if stale {
            "review_required"
        } else if reviewed {
            "human_reviewed"
        } else if rejected {
            "rejected"
        } else {
            "proposed"
        };
        if filter
            .review_state
            .as_deref()
            .is_some_and(|s| s != review_state)
        {
            continue;
        }
        let from = format!("{}:subject", a.id);
        let to = format!("{}:object", a.id);
        nodes.push(json!({"id":from,"label":a.subject,"identity":"source-bound; matching names are not merged"}));
        nodes.push(json!({"id":to,"label":a.object.as_deref().unwrap_or(&a.proposition),"identity":"source-bound"}));
        edges.push(json!({"id":a.id,"source":from,"target":to,"label":a.predicate,"assertion":a,"fingerprint":a.fingerprint(),"review_state":review_state,"review":receipt,"truth":"attributed assertion, not independent fact"}));
    }
    Ok(Json(
        json!({"nodes":nodes,"edges":edges,"ontology_version":ONTOLOGY_VERSION,"projection_rebuildable":true,"tool":tool,"selection":{"known_as_of":known,"valid_at":filter.valid_at,"actor_label":filter.actor,"review_state":filter.review_state,"time_semantics":"All exact evidence revisions must be available/published by the knowledge cutoff and valid at the selected world time. Unknown validity remains unknown; freshness is checked against current provenance."}}),
    ))
}

#[cfg(test)]
mod domain_graph_filter_tests {
    use super::*;
    #[test]
    fn selection_rejects_future_cutoffs_arbitrary_tools_and_fake_verified_states() {
        for query in [
            json!({"known_as_of": 101}),
            json!({"tool": "imported-plugin"}),
            json!({"review_state": "verified"}),
            json!({"actor": "x".repeat(201)}),
        ] {
            assert!(serde_json::from_value::<GraphFilter>(query)
                .unwrap()
                .validate(100)
                .is_err());
        }
        assert!(
            serde_json::from_value::<GraphFilter>(json!({"endpoint": "https://example.com"}))
                .is_err()
        );
        let filter = GraphFilter {
            tool: Some("rights".into()),
            known_as_of: Some(50),
            valid_at: Some(40),
            ..Default::default()
        };
        let (known, tool) = filter.validate(100).unwrap();
        assert_eq!(known, 50);
        assert_eq!(tool.unwrap().id, "rights");
    }
}
