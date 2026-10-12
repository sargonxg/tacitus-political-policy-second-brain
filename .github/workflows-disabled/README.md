# Inherited workflow quarantine

The 24 upstream YAML workflows are retained here for attribution and future audit.
GitHub does not execute them from this directory. They include publication, releases,
deployments and heavyweight/credential-dependent integration jobs.

The only new workflow under `.github/workflows` is read-only scoped library CI,
with a source-verified pinned checkout action and no model downloads. Repository Actions
remain disabled, so no CI pass is claimed. Enable only after the publication/history
review and explicit project CI readiness.
