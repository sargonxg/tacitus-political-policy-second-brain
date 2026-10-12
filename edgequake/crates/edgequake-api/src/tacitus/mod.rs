//! Persistent policy extension, using the existing authentication, retrieval and database.
pub(crate) mod analysis;
pub(crate) mod capsules;
pub(crate) mod memory;
pub mod openapi;
pub mod provider_policy;
pub(crate) mod sources;
mod store;
use crate::state::AppState;
use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/tacitus/status", get(sources::status))
        .route("/tacitus/sources", get(sources::list).post(sources::sync))
        .route(
            "/tacitus/sources/{id}",
            get(sources::inspect).patch(sources::metadata),
        )
        .route("/tacitus/sources/{id}/replacement", post(sources::replace))
        .route(
            "/tacitus/analyses",
            get(analysis::list).post(analysis::submit),
        )
        .route("/tacitus/analyses/{id}", get(analysis::get))
        .route("/tacitus/analyses/{id}/cancel", post(analysis::cancel))
        .route("/tacitus/graph", get(memory::graph))
        .route("/tacitus/impact", get(memory::impact))
        .route("/tacitus/methods", get(memory::list).post(memory::propose))
        .route("/tacitus/methods/suggest", post(memory::suggest))
        .route("/tacitus/reviews", post(memory::review))
        .route("/tacitus/capsules/preview", post(capsules::preview))
        .route("/tacitus/capsules/export", post(capsules::export))
        .route("/tacitus/capsules/import", post(capsules::import))
        .layer(DefaultBodyLimit::max(12 * 1024 * 1024))
}

/// Single API-instance MVP: old unfinished requests become retryable interrupted records.
pub fn recover(state: &AppState) {
    if let Some(pool) = state.pg_pool.clone() {
        let boot = chrono::Utc::now();
        tokio::spawn(async move {
            if sqlx::query("UPDATE tacitus_analyses SET status='interrupted',error='Server restarted before completion; submit a new analysis',updated_at=now() WHERE status IN ('queued','retrieving','generating') AND created_at < $1")
                .bind(boot).execute(&pool).await.is_err(){tracing::warn!("Policy recovery unavailable; verify migration 170");}
        });
    }
}
