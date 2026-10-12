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

Implemented in `edgequake/crates/tacitus-*`: source/temporal/span/assertion/review/local
identity invariants; selected UTF-8 import; source-scoped evidence compiler; deterministic
bounded capsule ZIP validation, references/included modes, imported-attestation downgrade,
fresh-workspace proposal rebinding and developer preview/pack/inspect/compile CLI;
decision-brief proposal execution through a model trait, hypothesis matrix validation,
commitment records and dependency impact closure; comparable evaluation records.

Tested: 25 deterministic Rust tests; scoped Clippy; schema consistency; substantive
three-source synthetic file CLI export/import/fresh-workspace lexical compilation.
Three fictional dossiers contain 15 intentional CC0 documents. No live inference was used.
Fresh local-checkout library/schema rehearsal passed; application-volume rehearsal did not run.
See `docs/tacitus/verification.md` for exact commands, failures and limits.

These are not API security enforcement or persisted UI workflows until integrated.
No model adapter is wired. The compiler's file tool is lexical only; semantic input is
a library interface, not a tested EdgeQuake retrieval integration.

## Milestones 2–7

Not accepted. Source-built branded UI/API, persistent canonical storage, authentication
integration, ingestion/extraction, hybrid context retrieval, four end-to-end analysis
workflows, portable capsule UI, evaluations, browser proof, clean-clone/offline rehearsal
and final release verification remain unimplemented or unverified.

No real benchmark results exist. Human ratings are pending; external model arms NOT RUN.
No cloud services, tunnels, existing Tacitus/Praxis repositories or data were modified.

## Publication and next step

The public upstream fork exists; Tacitus commits currently remain LOCAL. New pushes are
withheld pending fingerprint-level triage of 35 inherited selected-pin history findings.
The full upstream dependency audit also reports RSA RUSTSEC-2023-0071; no readiness claim.
The first unfinished milestone remains the real upstream local baseline (milestone 1),
then source-built Tacitus API/frontend integration (milestone 2). No milestone is falsely
marked accepted on the strength of library tests.
