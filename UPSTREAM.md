# Built on EdgeQuake

This project is an independent derivative of [EdgeQuake](https://github.com/raphaelmansuy/edgequake). No upstream endorsement is implied. Apache-2.0 licensing and upstream history are retained.

- Selected stable release: **v0.34.0**, not a prerelease.
- Exact upstream commit: **c994bc89c95fd44e191e4f015a2af0e84a27244f**.
- Selection date: 2026-10-11.
- Workspace requires Rust 1.95; installed compiler is 1.95.0.
- Frontend: pnpm 10.13.1, committed pnpm lockfile, Next 16.3.3, React 19.2.3.
- Build compatibility is **not verified**: Docker is unavailable. This is an inspected candidate pin, not an accepted baseline.

## Inspected integration surfaces

`edgequake/Cargo.toml`, `edgequake/Cargo.lock`, root README/AGENTS/LICENSE,
`docker-compose.quickstart.yml`, backend/frontend/unified PostgreSQL Dockerfiles,
`edgequake/docker/extension-pins.sh`, migration inventory, provider role/Ollama guides,
API query types/handlers/routes, API test inventory, frontend package/lockfile/layout/config.

`POST /api/v1/query/context` is the source-inspected context retrieval route.
`POST /api/v1/query` also accepts `context_only: true`, with deprecation headers pointing
to the newer route. HTTP execution remains unverified. `only_need_context` is not the contract.

Upstream role resolution permits saved connections and silent fallthrough. The MVP
continuation connects Tacitus local-only guards to API/provider paths and adds real
Ollama generation. Source checks indicate intended rejection of saved connections,
unapproved endpoints/models, cloud and mock providers; runtime enforcement remains
**unverified**. See [cloud acceptance](docs/tacitus/cloud-quickstart.md).

Upstream default PG18 extension pin uses Apache AGE `PG18/v1.8.0-rc0` and pgvector
`v0.8.5`. Do not represent these as verified stable images. Registry manifests,
extension loading, dimensions, and migrations must be tested before runtime acceptance.

## Update procedure

Fetch upstream tags explicitly; create an update branch; inspect release notes and
provider/query/migration changes; run the unmodified live baseline in fresh named volumes;
then run Tacitus contract, security, capsule round-trip and live regression suites.
Record exact SHAs and image/model digests. Never move the runtime pin automatically.

## Workflow quarantine

Inherited workflows include releases, GHCR/SDK/registry publication, GCP and website
deployment, secret-dependent and heavy integration jobs. Actions were disabled at the
new fork repository level before Tacitus pushes. All 24 inherited workflow files are
also quarantined under `.github/workflows-disabled` on the local Tacitus branch.
The new scoped library CI uses `permissions: contents: read`, a pinned checkout action
and no model downloads. Repository Actions remain disabled; CI has NOT RUN.
Enabling inherited workflows is not part of normal local setup.
