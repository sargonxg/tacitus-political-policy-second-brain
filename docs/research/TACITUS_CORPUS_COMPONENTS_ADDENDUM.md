# TACITUS: a curated corpus of 100–1000 documents

Addendum to the diplomatic knowledge architecture memo • Source checks: 18 October 2026, 12:34–12:43 UTC

## Decision in brief

Keep the existing EdgeQuake ingestion/retrieval infrastructure, TACITUS Rust trust boundary, and Foundry adapter. Add a versioned corpus registry and an independent lexical candidate path before adding another database. Treat semantic extraction as a reviewable proposal, with exact-source mapping and reversible entity resolution. Select a small, purpose-specific capsule from the corpus for each task.

Four external components were examined against this baseline: **Docling** for document conversion, **Tantivy** for embedded lexical retrieval, **LanceDB** for an optional embedded hybrid-search deployment, and **Graphiti** for temporal/entity-resolution design reference. None is a justified wholesale replacement. None was installed, built, or benchmarked in this research. Recent commits and releases show activity, not reliability or superiority.

The original memo and its 35 proposed evaluation cases remain separate. This addendum records new source inspections and a proposed corpus-scale protocol. It does not turn proposed tests into executed results. The previously reviewed minister fixture demonstrates three bounded CLI cases; it does not demonstrate a 1000-document semantic library. Newer implementation work should be assessed at its own reviewed revision.

## 1. Concrete gaps in the inspected implementation

### A corpus and a capsule have different jobs

The inspected native capsule validators limit a capsule to **32 sources and 256 assertions**, with additional method, review and dependency bounds. These limits make a bounded export defensible. They do not preclude a larger library, but the larger library needs a separate catalog and selection process. A single 1000-document archive is outside this contract. Sources: [native capsule validation](https://github.com/sargonxg/tacitus-political-policy-second-brain/blob/df82de5d33f2debf4ce618b660f76c6d53a39d71/edgequake/crates/tacitus-capsules/src/lib.rs), [native-v2 validation](https://github.com/sargonxg/tacitus-political-policy-second-brain/blob/df82de5d33f2debf4ce618b660f76c6d53a39d71/edgequake/crates/tacitus-capsules/src/policy.rs).

**Proposed integration:** a corpus snapshot identifies eligible document revisions and accepted claims. Retrieval returns stable IDs, evidence spans and scores. A selection/compiler stage follows required qualifications and correction dependencies, then emits one or more bounded capsules. If the necessary closure exceeds a capsule or context budget, report the omission or ask for a narrower task. Partitioning must not sever an exception from its claim.

Document count is an inadequate capacity measure by itself. Record original bytes, extracted bytes, pages, chunks, accepted claims, languages and index sizes. One thousand one-page notes and one thousand scanned reports are different workloads.

### Existing hybrid search has a specific recall boundary

In the inspected EdgeQuake `fuse_vector_and_bm25_chunks` implementation, an empty dense-result list returns before native text search runs. When native full-text search is unavailable, the in-memory BM25 path reranks dense candidates rather than searching the whole corpus. PostgreSQL's native path uses `ts_rank_cd` cover-density ranking, not Okapi BM25. The default sparse fusion ordering is sparse-first; reciprocal-rank fusion is an explicit configuration choice. These observations apply to this function and revision, not to every upstream query path. Source: [sparse retrieval implementation](https://github.com/sargonxg/tacitus-political-policy-second-brain/blob/df82de5d33f2debf4ce618b660f76c6d53a39d71/edgequake/crates/edgequake-query/src/sparse_retrieval.rs).

**Recommendation:** retrieve lexical and dense candidates independently. Union them by stable evidence ID, then fuse ranks with a documented rule. An exact identifier or rare legal citation must remain recoverable when dense retrieval returns nothing. First test a full-corpus lexical scan or the existing PostgreSQL text index. Introduce Tantivy only if the embedded deployment or measured latency requires it. Do not describe candidate reranking as corpus-wide lexical recall.

### Extraction reuse is not evidence identity

EdgeQuake already has useful incremental machinery. Its `ChunkReuseIndex` hashes whitespace-normalized content, rebinds extraction chunk IDs, and in non-page-scoped mode can fall back to positional chunk ID after a hash miss. Page-scoped reprocessing has stricter fail-closed behavior. This is a local class observation; a full caller/configuration audit would be needed to characterize every ingestion path. Source: [chunk reuse](https://github.com/sargonxg/tacitus-political-policy-second-brain/blob/df82de5d33f2debf4ce618b660f76c6d53a39d71/edgequake/crates/edgequake-pipeline/src/chunk_reuse.rs).

**Recommendation:** reuse the extraction work only behind an adapter that revalidates the destination's exact source bytes, revision and span mapping. Bind cache identity to parser, normalizer, chunker, extractor/model, prompt and schema configuration. A whitespace-normalized match is not an exact-byte proof. A changed paragraph with the same positional ID must not inherit review acceptance. Repeated boilerplate may share computation but cannot collapse independent authorship, access policy or attribution.

The existing `EntityExtractor` trait is a natural integration seam. Its results include entity string spans, importance values, source chunk IDs, relationship weights and extraction metadata. These are useful proposal inputs; they do not establish exact relation evidence, calibrated confidence, or institutional approval. Relationship endpoints expressed as names must be resolved before becoming durable actor IDs. Sources: [extractor interface](https://github.com/sargonxg/tacitus-political-policy-second-brain/blob/df82de5d33f2debf4ce618b660f76c6d53a39d71/edgequake/crates/edgequake-pipeline/src/extractor/mod.rs), [result types](https://github.com/sargonxg/tacitus-political-policy-second-brain/blob/df82de5d33f2debf4ce618b660f76c6d53a39d71/edgequake/crates/edgequake-pipeline/src/extractor/types.rs).

## 2. Component assessment, with actual licenses and code fit

The table records the latest published release returned by GitHub at the check time. Code inspections used the separately pinned HEADs below. HEAD features are not assumed to exist in the listed release. All five repositories were unarchived when checked. Licensing statements concern the repository code, not every dependency, model, source document or hosted service.

| Component | License file inspected | Latest release observed | Last repository push observed | Decision |
|---|---|---|---|---|
| Existing EdgeQuake | Apache-2.0 | v0.35.6, 17 Oct 2026 | 18 Oct 2026 | Preserve local pinned integration; no automatic upgrade |
| Docling | MIT | v2.140.0, 16 Oct 2026 | 18 Oct 2026 | Optional conversion adapter after fidelity comparison |
| Tantivy | MIT | 0.26.1, 10 May 2026 | 17 Oct 2026 | Best narrow optional addition for independent embedded lexical search |
| LanceDB | Apache-2.0 | v0.40.0, 7 Oct 2026 | 18 Oct 2026 | Alternative deployment choice, not an additional default database |
| Graphiti | Apache-2.0 | v0.31.1, 15 Oct 2026 | 17 Oct 2026 | Design reference or isolated research comparator |

Exact timestamps, SHA pins, repository/API links and license links are retained in `COMPONENT_SOURCE_CHECKS.json`. The existing local EdgeQuake workspace identifies version 0.34.0 and Rust 1.95; upstream release activity does not establish compatibility with TACITUS's reviewed branch. [Upstream release](https://github.com/raphaelmansuy/edgequake/releases/tag/v0.35.6), [inspected upstream license](https://github.com/raphaelmansuy/edgequake/blob/17e836a1bd58d556cac684de0341c01d27e08b23/LICENSE), [local workspace manifest](https://github.com/sargonxg/tacitus-political-policy-second-brain/blob/df82de5d33f2debf4ce618b660f76c6d53a39d71/edgequake/Cargo.toml).

### Docling: use only for a measured document-conversion gap

The primary [Docling paper](https://arxiv.org/abs/2501.17887) describes conversion into a structured document representation. Current code exposes `DocumentConverter.convert` and iterable `convert_all`, with page and file-size bounds. `DoclingDocument` represents text, tables, hierarchy, reading order, layout and provenance; it is richer than flattening a PDF to one text string. Sources: [converter](https://github.com/docling-project/docling/blob/0e54294231b69bc2cf7a39873235495788f76b57/docling/document_converter.py), [document model](https://github.com/docling-project/docling/blob/0e54294231b69bc2cf7a39873235495788f76b57/docs/concepts/docling_document.md).

**Fit:** run an isolated, version-pinned conversion adapter only for formats or layouts where the current pipeline loses necessary evidence. Emit structured JSON, a canonical UTF-8 extraction artifact, original-file digest, page/region locators and conversion configuration. TACITUS exact spans should refer to the named extraction artifact. A PDF layout coordinate does not prove those words existed as a contiguous raw byte sequence in the PDF. OCR text needs a visible derived-source status and human comparison for consequential claims.

**Tradeoff:** Python/model packaging adds operational weight to a Rust system. Existing EdgeQuake PDF handling must be evaluated before adding a parallel parser. Compare tables with exceptions, footnotes, two-column order and scanned multilingual pages, rather than only easy prose. The current README requires Python 3.10+ and explicitly separates code licensing from individual model licensing. [README](https://github.com/docling-project/docling/blob/0e54294231b69bc2cf7a39873235495788f76b57/README.md), [MIT license](https://github.com/docling-project/docling/blob/0e54294231b69bc2cf7a39873235495788f76b57/LICENSE), [release](https://github.com/docling-project/docling/releases/tag/v2.140.0).

**Adoption gate:** demonstrably better recovery of the user's actual hard documents, traceable locators, explicit failures, acceptable measured runtime, approved dependencies and model licenses. The paper does not establish superiority over this EdgeQuake configuration.

### Tantivy: a narrow Rust lexical-index adapter

Tantivy offers an embedded Rust full-text index and incremental updates. Its immutable-document update pattern is delete and reindex. The inspected writer exposes `delete_term`, add operations and `commit`; committed changes may not yet be visible to an existing reader under delayed reload. Use a deterministic reader reload when testing or promising immediate freshness. Sources: [README](https://github.com/quickwit-oss/tantivy/blob/67889269cae67063c372bc236b102d632a021568/README.md), [writer code](https://github.com/quickwit-oss/tantivy/blob/67889269cae67063c372bc236b102d632a021568/src/indexer/index_writer.rs).

**Fit:** index a stable `chunk_revision_id`, source revision, language, title, body, identifiers and catalog generation. Keep exact evidence in TACITUS's source store. Use an untokenized ID field for deletion/replacement. Run this channel even when dense search is unavailable. Rebuild from the authoritative ledger; index corruption must not alter accepted evidence.

**Tradeoff:** tokenization and language analysis require explicit tests. BM25 cannot supply cross-language meaning by itself. Do not silently use an English stemmer for all texts. Avoid storing long source copies where a narrow index suffices. The inspected manifest's minimum Rust version is 1.88, below the local workspace's declared 1.95; this is a manifest-level compatibility check, not a successful integration build. [Manifest](https://github.com/quickwit-oss/tantivy/blob/67889269cae67063c372bc236b102d632a021568/Cargo.toml), [MIT license](https://github.com/quickwit-oss/tantivy/blob/67889269cae67063c372bc236b102d632a021568/LICENSE), [release](https://github.com/quickwit-oss/tantivy/releases/tag/0.26.1).

**Adoption gate:** it closes independent lexical recall or embedded latency requirements that the existing PostgreSQL path or simple scan cannot satisfy. At 100–1000 documents, engineering simplicity can matter more than a faster index.

### LanceDB: useful if embedded hybrid storage is an actual requirement

The inspected Rust example combines full-text and vector search; keyed `merge_insert` supports update/insert workflows. Current hybrid-search documentation describes RRF as the default and distinguishes prefiltering from postfiltering. Sources: [Rust example](https://github.com/lancedb/lancedb/blob/ca275b6d0ecc0b72cb13978ff227200727f6334c/rust/lancedb/examples/hybrid_search.rs), [table API](https://github.com/lancedb/lancedb/blob/ca275b6d0ecc0b72cb13978ff227200727f6334c/rust/lancedb/src/table.rs), [hybrid documentation](https://github.com/lancedb/lancedb/blob/ca275b6d0ecc0b72cb13978ff227200727f6334c/docs/web/search/hybrid-search.mdx).

**Fit:** consider it as a replaceable local search adapter if TACITUS needs an embedded deployment without the existing server stack. Use stable business IDs, not physical row IDs, as evidence identity. Store source revision and eligibility metadata; verify both retrieval channels apply required filters. Vector storage versioning is not legal-validity or assertion-correction semantics.

**Tradeoff:** it overlaps existing vector and full-text capabilities. Adding it beside PostgreSQL and Tantivy by default creates extra consistency work. The inspected main-branch manifest uses Rust 1.94 and pinned beta Lance dependencies; assess a stable released dependency graph before adoption rather than copying HEAD. The example's embedding backend is an additional model/runtime decision, not a free capability supplied by the database. [Workspace manifest](https://github.com/lancedb/lancedb/blob/ca275b6d0ecc0b72cb13978ff227200727f6334c/Cargo.toml), [crate features](https://github.com/lancedb/lancedb/blob/ca275b6d0ecc0b72cb13978ff227200727f6334c/rust/lancedb/Cargo.toml), [Apache license](https://github.com/lancedb/lancedb/blob/ca275b6d0ecc0b72cb13978ff227200727f6334c/LICENSE), [release](https://github.com/lancedb/lancedb/releases/tag/v0.40.0).

**Adoption gate:** a deployment need and measured operational simplification, with update/filter parity and retrieval benefit demonstrated. No evidence here establishes that replacing the current storage improves relevance.

### Graphiti: borrow temporal questions, not automatic truth authority

The [Zep paper](https://arxiv.org/abs/2501.13956) motivates temporal graph memory. Current Graphiti code contains candidate retrieval, exact-name duplicate handling, model-mediated entity resolution, edge deduplication and invalidation. Its current contradiction prompt explicitly rejects recency alone as a reason to invalidate a fact. This qualifies the original memo's caution about the older paper: do not attribute a simplistic latest-ingestion-wins rule to the inspected current prompt. Sources: [node operations](https://github.com/getzep/graphiti/blob/417c3a125ae3ac53c19b8e53fffaccf3e665a88e/graphiti_core/utils/maintenance/node_operations.py), [edge operations](https://github.com/getzep/graphiti/blob/417c3a125ae3ac53c19b8e53fffaccf3e665a88e/graphiti_core/utils/maintenance/edge_operations.py), [contradiction prompt](https://github.com/getzep/graphiti/blob/417c3a125ae3ac53c19b8e53fffaccf3e665a88e/graphiti_core/prompts/dedupe_edges.py).

**Fit:** use it to design adversarial cases and, later, an isolated comparator. Its temporal utility treats planned/promised future assertions as valid from the statement reference time, retaining future dates separately; this differs from treating validity as the occurrence date. TACITUS needs an explicit mapping among assertion time, event time and legal effect. The helper uses English-language patterns, which is not evidence of multilingual temporal parity. [Temporal utility](https://github.com/getzep/graphiti/blob/417c3a125ae3ac53c19b8e53fffaccf3e665a88e/graphiti_core/utils/maintenance/temporal_edge_utils.py).

**Tradeoff:** Python, graph drivers and inference clients add another operational system. The current dependency file marks its Kuzu extra deprecated because the upstream project is unmaintained; that is a concrete dependency warning, not proof that Graphiti itself is unmaintained. Name-based collapse and model resolution still require TACITUS-specific homonym, alias and authority tests. [Dependency manifest](https://github.com/getzep/graphiti/blob/417c3a125ae3ac53c19b8e53fffaccf3e665a88e/pyproject.toml), [Apache license](https://github.com/getzep/graphiti/blob/417c3a125ae3ac53c19b8e53fffaccf3e665a88e/LICENSE), [release](https://github.com/getzep/graphiti/releases/tag/v0.31.1).

**Adoption gate:** an experimentally justified need beyond the existing ledger/graph, without surrendering exact-source review, correction or export semantics. Present evidence supports studying the approach, not importing its graph as trusted institutional memory.

## 3. An incremental design that fits the existing system

This is a proposed adapter-level design, not a claim that these records or endpoints already exist.

1. **Capture a source occurrence.** Assign a catalog ID; record user-supplied file or fetched URL, capture time, author/source label, rights/access policy, media type and exact original hash. A web URL is a location, not a revision. Keep a fetched snapshot and its capture status. A user's idea is attributed to the user; it is not evidence that the outside world agrees.
2. **Create a source revision.** A changed file or page creates another immutable revision linked to its predecessor. Identical bytes can reuse storage while preserving separate source occurrences and attribution. Deletions and access withdrawal create explicit status changes rather than silently rewriting history.
3. **Normalize as a named derivation.** Record parser/configuration/model versions, derived-text hash and original locators. Failed OCR, truncated pages and unmappable spans remain visible. Keep a map from each chunk to this artifact; ambiguity does not justify choosing the first matching quotation.
4. **Generate proposals.** Reuse EdgeQuake extraction results where safe. Store mention-level evidence before entity canonicalization. Proposal types should preserve speaker, modality, negation, exceptions, jurisdiction, date precision and inference status. Model confidence or graph importance never grants approval.
5. **Resolve identity conservatively.** Distinguish person, post, organization, committee, legal instrument and source document. Prefer sourced stable identifiers; retain aliases by language. A normalized name is a search key, not identity proof. Accept unresolved/NIL outcomes, version merge decisions and support splitting an erroneous merge. A minister's office and its successive occupants are different entities.
6. **Accept a payload, not just an ID.** Review acceptance binds a digest of the complete proposal, exact evidence revisions, schema and relevant configuration. Mutating any accepted content produces a new revision that needs review. Imported review labels cannot impersonate local authority.
7. **Publish a catalog generation.** Build/update derived indexes, then expose a manifest identifying the ledger and index generations. Queries use a consistent generation or explicitly refuse mixed state. Recovery may replay an idempotent journal. Do not promise a cross-database transaction unless one is implemented and tested.
8. **Retrieve, qualify and compile.** Apply access/eligibility filters before retrieval where possible and revalidate at compilation. Retrieve lexical, optional dense and explicit entity channels independently. Rank candidates, add required qualifications/corrections, compile within limits and retain an omission ledger. A stale index must not resurrect withdrawn evidence.
9. **Track dependent outputs.** A correction or withdrawal marks dependent drafts/capsules stale and records the reason. An offline capsule can only communicate its last-known status; it cannot learn a later revocation without an update channel.

An extraction cache key should include the source/derived-content identity and the extraction configuration. When sharing computation across identical content, rebind only after checking each destination's exact span mapping and policy. Corpus generation and source revision must travel through the Foundry adapter as well; transport should not create another truth store. Existing local real-binary adapter checks remain useful, while authenticated live platform behavior remains a separate, unproven gate.

### Relevant evidence and important evidence are different

Query relevance asks whether a passage helps answer this request. Saliency asks which supported points matter under this user's declared purpose. Treat a user pin, dossier priority or explicit briefing objective as editable metadata. An extractor's importance float, source repetition, graph degree or document recency is not a substitute for that judgment.

Proposed ranking order: first enforce eligibility; then retrieve task-relevant evidence; then apply transparent purpose-specific prioritization and diversity; finally retain all mandatory qualifications. Show why a passage was selected: literal match, reviewed entity match, semantic candidate, user priority, or required exception. Do not imply that ingesting a paper means the user endorses it. Do not let a large repeated press release outweigh one authoritative correction.

## 4. A fair test of relevance, saliency and freshness

All numbers in this section are a **proposed test design**, not results. The accompanying `CORPUS_EVALUATION_PROTOCOL.json` makes the plan machine-readable.

### Corpus and judgments

- Use nested 100-, 300- and 1000-document snapshots, with a frozen core and additional realistic distractors. Report bytes, pages, chunks and language proportions. Record public/source rights. Do not count duplicated synthetic notes as meaningful corpus diversity.
- Start with 120 tasks, 20 in each of six cohorts: exact identifiers; same-language paraphrases; cross-language questions; temporal/version questions; multi-document dossiers; and unanswerable/conflicting requests. Reserve 5 per cohort for development and 15 for held-out evaluation, splitting by dossier/source family where possible.
- Annotate supporting evidence spans, alternative sufficient evidence sets, mandatory caveats, temporal scope, acceptable abstention and a 0–3 relevance grade. Separately annotate purpose-specific saliency weights. Have a second reviewer check high-consequence and disputed items; record disagreement rather than manufacture consensus.
- Gold answers must not be generated solely by the same extractor/retriever under evaluation. Pool multiple retrieval systems for judging, then check for missed evidence so one system's retrieval does not define the gold set.

### Ablations that isolate the benefit

Compare, on identical source snapshots and final budgets:

A. Existing capsule literal/entity selection, clearly bounded to eligible prebuilt capsules.

B. Independent corpus-wide lexical retrieval into the same compiler.

C. Dense-only retrieval with a fixed, separately authorized model.

D. Independent lexical+dense retrieval with fixed rank fusion.

E. D plus bounded, typed, evidence-backed graph expansion.

F. E plus explicit user-priority saliency metadata.

The existing bounded-capsule baseline and a corpus retriever have different search scopes; report both the full-library comparison and a matched-scope comparison. Otherwise an apparent semantic gain may merely be access to more documents. Keep extraction fixed for retrieval comparisons. Test parser changes separately. If a generation model is later used, hold its version, prompt, temperature and evidence budget fixed; include an oracle-evidence condition to separate retrieval failure from drafting failure.

For a first offline protocol, use candidate cutoffs 20 and 50 and final serialized-context budgets of 16 KiB and 32 KiB where supported by the compiler. These are experimental settings, not claims about optimal product defaults. Record the actual emitted bytes and tokens. Report model/API use separately; no inference run was performed for this addendum.

### Metrics and release gates

- **Relevance:** Recall@20/50 over gold evidence and nDCG@10 over graded judgments; evidence recall after compilation at each budget. Report by language and task cohort, not just an aggregate.
- **Saliency:** weighted coverage of independently judged important points within the fixed budget, plus mandatory-qualification retention. Report how many optional points were displaced by each required caveat. More graph nodes or longer answers are not wins.
- **Identity:** false merges, false splits and unresolved rate on labeled mentions; prioritize false-merge visibility because a plausible wrong person can contaminate many answers.
- **Temporal/correction behavior:** stale-evidence rate, correct historical-version selection, withdrawal propagation and newly available source recall. Preserve the distinction between world-valid history and what was known at a given system time.
- **Operations:** cold/warm p50 and p95 retrieval latency, incremental-update latency, peak memory, disk/index growth, bytes/chunks reprocessed, and inference calls/tokens if any. Measure on declared hardware; make no current performance claim.
- **Hard contract gates:** zero wrong-byte citations, accepted-payload substitution, unauthorized-source leakage, unresolved required-correction omission and silent budget overflow in the adversarial suite. This is a release criterion, not a statistical guarantee against unseen failures.

Predeclare the smallest useful relevance/saliency gain and acceptable operational regression after measuring the baseline. Use paired per-task comparisons and uncertainty intervals clustered by dossier; the three nested corpus sizes are not independent samples. Publish failures, abstentions and fallback-path counts. BEIR supports heterogeneous retrieval evaluation; its benchmark findings do not establish the best component for this corpus. [BEIR paper](https://arxiv.org/abs/2104.08663).

### The first regression worth implementing

Create one eligible document containing a rare identifier, make the dense candidate list empty, and query that identifier. The independent lexical channel must recover it; the current inspected fusion function cannot. Then withdraw the source and repeat against a deliberately stale index: retrieval may find a stale candidate, but compilation must reject it. This pair tests a concrete recall improvement and the unchanged trust boundary without an embedding model or paid service.

Other required update tests include identical reimport; whitespace-only changes; changed content under the same chunk ID; repeated quotations in different locations; two people sharing a name; a committee renamed without a new organization; a role-holder transition; contradictory sources without an explicit correction; a later-recorded historical document; a changed extractor prompt; partial index publication; a deleted source still present in an old index; and a bilingual paraphrase with no shared lexical tokens.

## 5. Order of work and stopping rules

**First:** finish the review-gated extraction proposal boundary, exact mapping checks and source/catalog revision contract. Add the empty-dense lexical recovery regression and stale-index rejection case. These address observed gaps without a new inference platform.

**Second:** implement incremental corpus selection and replayable index generations. Run the 100/300/1000-document lexical baseline. Retain the existing capsule limits until evidence justifies changing them.

**Third:** choose at most one narrow optional dependency for a measured gap: Docling for hard document conversion, or Tantivy for embedded lexical search. Prefer existing EdgeQuake/PostgreSQL functionality when it passes the same acceptance tests.

**Fourth:** evaluate multilingual dense retrieval and graph expansion as controlled additions. Consider LanceDB only for an explicit embedded-storage requirement; study Graphiti without importing automatic merge/invalidation decisions as accepted facts.

Stop adding infrastructure when the three user workflows meet measured needs. If a component fails the held-out benefit, freshness, licensing or operational gate, keep the simpler path and publish the failure. The near-term claim TACITUS can earn is traceable reuse of a curated corpus with demonstrably better evidence selection, not universal semantic understanding.
