# Explicit scoped-artifact evidence briefs

Experimental local Rust path, layered on PR9. Native-v1 and native-v2 schemas are unchanged. The new sidecar format is `tacitus-scoped-artifacts/1`; the resulting extractive document is `tacitus-compact-evidence-review/1`.

## What works

The three corrected native-v2 capsules now each produce three fully qualified claims within 32 KiB, including their exact assertions, citations, unknown-validity disclosures, original source titles/locators/dates/license notices, complete manifest limitations, global safety constraints and refresh instructions. The same 32 KiB limit makes the existing full opaque-dossier workflow refuse output. This is an explicit selected-evidence view with supplemental context omitted, not an equivalent full dossier or a better analysis.

Measured JSON payload sizes: heat 28,957 bytes; compute 29,098; humanitarian access 29,680. Actual templated text output is 30,145 / 30,100 / 30,934 bytes. The CLI was exercised against each actual archive and its emitted stdout bytes checked against the limit and expected payload. `json` emits only compact Document bytes, without newline or a pretty-print envelope. Library inspection envelopes are intentionally larger and are not bounded model payloads.

The 8 KiB limit cannot hold all global qualifications and rejects. At 16 KiB the output holds global qualifications and complete omission receipts, but **zero claims**. This is not a usable evidence answer; callers must inspect the claim count. Explicit authored-order priority is used for these measurements. A higher-priority claim may not fit while a later smaller claim does. No semantic saliency, factual accuracy, expert review, compression-ratio parity, model quality or speed superiority is claimed.

## Mapping contract and safeguards

The sidecar binds the exact native archive, artifact strings and assertion fingerprints. Partition completeness is checked over parsed serde_json values; duplicate object keys are not currently rejected. This prototype must not be used as a general arbitrary-untrusted marketplace ingestion validator. Each recognized JSON artifact has an exhaustive, non-overlapping partition. Required rights, limitations, temporal contract, review state, safety constraints, updates and native-adaptation caveats remain global. Complete research claim and research-summary records accompany their exact selected claim. Official source records accompany any referencing claim. Source summary text must match its exact native source revision and evidence.

The recognized three artifact keys are profile-specific. The explicit profile version is `tacitus-policy-research-r2/1`, for content revision 0.1.1. A checked-in recursive field allowlist rejects unknown root or nested fields and object paths. New content revisions or fields require an explicit reviewed profile migration; the profile is deliberately narrow, including the current publisher-date map fields. Unrecognized opaque artifact keys or non-JSON artifacts reject and require the atomic workflow. A mapping remains an analyst proposal: exhaustive mechanical accounting does not prove that a supplemental section is irrelevant or that all natural-language corrections were discovered. Future added fields within a recognized artifact reject rather than silently becoming supplemental. A future migration still requires semantic/content review; recognizing a field name does not establish its truth or relevance.

Declared analytical dependencies are directional: selecting an analysis requires its dependencies, while inspecting an underlying source claim does not require accepting the analysis. Declared correction and contradiction links are symmetric selection groups, and every linked group is retained or omitted together. Recognized claim-to-claim edges in the research graph must appear in the sidecar; unknown claim-relation types reject. No relationships are inferred from prose. The benchmark has dedicated synthetic corrections, contradictions and nonempty exception cases; real 32 KiB outputs contain source findings, not the larger analytical recommendations.

Repeated text may use an explicit exact-text reference to a retained assertion proposition or full source quote. Only byte-identical text is referenced. All other record fields remain inline. Reconstructing the record must reproduce the original canonical serde JSON item hash; no semantic summarization occurs. This is an explicit projection format, not the original research schema. Exact serde byte hashes are not RFC8785 JCS or signatures.

Current access/time checks reuse the complete native-v2 workflow before projection. Revoking any source conservatively blocks the entire export, even if a narrower future projection might not need it. Strict date mode rejects unknown world-validity. Imported review records do not grant authority. Cognitive methods are not applied by this evidence-only path; the original archive remains necessary to inspect their guards and provenance. Omission item indexes resolve through the accompanying mapping. The standalone brief does not reproduce every omitted section name.

## Reproduce

From the isolated three-crate workspace:

    cargo test --offline --locked --all-targets
    cargo run --offline --locked -p tacitus-analysis --example compact_bench -- outputs
    cargo build --offline --locked -p tacitus-analysis --example compact_review
    python check_cli.py

In the repository, first `cd edgequake`; run the patched receipt checker as `python check_compact_cli.py` from that workspace root. Rust 1.95.0, existing dependencies only. `compact_bench` exports the exact embedded corrected fixtures, mapping proposals, requests, bounded payloads and optional bounded text. The separate inspection envelopes support audit, not budget compliance.

Generic local invocation:

    compact_review json ARCHIVE.zip MAPPING.json < request.json
    compact_review text ARCHIVE.zip MAPPING.json < request.json

Requests contain the v2 question/time mode/byte budget, explicit complete priority list, and current host revocation state. The host must authenticate and supply current access/revocation state; archive fields cannot authorize it. No hosted endpoint, database, external model, payments, account creation or production PRAXIS integration is added.

## Verification

80 tests pass in the isolated existing three-crate workspace, including 17 new compact tests. Repeated fixture results are byte-identical. Actual CLI output bounds, strict-time refusal and package-revocation refusal are checked. Renderer mutation, missing temporal disclosure and invalid text-reference reconstruction return errors. Full-repository CI, production browser behavior, semantic truth and model injection resistance are not established.
