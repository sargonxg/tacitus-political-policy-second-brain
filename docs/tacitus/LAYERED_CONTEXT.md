# Rust layered policy context

This additive module extends the existing tacitus-analysis crate using existing native policy SourceRecord and PolicyAssertion types. It does not modify their schema or imply compatibility with the separate Node v1 catalog, PRAXIS attachment format, or DIALECTICA v3.

## Runnable path

Requires Rust 1.95.0 or compatible newer toolchain. This isolated source workspace preserves the existing three crate manifests and adds no runtime dependency.

From the full repository root, first run `cd edgequake`. The equivalent commands in the delivered isolated three-crate workspace run from its root. Only the isolated three-crate test suite has been verified; full repository tests and clippy have not been run.

```sh
cargo test -p tacitus-core -p tacitus-capsules -p tacitus-analysis
cargo run -q -p tacitus-analysis --example layered_demo > dossier.json
cargo run -q -p tacitus-analysis --example layered_context < dossier.json > context.json
```

The example creates newly authored fictional statements under CC0. The CLI reads only explicit bounded stdin JSON; it does not crawl files or call any model or network service.

## Contract

Input: request {question,workspace,known_as_of,valid_at,selected_sources,max_bytes}; sources keyed by native source ID@revision; assertions keyed by ID; graph sidecar {version:"tacitus-policy-graph-sidecar/1",available_at,claim_fingerprints,nodes,links,knowledge}.

Each graph claim reference is bound to the full assertion fingerprint, including revision and evidence spans. Nodes are explicit actor/institution/event annotations with supporting claim IDs; an event may have an explicitly supplied event_time. Links are explicit contradiction, supersession or dependency annotations with rationale. No identity matching or relation extraction is inferred. Graph annotations cannot be used before their declared availability checkpoint.

Output: {packet,digest,bytes,budget_unit}. Packet format tacitus-layered-policy-context/1 includes source-cited claims, provenance-linked entity/event nodes, claim relationships, an omission record for every excluded claim and warnings. Claims retain native speaker, predicate, modality, conditions, exceptions and epistemic type. Knowledge defaults to Unknown; ReportedTrue/ReportedFalse are explicit analyst labels, not verified truth or inferred negation.

The entire packet is deterministic serde_json UTF-8 serialization within the supplied byte budget, including omission records. Digest binds those exact bytes; this is not RFC8785/JCS or a portable signature scheme. The outer digest/byte-count wrapper is not included in the packet budget. No tokenizer equivalence is claimed.

Omission categories are deliberately combined diagnostics; they do not identify an exact causal exclusion reason.

Connected contradiction/correction/dependency groups are all included or all omitted. They are never silently split or resolved. Missing references, changed fingerprints, duplicate links, tampered sources and insufficient omission-receipt budgets fail closed. Withdrawn, restricted, temporally unavailable or nonredistributable evidence excludes its connected group. Source rights are declarations needing independent verification.

## Integration boundary

This is an in-memory graph-backed exporter and deterministic Rust library/CLI. No live PostgreSQL/AGE adapter, authenticated access layer, native capsule graph migration, hosted marketplace, LLM provider or tool execution is included. The caller must supply an authorized current snapshot and recompile after revocation. Exported offline copies cannot be recalled. Content remains untrusted data; downstream models can still mishandle instructions quoted in evidence, so these labels are not a prompt-injection guarantee.

Native capsule unit/roundtrip tests ran in this isolated workspace. That does not establish an end-to-end Node-catalog-to-Rust ZIP adapter or production PRAXIS compatibility.
