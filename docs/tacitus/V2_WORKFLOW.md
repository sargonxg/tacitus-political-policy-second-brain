# Explicit native-v2 local workflow

This additive Rust adapter connects corrected native-v2 capsules to a local catalog, source inspection, the existing layered context compiler and a cited review document. It is in the existing tacitus-analysis crate and requires its layers module. No new runtime dependencies are added. Native-v1 catalog/brief behavior is unchanged.

## Workflow

1. `LocalCatalog::import_native_v2` validates an archive, exact source evidence and native assertion/method contracts. Valid native-v1 is deliberately rejected before the underlying importer can migrate it.
2. `search` and `inspect` return descriptors with exact archive digest, revision, source-validity gaps, scope, declared rights and unreviewed status. Same identity/version with different bytes is rejected. Reimport cannot clear revocation.
3. `export_review` checks current local revocation state and all source/assertion/method timestamps, then compiles all claims through the existing layers exporter. The adapter currently constructs an empty relation sidecar bound to each assertion fingerprint; editorial graph annotations remain preserved opaque artifacts, not automatically converted into native relations.
4. A review document includes layered citations, complete text-free source metadata, per-claim temporal qualifications, package scope/limits, methods and original artifacts. Imported reviews do not confer approval. The plain-text view includes a complete qualified appendix.

## Time modes

`require_declared_interval` fails when either source world-validity bound is unspecified. Matching finite declared bounds still does not establish truth, legal applicability or independent temporal verification. Explicitly unbounded intervals are conservatively not treated as fully declared finite intervals.

`disclose_unknown` permits unspecified bounds only with document-level and derived per-claim unknown-validity statements. These are added outside immutable assertions, so fingerprints remain unchanged. Null means unfiltered/unknown, never valid at every date. Original publisher dates, summary authorship, availability and forecast/observation metadata remain inspectable.

## Fail-closed boundaries

All packages are atomic in this adapter. Any revoked source, unavailable claim/method, unresolved package dependency, partial layer result or insufficient complete-document budget prevents output. Opaque artifacts therefore cannot reintroduce a revoked or omitted claim. Current revocations are local-owner operations; a future server must add authenticated authorization. Offline exported copies cannot be recalled.

Limits: 8 MiB input archive, 100 catalog entries, 32 MiB serialized retained payload (including manifest, data and evidence), 250,000-byte compact review document, 1,000,000-byte plain-text view. The retained-payload cap is not a process RSS limit; temporary clones and allocator overhead are additional. Digests bind exact serde_json bytes, not JCS or signatures. Text is untrusted data and is never executed. No model, semantic ranking or answer generation runs.

## Run

Rust 1.95 or compatible newer toolchain. In the delivered isolated three-crate workspace:

```
cargo test --offline --locked --workspace
cargo run --offline -q -p tacitus-analysis --example v2_workflow -- inspect capsules/*.tacitus.zip
cargo run --offline -q -p tacitus-analysis --example v2_workflow -- review capsules/nyc-heat-resilience.r2.native-v2.tacitus.zip < examples/nyc-heat-resilience.request.json
cargo run --offline -q -p tacitus-analysis --example v2_workflow -- review-text capsules/nyc-heat-resilience.r2.native-v2.tacitus.zip < examples/nyc-heat-resilience.request.json
```

In the full repository first `cd edgequake`; use selected archive/request paths. Offline commands require dependencies already cached. Full-repository tests, browser UI, persistent database catalog, hosted authorization and live PRAXIS integration are not established.

## Verification

63 isolated tests passed, including 11 new workflow tests, independently reproduced. All three corrected research archives ran through inspect → layers → JSON/plain-text documents, preserving five claims each. Strict date mode refused all three unknown-validity packages. Actual compact document sizes: NYC 51,600 bytes; compute 51,904; humanitarian 52,734.

Fixtures contain explicitly labeled assistant-authored source-linked research, not authenticated publisher originals or expert review. Structural/integrity tests do not establish factual correctness, rights or model behavior. No accuracy, latency or compression improvement is claimed.
