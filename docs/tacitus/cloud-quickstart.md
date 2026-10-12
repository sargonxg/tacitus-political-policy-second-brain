# Test Tacitus on a cloud VM

The implementation is on `main`. This guide runs it on a VM you own; it does not
provision paid resources. Cloud execution is still unverified. An explicit failed
check is a defect to fix, not evidence that the MVP works.

## Host

Use a Linux x86-64 VM with approximately 8 vCPU, 32 GiB RAM and at least 100 GiB
free disk. A preconfigured Docker VM is convenient. Install Git, Node 22 or later,
Docker Engine with Compose v2, and allow your account to use Docker. Source checks
use their own container with Rust 1.95 and pnpm; no host Rust/pnpm installation is
needed for that route. The default stack uses CPU inference. GPU acceleration is
not configured automatically, even if you select a GPU VM.

Keep application ports closed to the Internet. Use an SSH tunnel for browser access.
Do not run these builds/model downloads on the storage-constrained laptop.

## Build and start — run on the VM

```sh
git clone https://github.com/sargonxg/tacitus-political-policy-second-brain.git
cd tacitus-political-policy-second-brain
git switch main
git rev-parse HEAD
node scripts/tacitus-stack.mjs doctor
node scripts/tacitus-stack.mjs init cloud
node scripts/tacitus-stack.mjs check
node scripts/tacitus-stack.mjs check-source
node scripts/tacitus-stack.mjs build
node scripts/tacitus-stack.mjs download
node scripts/tacitus-stack.mjs up
node scripts/tacitus-stack.mjs status
curl --fail http://127.0.0.1:8080/health
node scripts/tacitus-stack.mjs models
```

`init` creates private random credentials in `.tacitus/stack/config.json` and refuses
to overwrite an existing file. Run it once. The source-check container downloads build
dependencies and runs native/API/frontend checks and the frontend audit; any failed
step returns an error. These commands have not been executed on a cloud host yet.
The inherited audit gate remains open regardless of a successful startup.

Builds can take substantial time and disk space. Model downloads are explicit and
must run while runtime Ollama is stopped. Keep the default ports 3000/8080 for the
acceptance runner. Database/model volumes survive `down`; never delete them to make
a persistence test appear to pass.

If a step fails, inspect private diagnostics:

```sh
node scripts/tacitus-stack.mjs logs
```

The API requires the migration service and database/model health checks. A healthy
API alone does not prove ingestion, embeddings or generation. Do not share raw
configuration or logs containing private evidence or credentials.

## Open the browser — run on your laptop

```sh
ssh -N -L 3000:127.0.0.1:3000 -L 8080:127.0.0.1:8080 YOUR_USER@YOUR_VM
```

Open http://localhost:3000/login. The username is `admin`; obtain the generated
`adminPassword` privately from the VM's config file. If credentials have been rotated,
use the existing password. Select a workspace, upload public text/Markdown, wait
for ingestion, then ask a question and open its citations. Start with public synthetic
material while release/security verification remains unfinished.

## Real acceptance and restart — run on the VM

```sh
TACITUS_CLOUD_EXECUTE=1 node scripts/tacitus-cloud-test.mjs
node scripts/tacitus-stack.mjs restart
TACITUS_CLOUD_EXECUTE=1 node scripts/tacitus-cloud-test.mjs after-restart
```

The helper logs in privately with the generated bootstrap configuration. It renews
the 15-minute session when needed without extending server token lifetimes or printing
tokens/passwords. If the bootstrap password has changed, supply `E2E_PASSWORD` using
a private session environment. Optionally select an existing tenant with `TACITUS_TENANT`.

The first run creates synthetic workspaces and new random-marker evidence. It checks
real ingestion/retrieval/generation, three policy harnesses, exact passages, human-bound
method review, native capsule export/import, fresh-workspace reuse, source-text changes
and affected dependencies. Restart checks reuse the saved receipt and persisted data.
A full successful receipt is saved privately in `.tacitus/cloud-acceptance.json`; the
runner refuses to overwrite it. A fresh rehearsal needs a separately chosen clean
workspace/checkout, not deletion of existing data.

Browser automation, actual offline-network verification, model/extension identities,
clean-volume rehearsal and unresolved audit findings remain separate acceptance gates.
See [the detailed MVP guide](policy-mvp.md) for browser commands and evidence requirements.
No efficiency or superiority claim is established by this smoke test.
