//! Persistent-policy endpoints. Native capsule schema is versioned separately.
#[derive(utoipa::OpenApi)]
#[openapi(paths(
 super::sources::status,super::sources::list,super::sources::sync,super::sources::inspect,super::sources::metadata,super::sources::replace,
 super::analysis::submit,super::analysis::get,super::analysis::list,super::analysis::cancel,
 super::memory::propose,super::memory::suggest,super::memory::review,super::memory::list,super::graph::graph,super::memory::impact,
 super::capsules::preview,super::capsules::export,super::capsules::import,
 super::ontology::list,super::ontology::apply
),tags((name="Tacitus policy",description="Authenticated, workspace-scoped persistent policy MVP. Real Ollama runtime verification pending.")))]
pub struct PolicyApi;
