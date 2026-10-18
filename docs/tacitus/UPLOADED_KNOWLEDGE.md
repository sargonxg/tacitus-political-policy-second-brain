# Uploaded knowledge capture and reuse (experimental)

This path retains exact UTF-8 document bytes, source/version/hash/spans, attributed policy claims, typed actor/institution/event nodes, and explicit correction/dependency edges inside a native-v2 capsule. Different prompts select different graph-connected evidence with citations, qualifiers and temporal disclosures. The graph is portable structured data in the archive; this does not deploy a graph database.

## Extraction is a distinct step

`propose` derives literal line candidates automatically from hydrated SourceRecord documents. It does not discover entities, infer semantic relations or determine truth. `pack` accepts an extractor's explicit source-bound annotations. The sample includes synthetic annotations to exercise that boundary. Free-form understanding still needs a model/extractor adapter or human annotation and review. Existing EdgeQuake EntityExtractor results require a checked source-span adapter, not an assumed ontology conversion.

All generated structures remain unreviewed proposals. Policy relationships use the existing tacitus-policy/1 vocabulary. Exact graph claim fingerprints bind annotations to their assertion revisions; imported labels do not confer review authority. Uploaded text is never executed. Rights are declarations, not legal determinations.

## Reproduce without a model or service

From the repository root, first prepare the supported isolated workspace:

    python3 scripts/prepare-capsule-workspace.py ../knowledge-workspace
    cd ../knowledge-workspace
    cargo +1.95.0 build --locked --example knowledge_capture

Let F=crates/tacitus-analysis/tests/fixtures/knowledge and B=target/debug/examples/knowledge_capture. Use a fresh output directory and avoid replacing existing output files:

    mkdir knowledge-results
    "$B" propose "$F/documents.json" > knowledge-results/candidates.json
    "$B" pack "$F/capture.json" knowledge-results/policy.zip
    "$B" inspect knowledge-results/policy.zip > knowledge-results/inspection.json
    "$B" reuse knowledge-results/policy.zip < "$F/port.query.json" > knowledge-results/port-context.json
    "$B" reuse knowledge-results/policy.zip < "$F/heat.query.json" > knowledge-results/heat-context.json

The port query retains both the conditional commitment and later withdrawal, including their exception. The heat query retains the separate event with observed_at=90. Both leave publication and world-validity dates unknown. A strict date request refuses unknown validity. A checkpoint preceding the graph snapshot refuses and requires the historical capsule revision; it does not invent an earlier graph. A revoked source refuses the entire output because package metadata is unscoped. No-match and too-small evidence budgets produce explicit omission receipts or errors, never usable-answer claims.

The JSON stdout byte count is bounded by max_bytes, with no appended newline. Errors are stderr/nonzero. `pack` refuses an existing output, including a final symlink. Use argument-array subprocess invocation, not a shell, when integrating this CLI.

## Input and output contracts

`propose DOCUMENTS.json` consumes an array of existing hydrated native SourceRecord objects. `pack CAPTURE.json NEW.zip` consumes `Capture` in tacitus-analysis::knowledge_capture: existing manifest/documents/assertions/graph plus explicit observed_at map, extractor identity and rights declaration. Source evidence must exactly match its original/text digest. No URLs or file references are fetched.

`reuse CAPSULE.zip` reads Query JSON on stdin: question, known_as_of, valid_at, time_mode (`disclose_unknown` or `require_declared_interval`), max_bytes, revoked_sources. It emits the whole Context JSON with archive digest, manifest guards, selected graph packet, exact citations, source metadata, observation times, unknown-time labels and omissions. Selection is lexical/entity matching and explicit link closure. It is not semantic ranking or a measured accuracy improvement. Unmatched claims are accounted for; linked corrections stay together or are omitted together.

Only the recognized uploaded-knowledge graph artifact is accepted. Existing opaque native-v2 editorial packages retain their separate atomic workflow. No native-v1 conversion, Foundry deployment, hosted authentication, payments, model invocation or expert validation is claimed.

This bounded capture profile requires at least one typed entity and one explicit claim relationship. Entity-only extraction remains a proposal, not a completed capture under this profile.

Graph closure is conservatively undirected: any matched member brings its declared linked component. This preserves corrections but can retain more context than a directed causal traversal. Strict-date checks concern selected evidence, not every unrelated source in the archive. The current graph snapshot cannot reconstruct historical states; supply the corresponding historical capsule revision. The manifest contains unscoped global guards and is retained atomically, so any revoked, unavailable or expired source causes refusal rather than partial export.
