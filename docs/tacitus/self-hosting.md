# Self-hosting on a local machine or cloud VM

This is a source-build installation candidate for the retained EdgeQuake application,
not acceptance of the complete Tacitus policy workflow. Configuration/unit/native-library
checks have run; Docker startup, database migrations, local inference, persistence and
application browser E2E have not. The inherited release/security gate remains blocked.

Use a separate machine. The inspected laptop has 15.7 GiB RAM, approximately 2 GiB free
RAM and 9 GiB free disk. The user declined installations and model downloads on it.
For source builds, provision approximately 32 GiB RAM and 100 GiB free disk. These are
planning recommendations, not measured minimums. A GPU is optional; CPU generation
can be slow. Check model licensing, available RAM and measured inference before sizing
a service for other users. Do not provision paid resources without approval.

## One installation path for both environments

Prerequisites on the chosen host: Git, Node 22+, Docker Engine/Desktop with Linux
containers and Compose v2. Follow the vendors' installation instructions. This launcher
does not install software, enable WSL, provision VMs, open firewall ports or alter Actions.

Clone the existing repository and select the reviewed implementation revision. Do not use
the inherited upstream image as proof that Tacitus changes are running. From that checkout:

```sh
node scripts/tacitus-stack.mjs doctor
node scripts/tacitus-stack.mjs init local
# On a VM, use init cloud instead. The security/network topology is identical.
node scripts/tacitus-stack.mjs check
node scripts/tacitus-stack.mjs build
node scripts/tacitus-stack.mjs download
node scripts/tacitus-stack.mjs up
node scripts/tacitus-stack.mjs status
```

`doctor` is read-only and returns exit 2 for missing Compose or insufficient free
resources. `init` writes new random credentials to ignored `.tacitus/stack/config.json`
and refuses to overwrite it. Keep this file private. The bootstrap username is `admin`;
read the password locally from `adminPassword`. Never paste it into issues or support logs.
POSIX files use mode 0600; on Windows also protect the directory with your user ACL.
Existing credentials survive restart. A replacement config cannot change credentials
already stored in an existing database; backup/rotation require separate approved steps.

`check` validates without printing the rendered configuration (which contains secrets).
`build` builds PostgreSQL extensions, API and Next.js from this checkout. Software
downloads happen during image builds. `download` explicitly downloads the selected
models in a separate one-shot networked container using the shared model volume.
Run it only while the runtime Ollama container is stopped; do not modify model files
concurrently with inference. Runtime `up` never builds or pulls images automatically.

The candidate uses PostgreSQL 16, pgvector v0.8.5 and the existing upstream AGE
`PG16/v1.6.0-rc0` pin. The AGE tag is a release candidate; its loading/compatibility is
unverified here. Do not describe this as an accepted stable database stack. A one-shot
`edgequake migrate` service must succeed before API startup. Database and model data use
named persistent volumes. No parallel database, authentication service or knowledge kernel
is introduced. `down` preserves volumes; there is deliberately no delete-volume command.

Ollama is pinned to 0.40.3 (official release checked on 2026-10-12). Initial model choices
are `qwen3:4b` and `embeddinggemma:latest`; tags are not immutable model digests. Record
the actual digests/context/dimensions after download. These models have not been downloaded
or measured here. Vision/scanned PDFs are not accepted with the text-only chat model.
The separate upstream decision engine is disabled until separately verified; normal
extraction, query and embedding roles select Ollama. Do not change embedding models on
an existing index without a validated rebuild/migration.

## Access locally or through SSH

Local browser: http://localhost:3000. API: http://localhost:8080/health.
These are intended URLs; the full application has not been started on this laptop.
PostgreSQL and Ollama publish no host ports. API/frontend ports bind to 127.0.0.1.

For a cloud VM, keep inbound application ports closed and connect from your workstation:

```sh
ssh -N -L 3000:127.0.0.1:3000 -L 8080:127.0.0.1:8080 your-user@your-vm
```

Then open the same localhost URLs. This is self-hosted inference on your VM, not a cloud
model API. Multi-user Internet hosting requires a separately reviewed TLS/reverse-proxy,
origin, tenant isolation, backup and authentication configuration; it is not enabled by
changing a label to `cloud`. The VM owner controls its costs and access.

## Local-only boundaries and outstanding checks

API, migrations, database and runtime Ollama attach only to a Docker `internal` network.
Ollama receives `OLLAMA_NO_CLOUD=1` in its server environment. Model download uses a
different network and is absent from normal runtime startup. Provider keys, proxy settings,
ambient Compose overrides and `.env` files are not passed to the launcher/container spec.
Docker commands explicitly select the local default context. Frontend has its own bridge
network and reaches the API through the browser's loopback address, not a shared API network.

This is intended network defense, not proof that every upstream provider/request/saved
connection path is constrained server-side. That integration remains unfinished. Saved
connections and role overrides must be inspected/rejected before importing private data.
Do not advertise a local-only/verified product badge based on this generated configuration.

On the chosen host, verify and record:

1. Source commit and source-built image IDs; Docker/Compose/OS/GPU identities.
2. Loaded AGE/vector extensions and successful migration ledger.
3. Authenticated login, workspace isolation and rejection of cloud/mock/connection overrides.
4. `docker compose` inspection of Ollama's actual environment, server log
   `Ollama cloud disabled: true`, model digests, dimensions and request identities.
5. A new random-marker document: real extraction, embeddings, retrieval, exact citations
   and a cited generated answer. Model failure must fail visibly without substitutions.
6. Restart and offline repeat after downloads, including the same persisted source/version.
7. Full policy workflow, native export/fresh-workspace reuse and changed-evidence review.
8. Actual browser E2E, clean-volume rehearsal and disposition of inherited audit findings.

```sh
node scripts/tacitus-stack.mjs restart
node scripts/tacitus-stack.mjs down
```

These commands have contract tests only. No background application progress is claimed.

## Checks that run without application installations

```sh
node --test tools/tacitus-policy-lab/test.mjs scripts/tacitus-stack.test.mjs
node tools/tacitus-policy-lab/cli.mjs serve
node scripts/tacitus-lab-roundtrip.mjs
```

The lab serves at http://127.0.0.1:4317. It is a development tool, not a second production
application. Native rehearsal requires Rust 1.95 and cached locked dependencies; it uses
the authoritative Rust CLI and new ignored output directories. It performs real native
pack/import/compile, but lexical context compilation is not local inference or persistence.

References: [Ollama local-only FAQ](https://docs.ollama.com/faq),
[Compose networking](https://docs.docker.com/compose/how-tos/networking/),
[Ollama release](https://github.com/ollama/ollama/releases/tag/v0.40.3).
