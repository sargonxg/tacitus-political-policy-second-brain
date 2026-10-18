# Review-gated extraction proposals (experimental, local operator)

This adapter accepts structured annotations from a human or model through a strict envelope around the existing native `Capture`, `Graph`, `SourceRecord` and `Span` contracts. It performs no inference and evaluates no semantic model. EdgeQuake's existing `ExtractionResult` uses string spans and chunk lineage; those require an explicit exact-byte adapter before admission here. No automatic ontology conversion is asserted.

## Candidate → local review → reusable capsule

`extraction_review inspect PROPOSAL.json` validates the candidate and emits its exact-byte SHA256 plus all review material. Candidates cannot set the rights-confirmation flag or settled true/false graph labels. Every entity has exact Span+quote mentions, and every linked entity claim must have a mention contained in that claim's evidence. Native validation checks source hashes, UTF-8 boundaries, vocabulary, intervals, graph fingerprints and resolved relation endpoints. A proposed supersession must not precede its target's authorship.

The operator examines all claims, mentions, relation interpretations, qualifiers, time uncertainty, extractor provenance, complete source metadata and manifest guards. A local decision must name the exact proposal digest, actor label, workspace, rationale and recording time, and explicitly acknowledge complete review, rights and any correction interpretations. The decision is an operator input, never an imported grant. Model output must not be routed into this local decision step automatically.

`extraction_review promote-local PROPOSAL.json CURRENT_SOURCES.json LOCAL_DECISION.json NEW.zip` creates a process-local acknowledgment, rechecks the current source snapshot, consumes that grant and writes a fresh native-v2 archive. It refuses existing destinations, including final symlinks. Receipt JSON is emitted separately. Extractor provenance and the bounded local acknowledgment also survive in the existing extractor metadata; native expert-review records remain empty, and imported capsules retain untrusted proposal status.

Source array order is immaterial: the snapshot hashes complete records sorted by source revision key. Duplicate identities, removed/added/replaced revisions, changed text, rights, restrictions, withdrawals, availability or any other metadata require a fresh proposal and review. Exact proposal bytes are bound, including whitespace and all output-bearing manifest metadata. Changing the source snapshot between acknowledgment and promotion fails closed. Caller-supplied recording times cannot establish an external trusted clock; future-versus-now checks use the caller's stated checkpoint.

This is local API hygiene, not authentication, signatures, multiuser authorization, tamper-proof review storage or cross-process replay prevention. Actor names are labels. A hostile caller can invoke the acknowledgment API or use the existing lower-level capture API; a hosted service must independently authenticate the reviewer, authorize workspace access, control the current snapshot and keep the review operation out of model-facing tools. No such service is implemented here.

A matching quote proves anchoring, not identity, entailment, correction authority or truth. Human acknowledgment records a reviewed interpretation; later authorship alone does not make a source authoritative. Unknown publication, event and validity values remain unknown. Source rights remain declarations. All selected corrections/dependencies retain the existing conservative context closure.

## Reproduce the synthetic acceptance case

From the repository root, prepare a fresh isolated workspace as in `UPLOADED_KNOWLEDGE.md`, then run:

    cargo +1.95.0 build --locked --example extraction_review --example knowledge_capture
    F=crates/tacitus-analysis/tests/fixtures/extraction
    B=target/debug/examples
    mkdir extraction-results
    "$B/extraction_review" inspect "$F/proposal.json" > extraction-results/proposal-review.json
    "$B/extraction_review" promote-local "$F/proposal.json" "$F/current-sources.json" "$F/local-decision.json" extraction-results/policy.zip > extraction-results/local-receipt.json
    "$B/knowledge_capture" inspect extraction-results/policy.zip > extraction-results/inspection.json
    "$B/knowledge_capture" reuse extraction-results/policy.zip < crates/tacitus-analysis/tests/fixtures/knowledge/port.query.json > extraction-results/context.json

The included decision is a synthetic test fixture, not a real user's completed review. A real uploaded dossier requires a new deliberate local decision after inspecting its complete material. The resulting context preserves the conditional access claim and proposed withdrawal together. The malicious instruction line in the uploaded fixture remains quoted untrusted data and is never executed.
