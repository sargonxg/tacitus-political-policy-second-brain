# Implementation status

This repository is **not a functioning Tacitus application yet**. The upstream app is retained;
Tacitus library work does not imply an integrated or accepted runtime.

## Milestone 1 — repository and live upstream baseline

Implemented: isolated public fork under verified `sargonxg`; upstream history/license;
correct remotes; explicit gh default; development branch at v0.34.0; Actions disabled.

Inspected: upstream runtime, workspace, migrations, providers, query context contract,
frontend package manager and tests. See UPSTREAM.md.

Blocked: Docker/Compose and Ollama are not installed; no WSL distribution was listed.
Live ingestion, embeddings, graph extraction, retrieval, generation and source references
have **not run**. Baseline acceptance is incomplete.

## Independent library work

Domain/temporal/source-span/review/local-identity invariants are being implemented in
`edgequake/crates/tacitus-*`. These are not API security enforcement until integrated.
Test results will be recorded in `docs/tacitus/verification.md`.

## Milestones 2–7

Not accepted. Source-built branded UI/API, persistent canonical storage, authentication
integration, ingestion/extraction, hybrid context retrieval, four end-to-end analysis
workflows, portable capsule UI, evaluations, browser proof, clean-clone/offline rehearsal
and final release verification remain unimplemented or unverified.

No real benchmark results exist. Human ratings are pending; external model arms NOT RUN.
No cloud services, tunnels, existing Tacitus/Praxis repositories or data were modified.
