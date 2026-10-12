# Implementation status

## Persistent MVP candidate — verification pending

The `codex/policy-mvp-cloud-02` continuation adds native policy records and explicit
capsule v2 migration, PostgreSQL migration 170, authenticated API routes, bounded
context compilation, three policy harnesses, real Ollama generation, local-provider
guards, source-bound review/memory/dependency handling, an Ask workbench and CLI.
The tracked marketing app is removed; the EdgeQuake application remains.

**No build, test, inference, database or browser execution has run for these changes.**
The user requested code first and all verification on a cloud host chosen later.
Schema v2 was generated as a source artifact; that is not a compatibility test.
See [MVP instructions](docs/tacitus/policy-mvp.md) for authored verification commands,
remaining acceptance gaps and audit boundaries. The first unfinished milestone is
compiling this integration on the chosen host, then its real ingestion/model baseline.

The counts below belong only to the earlier checkpoint and are not current proof.

This repository is **not a functioning Tacitus application yet**. The upstream app is retained;
Tacitus library work does not imply an integrated or accepted runtime.

## Continuation verified on 2026-10-12

- Windows/Node 24.12.0: 81 existing lab tests plus 7 self-hosting launcher tests pass.
  Publication review caught Compose interpolation of downloader shell variables;
  the new regression failed before dollar escaping was fixed and all 88 tests pass.
- Rust 1.95.0: 28 native tests pass; scoped Clippy and formatting pass. Three new
  regressions first failed, then passed after fixes for imported method acceptance,
  method provenance/identity and assertion/source workspace binding.
- Actual Chrome HTTP navigation to http://127.0.0.1:4317 works. Fictional dossier,
  exact-source inspection, earlier/later time selection, novel selected text, proposed
  memory and dependency impact were exercised. This is lab acceptance, not Next.js E2E.
- `node scripts/tacitus-lab-roundtrip.mjs` runs the real authoritative Rust
  preview/pack/validate/import/fresh-workspace compile against lab-generated input;
  four exact versions/citations, qualifications and common-source provenance survive.
- `scripts/tacitus-stack.mjs` provides source-build local/cloud-VM installation commands.
  Seven contract tests and official Compose schema validation pass; Docker execution is
  NOT RUN. See [self-hosting](docs/tacitus/self-hosting.md).
- Clean detached local checkout at `f4d9b5eafc096f850417b24f4e7761627f951bf7`
  reproduced 87 Node tests, 28 Rust tests, native lab round-trip and schema consistency
  with cached dependencies. Its tracked working tree stayed clean. This is not a
  clean-volume Docker/application rehearsal or a remote-clone build.
- All 35 inherited selected-pin scan fingerprints were inspected and recorded in
  [the redacted triage ledger](docs/tacitus/inherited-audit-triage.json). Seven insecure
  example findings remain open. No findings were waived or allowlisted. The inherited
  RSA advisory and full runtime acceptance still block release.

The user declined installation/model downloads on this laptop (15.7 GiB RAM,
approximately 2 GiB free RAM and 9 GiB free disk) and chose self-hosting preparation
for a host selected later. No Docker/Ollama/models, paid resources or cloud deployment
were installed/created. PostgreSQL persistence, actual inference, API/provider enforcement,
integrated policy UI and the full persistent dossier-to-memory workflow remain unfinished.
The first unfinished milestone is still the real source-built application/model baseline
on a suitable host. No model identity, inference result or serving application is claimed.

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

The Tacitus checkpoint and intentional synthetic fixtures are published on `main` in
https://github.com/sargonxg/tacitus-political-policy-second-brain. Publication was explicitly
requested; it does not certify the unfinished application or clear inherited audit findings.
Fingerprint-level triage of 35 inherited selected-pin history findings is now recorded;
seven insecure-example findings remain open and publication of a draft implementation
branch does not waive the release gate.
The full upstream dependency audit also reports RSA RUSTSEC-2023-0071; no readiness claim.
The first unfinished milestone remains the real upstream local baseline (milestone 1),
then source-built Tacitus API/frontend integration (milestone 2). No milestone is falsely
marked accepted on the strength of library tests.
