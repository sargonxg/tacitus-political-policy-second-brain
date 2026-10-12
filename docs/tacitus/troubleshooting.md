# Troubleshooting and next live acceptance

- **Docker missing:** `docker version` / `docker compose version` cannot run. Install and
  start a compatible Linux-container runtime through normal host setup. No automatic
  virtualization/security changes are made by project scripts.
- **Cached crate missing:** isolated `test` is intentionally offline. The regular Cargo
  test command permits initial dependency downloads, with the authoritative lockfile.
- **Model missing/service unavailable:** no model adapter is wired yet. The tested analysis
  interface propagates an actionable failure; it does not switch to mock/cloud inference.
- **Context too small:** retain output and framing budgets. Full source omission is explicit;
  compilation errors when no source fits. Actual tokenizer/memory verification is outstanding.
- **Unavailable evidence in capsule:** references-only is not a stand-in for evidence;
  supply an explicitly authorized original before compilation.
- **Capsule invalid:** do not bypass schema/hash/path/resource checks. Imported reviews
  never become locally trusted automatically.
- **Source changed/withdrawn:** library impact closure identifies dependents; actual cache,
  vector/graph invalidation and durable review replacement remain integration work.

Next uncompleted milestone is the unmodified live upstream baseline. Required evidence:
source pin, exact source-built images, compatible PostgreSQL extensions/migrations,
selected local model identities/digests/dimensions/budgets, actual ingestion/vector/graph
retrieval/cited generation, timings and failures. Then integrate Tacitus API/UI/storage.
Fresh volumes, clean-clone startup, restart and egress-restricted offline reuse must be
tested before documenting a supported app startup command.
