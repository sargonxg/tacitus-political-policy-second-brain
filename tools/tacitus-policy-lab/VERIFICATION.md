# Verification — Policy Lab 0.1

## Windows continuation, 2026-10-12

Reproduced all 81 lab tests on Node 24.12.0. All 28 native Rust tests pass with Rust
1.95.0 after three new capsule trust/provenance regressions were fixed. Scoped Clippy
and Rust formatting pass. `node scripts/tacitus-lab-roundtrip.mjs` completed real native
preview/pack/validate/import/compile in a fresh workspace using this lab's generated
inputs: four exact source texts/hashes/citations, qualifications and common origins
preserved. Schema generator consistency passes; v1 wire fields were not extended.

Actual Chrome navigation to http://127.0.0.1:4317 worked. The HTTP-served UI exercised
fictional dossier loading, exact passage inspection, earlier/later cutoff selection,
novel selected text, proposed memory and dependency impact. No embedded assets or test
cryptography were substituted. This is exercised lab HTTP behavior, not an automated
full-application E2E suite. The preceding blocked-browser result remains historical.

Self-hosting launcher: 6 tests pass; generated JSON validates against the official
Compose schema. User declined Docker/Ollama/model installation due to laptop capacity,
choosing a host later. API/frontend builds, migrations, persistent database, local
inference/embedding model identities, offline/restart application proof and full
policy-workflow E2E remain NOT RUN. No private Praxis compatibility is claimed.

## Previous build environment

Inspected repository base: `edd78305254a367686bd9e3fa4bd118dc207e05d`.
Environment actually used: Linux, Node v22.16.0; no npm dependencies installed.

## Executed successfully

1. `node --test tools/tacitus-policy-lab/test.mjs`: **81 tests passed, 0 failed**.
   Source hashes/strict wire shapes, UTF-8 spans, attributed proposals, temporal and
   source restrictions, byte budgets, memory guards, typed relations, trusted handlers,
   deterministic receipts, capsule input rights and negative cases are exercised.
2. Real loopback HTTP server: static asset allowlist, inaccessible arbitrary paths,
   blocked write method, Host rejection, health metadata and security headers.
3. `node tools/tacitus-policy-lab/cli.mjs demo`: real synthetic context/brief preparation;
   four source statements selected, later correction omitted at the earlier cutoff.
4. `demo --out NEW_DIRECTORY`: actual native manifest/data/evidence files created;
   repeating the command refuses overwrite. Temporary test data is synthetic.
5. JSON Schema Draft 2020-12 validation of generated native manifest/data. The inspected
   public schema was reconstructed, serialized identically, and its Git blob verified
   as `86ee68b3385524a839037eff810d526585550d53` before validation. This establishes
   structural conformance only, not Rust semantic/ZIP acceptance.
6. **12 isolated browser-component checks:** fixture/context; time-filtered matrix;
   exact passage inspector; guarded memory; later correction; hypothesis diagnostics;
   dependency closure; a newly generated random source and inert HTML; session
   download/reimport; references-only input download; narrow layout; no script errors.

The checked-in test file's Git blob is `b9bd91ecb79b2a823215745b589103e159cf4192`.
Runtime JS/HTML/CSS byte hashes were matched to their GitHub blobs. A syntax error in
an uncommitted draft test was fixed before the successful run and branch publication.

## Browser qualification

Actual navigation to the live localhost server returned
`net::ERR_BLOCKED_BY_ADMINISTRATOR`. No browser or network security policy was changed.
Browser interactions therefore ran with embedded local assets in an isolated about:blank
component test. That context lacks WebCrypto, so the test used a SHA-256/UUID bridge.
Real WebCrypto operations are independently exercised by the Node suite.

This verifies the exercised component logic, not actual browser HTTP navigation,
module loading, CSP enforcement, browser/server integration or the full application.
Any screenshot is labeled **component preview**, not full EdgeQuake execution proof.

## Not run / not claimed

Cargo/native Rust tests or ZIP pack/import/compile; Docker source build; PostgreSQL/AGE;
Ollama/local LLM generation; authenticated EdgeQuake API; Next.js build; real browser
HTTP E2E; Windows/macOS acceptance; genuine model benchmarks or frontier comparisons.

The earlier repository's 25 native test results were read, not reproduced. Actions and
the existing audit gate were not changed; no CI-pass claim is made. Review the unresolved
inherited findings before production acceptance. Main and production systems are untouched.
