# Create a native-v2 research capsule locally

This adds a conservative authoring path for original assistant-authored research summaries and synthetic demonstration notes. It reuses the existing native `manifest`, `data`, `evidence` JSON envelope and schema version 2. It does not introduce a capsule format or convert native-v1.

The included fictional cooling-pilot record contains one explicitly synthetic hypothesis and a nonempty exception. No real pilot, participants, outcomes or publisher text are asserted. Source dates describe authorship/availability of the note; `valid_from` and `valid_until` remain null. A date-verified request refuses this capsule. The disclosed-unknown review retains the hypothesis, exception, citation and temporal warning.

## Run from the repository's edgequake directory

```
cargo build --locked -p tacitus-capsules --example author_native_v2
cargo build --locked -p tacitus-analysis --example v2_workflow
python3 check_authoring.py
```

The script expects `authoring-demo/` beside it and `target/debug/examples/` by default. Pass a binary directory as its first argument for a custom Cargo target directory. It uses a temporary output directory, verifies refusal to overwrite an existing archive, inspects the new capsule, generates JSON and text reviews, and checks strict date-query rejection. Its results are saved in `evidence/`. An existing output path is never overwritten by the authoring executable.

Manual creation:

```
target/debug/examples/author_native_v2 --confirm-original-evidence authoring-demo/synthetic-input.json new-capsule.zip
target/debug/examples/v2_workflow inspect new-capsule.zip
target/debug/examples/v2_workflow review-text new-capsule.zip < authoring-demo/request.json
```

Review the input before declaring that the embedded evidence is original, authorized summary text. The flag is an explicit caller declaration, not proof or an external sharing approval. Do not use it for extracted publisher text, uncertain ownership, or arbitrary uploads. General reuse licensing is not inferred. This local tool does not publish, contact anyone, fetch URLs, execute imported content, call models or grant review authority.

## Validation boundary

- Strict JSON rejects duplicate decoded object keys before typed parsing and applies existing byte/depth/node bounds.
- Author and exact summary-extractor metadata must identify original summary records; the source original hash must match its summary text hash. Publisher facts/URLs belong in attributed summary text or hash-bound research metadata, and are not authenticated by this check.
- Sources must explicitly declare redistribution permission and a nonempty license/rights description. Misaligned authors, missing rights, false redistribution permission, unsupported dates and inconsistent hashes are refused. Software cannot adjudicate whether a declaration is truthful.
- Only unknown world-validity intervals are accepted by this narrow entrypoint. It never fills them from publication or packaging times. Existing native APIs remain available for other explicitly supported source types.
- Source availability/publication and assertion/method creation cannot extend beyond the authored record checkpoint. Existing native source, span and method validators also run.
- Imported reviews, dependencies and precomputed manifest file hashes are refused. Declarative methods may be preserved after validation; they are not executed or accepted as instructions.
- Native-v2 export followed by strict import must preserve all authored data and evidence exactly. The exporter computes manifest file hashes. These are integrity checks, not signatures, source truth, semantic support or expert review.
- Arbitrary artifact strings retain their existing opaque semantics. The full native-v2 review workflow is atomic; this authoring path does not invent a compact sidecar or claim general marketplace ingestion safety.

## Scope of verification

Tests cover the three existing corrected policy examples, null validity roundtrips, deterministic native archives, inspect/cited review, strict date refusal and rejection cases. The synthetic CLI path demonstrates a new record end to end. Full application build, hosted marketplace, UI and live PRAXIS behavior are separate gates. Existing PDFium, Docker/Bun and aggregate version prerequisites are unchanged.
