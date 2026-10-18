# TACITUS diplomatic knowledge architecture

Research and implementation decision memo • 18 October 2026

## Recommendation

Build the next TACITUS release around three connected views: an immutable source ledger, reviewable claims and events with explicit time semantics, and small reusable evidence capsules. Keep the existing Rust capsule kernel as the authority for validation and export. Add semantic extraction, search and drafting behind replaceable adapters; their outputs should never bypass source, time, correction, permission or budget checks.

The distinctive product benefit is institutional memory that a minister or diplomat can inspect and reuse: what a source said, who said it, what qualifications applied, when it was known, what later changed, and which passages support a new brief or speech. A graph is useful for navigating these relationships. Its usefulness must be demonstrated against simpler retrieval, rather than assumed from the presence of nodes and edges.

Prioritize three workflows:

1. Prepare an evidence-linked ministerial brief and show changes since a previous brief.
2. Reuse the same evidence in a bilingual speech without confusing official language versions, assistant translations and policy positions.
3. Build a legislative dossier that separates document versions, procedural events, legal effect and institutional responsibility.

This memo distinguishes **implemented baseline**, **standards-backed design**, **research-supported candidate**, and **unvalidated product hypothesis**. The proposed architecture and acceptance thresholds are recommendations, not reported TACITUS benchmark results. Research used public primary papers, official specifications and the public repository. No external inference service, software installation, private corpus or live Foundry execution was used for this memo.

## What exists and what remains

The public draft [PR 15](https://github.com/sargonxg/tacitus-political-policy-second-brain/pull/15), inspected at commit `a07909567c61167b839af14a137bd1e3eea892e1`, provides exact uploaded UTF-8 source bytes and spans, explicit typed actor/institution/event annotations, revision-bound links, temporal eligibility, portable native-v2 capsules and deterministic lexical/entity selection with conservative link closure. Its run guide documents `propose`, `pack`, `inspect` and `reuse`. These are appropriate foundations to preserve.

Important limits from the inspected implementation and run guide:

- Automatic candidates are literal lines. Free-form entity, relation and date interpretation still requires explicit proposals or a new extraction adapter.
- Retrieval is literal lexical/entity matching, not measured semantic search.
- The current graph snapshot cannot reconstruct an earlier graph. An earlier request needs the corresponding historical capsule revision.
- Linked evidence is retained conservatively as an undirected connected component. A large component can exhaust a budget.
- Global unscoped metadata means a revoked, unavailable or expired source refuses the entire output, rather than allowing a partial capsule.
- The current upload path verifies UTF-8 text. PDF/DOCX extraction, OCR, a production graph store and end-to-end generative drafting are separate work.

The local Foundry adapter's real-binary checks are separate implementation evidence. They do not establish authenticated live Foundry execution or deployment. This memo does not repeat the kernel or adapter authoring work.

## The research that changes the design

### Preserve provenance before building semantic richness

[W3C PROV-O](https://www.w3.org/TR/prov-o/) supplies a useful interchange vocabulary for entities, activities, agents and derivation. Use it to describe a source revision, the extraction run that produced a claim, and the human review that accepted or rejected it. Provenance records a derivation; it does not establish that a source is true or that a reviewer has relevant authority.

[W3C Web Annotation](https://www.w3.org/TR/annotation-model/) supports annotations anchored to source segments. Its text positions count normalized characters, whereas TACITUS uses exact UTF-8 byte spans. A standards export must map these explicitly. Never relabel byte offsets as character offsets. Keep exact bytes authoritative, with an additional quote/context selector and human locator for navigation.

**Design consequence:** every derived record points to an immutable source revision and explicit supporting spans. Summaries and graph labels remain secondary. Hashes detect changed bytes, not authenticity, correctness, rights or freshness.

### Separate two clocks and several kinds of dates

[OWL-Time](https://www.w3.org/TR/owl-time/) offers instants, intervals, temporal relations and date representations at different granularities. It is useful for interoperability, but a claim's valid time and a system's knowledge time still require an explicit application contract.

The [Zep temporal graph paper](https://arxiv.org/html/2501.13956v1) demonstrates separate event and ingestion timelines, raw episodes linked to derived facts, and hybrid retrieval. Its reported experiments concern conversational memory, not diplomatic or legislative reliability. Its approach prioritizes new information during edge invalidation; that policy should not be copied blindly into contested public-policy evidence.

**Design consequence:** maintain valid time separately from recorded/known-at time. A newly ingested allegation is not automatically more authoritative than older evidence. Represent disagreement, correction, withdrawal, amendment and supersession as different reviewed events. Preserve date-only values and uncertainty; do not invent midnight UTC for legal dates.

### Use graphs selectively

[GraphRAG](https://arxiv.org/abs/2404.16130) targets corpus-level sensemaking through entity graphs and community summaries. [Its current documentation](https://microsoft.github.io/graphrag/query/overview/) distinguishes local entity-oriented search, resource-intensive global search, DRIFT and a basic vector baseline. [HippoRAG 2](https://arxiv.org/abs/2502.14802) investigates graph-based associative retrieval and passage integration. These support testing graph-assisted search, not claiming it always beats lexical or vector retrieval.

**Design consequence:** use direct evidence retrieval for precise questions and optional graph expansion for relationship or cross-document questions. Introduce global thematic summaries only when users demonstrably need whole-corpus synthesis, and keep their source dependencies available. A fluent community summary must never become the sole evidence for a consequential claim.

### Evaluate citations as support relationships

[ALCE](https://aclanthology.org/2023.emnlp-main.398/) evaluates attributed generation along correctness, fluency and citation quality. [ReClaim](https://aclanthology.org/2025.findings-naacl.55/) explores sentence-level interleaving of references and claims. Neither makes citation presence equivalent to factual correctness.

**Design consequence:** construct a claim-and-evidence plan before drafting. Validate exact quotations mechanically; assess whether the cited passage supports the generated statement separately. Evaluate attribution, modality, exceptions, dates and scope, not just the citation URL. An automatic entailment score can triage review but cannot certify a ministerial statement.

### Institutions and terminology need explicit models

[W3C ORG](https://www.w3.org/TR/vocab-org/) distinguishes organizations, posts, roles and memberships, with duration on memberships. A post can persist when its holder changes. [SKOS](https://www.w3.org/TR/skos-reference/) provides concepts, language-tagged preferred and alternative labels, and mappings between vocabularies.

**Design consequence:** do not make a person's office a timeless property, and do not treat matching names as identity. Store a concept once with separately sourced language labels. Translation equivalence, aliasing, identity and thematic relatedness are different relations.

### Legislation needs both structure and process

[ELI](https://op.europa.eu/en/web/eu-vocabularies/eli) is a model for legislation metadata. [ELI-DL](https://interoperable-europe.ec.europa.eu/collection/eli-european-legislation-identifier/solution/eli-ontology-draft-legislation-eli-dl) describes legislative projects, activities and documents produced during those projects. [Akoma Ntoso](https://www.oasis-open.org/committees/tc_home.php?wg_abbrev=legaldocml) addresses structured legal and parliamentary documents.

**Current standards update:** Akoma Ntoso v2.0 Part 2, defining AKN 3.1, became an OASIS Standard effective 2 October 2026, according to the [official approval announcement](https://groups.oasis-open.org/discussion/akoma-ntoso-v2-0-part-2-akn-3-1-approved-as-an-oasis-standard-effective-2-oct). Pin actual schemas and parser versions when implementing; a generic assertion of support for “Akoma Ntoso” is insufficient. The [namespace document](https://docs.oasis-open.org/legaldocml/ns/akn/3.1/index.html) identifies the versioned namespace and related files.

**Design consequence:** use ELI-style stable identity and version metadata, ELI-DL-style procedural records, and an Akoma Ntoso adapter where the upstream source actually offers it. Do not require every upload to become a fully compliant legal XML document before it is useful.

## Proposed architecture

### 1 Source ledger

Store immutable content objects and separately versioned metadata. At minimum retain:

- Original file digest, extracted-text digest, media type, language and filename
- Canonical source identifier and retrieval URL where available
- Publisher, publication date with its precision, retrieval time and availability time
- Extraction tool/version, OCR status and transformations performed
- Page/section/paragraph locators and the mapping between original presentation and extracted text
- Rights declaration, access classification, permitted purpose, expiry and revocation state

For PDF or DOCX ingestion, preserve the original file and the extraction result as distinct content objects. A page crop or bounding-box locator can help people inspect evidence, but it does not replace the text-to-original mapping. OCR confidence describes recognition uncertainty, not claim truth. Failed or partial extraction should produce a review queue, not silently complete ingestion.

Keep search-normalized text as a disposable projection. Normalization, transliteration and whitespace repair must not alter the source bytes used by existing span checks. Search results resolve through an explicit mapping to source spans.

**Implementation recommendation:** retain files plus a small transactional catalog first. SQLite is a reasonable local candidate if supported in the existing stack; a server database can later serve multi-user requirements. A graph database is not required for the initial portable graph profile. This is an engineering recommendation, not a measured scale claim.

### 2 Extraction proposals and review

An extractor should emit a constrained proposal envelope, not unrestricted graph writes:

- Source revision and supporting spans
- Mention candidates with exact text and location
- Candidate entity IDs and an explicit unresolved/NIL result
- Attributed claim, modality, conditions, exceptions, quantities and jurisdiction
- Candidate temporal expression, original date text and its interpretation
- Relation type from an allowlist, with schema version
- Extractor/model version and run ID
- Review status, reviewer identity where appropriate, and review history

Separate four records that are often conflated: a source says something; a model infers something; a reviewer accepts a reading; an institution endorses a policy. Acceptance of extraction correctness does not grant institutional endorsement.

Start with a human-review workbench showing the source and proposed record side by side. Prioritize review by consequence and ambiguity: commitments, obligations, numbers, named people, legal dates, negation and contradictory evidence. Rejected proposals stay in the audit history but are excluded from authoritative exports.

Before allowing automatic acceptance, measure error rates on a held-out diplomatic corpus. A model's own “confidence 0.93” is not a calibrated probability. Preserve separate flags for extraction ambiguity, entity ambiguity, temporal ambiguity, source disagreement and reviewer status.

### 3 Small domain profile with standards mappings

Keep the first profile deliberately bounded:

| Record | Purpose | Useful mapping |
|---|---|---|
| Source revision | Immutable evidence object | PROV entity; legal document version |
| Evidence span | Ground a quoted or paraphrased claim | Web Annotation adapter |
| Attributed claim | Who asserted what, with qualifications | TACITUS policy assertion plus provenance |
| Person and institution | Stable identity, separate from labels | ORG-compatible organization profile |
| Role occupancy | Person, post, institution and duration | ORG membership/post |
| Event | Meeting, announcement or procedural occurrence | Time interval plus typed event |
| Concept and term | Meaning and sourced language labels | SKOS; optional terminology references |
| Legislative project and instrument | Process versus published legal object | ELI-DL and ELI |
| Review and correction | Explicit decision with provenance | PROV activity plus TACITUS revision links |

Mappings belong in a versioned adapter/profile. Do not inject new role-occupancy or legal-process predicates into the existing native vocabulary and pretend they are already supported. Until a schema extension is reviewed, retain richer proposals in a sidecar that the core does not treat as authoritative.

[SHACL](https://www.w3.org/TR/shacl/) can validate RDF exports against declared shapes. Continue using Rust validation for native contracts. Validation checks declared structure; it does not verify a legal conclusion or political assertion. [RDF 1.2 Concepts](https://www.w3.org/TR/rdf12-concepts/) was a Candidate Recommendation in the inspected specification; there is no need to make it a critical dependency for this first release.

### 4 Temporal and correction semantics

Every temporal claim needs answers to separate questions:

1. When did the source become available to this workspace?
2. When was this assertion or review recorded?
3. Which period does the claim describe as valid in the world?
4. Is that period declared, inferred, approximate or unknown?

Also preserve domain-specific dates: publication, adoption, signature, entry into force, applicability, transposition deadline, amendment and repeal. A publication date is not a validity interval. A procedural vote is not the same thing as legal application.

Use append-only revisions and explicit correction events. A correction references the exact prior assertion revision. A later source that disagrees should create a contestation relationship unless there is evidence of correction authority or a reviewed legal amendment. Do not implement a universal “latest wins” rule. Distinguish an unknown interval boundary from a deliberately unbounded interval; an empty value must not quietly mean both.

For historical answers, select a historical snapshot or replay an auditable event log that actually existed by the requested knowledge cutoff. Do not backdate today's extracted graph and call it a historical view. Distinguish “what was in TACITUS then” from “what publicly available sources said then”; those are different datasets.

For current answers, show a freshness statement and the corpus boundary. A capsule with a 2024 source and a 2026 capture date has recently captured old evidence; that does not make the source a current account.

### 5 Entity resolution and professional networks

Use stable internal IDs and preserve all source mentions. Resolve first on strong identifiers and contextual evidence, then consider names, aliases, language, institution, role and time. Never merge two people solely because their normalized names match. A candidate list and unresolved state are valid results.

[BELA](https://arxiv.org/abs/2306.08896) demonstrates multilingual end-to-end entity linking; [2026 research on historical multilingual entity linking](https://arxiv.org/abs/2601.08500) explicitly includes candidate retrieval and NIL prediction. These motivate an adapter and evaluation set. They do not establish accuracy for contemporary diplomats, transliterated names or the user's private institutional context.

Merge and split decisions must be reversible, sourced and versioned. A correction to identity should invalidate affected graph projections and flag downstream capsules for review. Distinguish same person, former officeholder, successor, institution rename and institution merger.

The network view should answer work questions such as “Which office owns this issue?” or “Who held this publicly documented role on the meeting date?” A line on the graph needs a named relation and source. Co-mention, co-attendance or shared membership cannot establish friendship, access, trust or political alignment. Do not manufacture a personal influence score.

### 6 Retrieval and capsule compilation

Treat retrieval as candidate generation followed by evidence compilation:

1. Interpret the request's topic, entities, date, language, jurisdiction, purpose and audience.
2. Restrict the candidate universe by authorization and available historical snapshot.
3. Retrieve through the baseline lexical/entity index. Later, add a multilingual dense channel and controlled concept expansion.
4. Optionally expand through explicitly typed graph relations with bounded traversal.
5. Recheck access, time, source state and relation validity on every candidate and dependency.
6. Add mandatory corrections, caveats and supporting evidence as atomic closure groups.
7. Rank eligible groups and compile them within the actual serialized byte budget.
8. Emit selected evidence, omissions, refusal reasons and retrieval provenance.

The first semantic experiment should compare lexical-only, dense-only and hybrid retrieval under the same corpus, filters, context budget and generator. Rank fusion is a candidate method, not a guarantee. Graph expansion can improve recall while harming precision; measure both. A highly ranked old claim cannot defeat eligibility rules.

Do not quietly shorten away a qualifier to fit a budget. If a necessary closure group will not fit, omit the group with a visible reason or ask for a larger budget. Retain today's conservative whole-component closure until a directed, typed alternative passes adversarial tests.

The capsule should remain model-independent. Recommended additions, through explicit versioned extensions, are:

- Purpose and scope, intended audience and language
- Snapshot/knowledge cutoff, valid-time request and freshness state
- Source and assertion revision IDs, digests, citation spans and dependency closure
- Ontology/profile version, extraction/review history and transformation history
- Provenance of term selection and translations
- Allowed uses, export restrictions, expiry and a revocation-check policy
- Query/selection receipt, omitted groups and reasons, compiler version and budget

Keep a compact prompt-ready view and a complete auditable artifact. A signature can attest who issued a capsule, not that its facts are true. Offline recipients cannot learn a new revocation without an update channel; disclose the last checked state and enforce bounded freshness where required. Sensitive capsules should not be made broadly portable simply because the format supports export.

### 7 Grounded briefs and speeches

Generate from a structured evidence plan with separate sections for sourced facts, attributed positions, disagreements, analysis, proposed language and unresolved questions. Every factual sentence should carry the claim/span IDs it uses. A rhetorical sentence does not need an artificial citation, but it must not smuggle in a new factual or policy commitment.

Use two output views: a clean reading copy and a review copy with sentence-to-evidence links. Mechanical checks catch missing citations, malformed references, altered quotations, amounts and explicit dates. Semantic checks and a human reviewer assess whether the language overstates the source, drops an exception or attributes a proposal as settled policy.

Changes to a supporting source should flag all dependent drafts and capsules as potentially stale. Rebuilding should produce a reviewable diff rather than silently replacing a speech. Publication, institutional approval and communication remain separate actions.

### 8 Multilingual terminology and translation

Use a language-neutral concept ID with language-tagged labels, definition, domain, institution, source and review status. [UNTERM](https://unterm.un.org/) is an official terminology resource for the UN's six official languages. [The UN translation service](https://www.un.org/dgacm/content/translation) describes expert terminology work and review; it also notes the nuance of diplomatic language. [EU Vocabularies](https://op.europa.eu/en/web/eu-vocabularies) provides multilingual reference data, including EuroVoc. The [2025 EuroVoc paper](https://aclanthology.org/2025.ldk-1.34/) describes its legislative and retrieval applications.

Use those resources as referenced authorities where applicable, subject to their access and reuse conditions. Do not claim a live terminology integration or bulk-redistribution right merely because the resource is publicly searchable.

Store separately: original-language quote; official parallel-language version; assistant translation; reviewed translation; approved organization-specific term. An official English version of legislation is not the assistant's English translation of the French excerpt. Preserve deliberate ambiguity, modality and institutional naming. Translation must not convert “may” into “shall,” a proposal into a commitment, or an approximate date into an exact deadline.

Search-language support and translation support require different evaluations. [MIRACL](https://aclanthology.org/2023.tacl-1.63/) is useful for multilingual retrieval; [XOR QA](https://aclanthology.org/2021.naacl-main.46/) explicitly tests questions and evidence across languages. [MMTEB](https://arxiv.org/abs/2502.13595) can help shortlist embedding candidates. Their aggregate results are not guarantees for the languages, registers and legal terms needed here.

## Three demonstration workflows

These are concrete product acceptance scripts. The current runnable slice is limited to the fixture and core behavior described below; the fuller experience is a proposed extension. It would be misleading to report three completed end-to-end generative workflows before their outputs and tests exist.

### Workflow 1 A ministerial brief that survives a changed position

**User request:** “Brief me for the port-access meeting. What was promised, under which conditions, and what changed since the last brief?”

**Inputs:** the existing synthetic port-access note, its later withdrawal, a distinct heat-response note, and historical capsule revisions where an earlier view is requested.

**Output:** a concise position brief with attribution, condition, exception, correction and source links; a change panel that identifies the prior and later assertion revisions; a clear distinction between known-at and valid-at.

**Runnable baseline:** the PR 15 fixture selects the port evidence and preserves its linked withdrawal, while a heat query selects different evidence. A strict date request with unknown validity refuses. That proves source-bound selection and conservative correction preservation, not semantic understanding or a completed natural-language brief.

**Next extension:** add a deterministic review view over the returned context, then an optional generator bound to a claim plan. Historical comparison requires two actual capsule revisions or a properly implemented event log.

**Pass criteria:** no current brief presents the original commitment without the linked withdrawal; every quote resolves to exact bytes; conditions and exceptions are retained; unrelated heat evidence does not become a port conclusion; unsupported earlier snapshots refuse; a smaller budget never separates a commitment from its required correction.

### Workflow 2 A bilingual speech from a reusable legislative capsule

**User request:** “Prepare French speaking notes on the AI Act using our evidence capsule. Keep the formal terminology consistent and let me inspect the original support.”

**Inputs:** short English and French excerpts from original Article 113, plus a separately authored contextual note on publication, phased exceptions and the original page's current-version warning. Verified original sources: [English](https://eur-lex.europa.eu/eli/reg/2024/1689/oj/eng) and [French](https://eur-lex.europa.eu/eli/reg/2024/1689/oj/fra).

**Output:** speaking notes with each legal assertion linked to the relevant original-language evidence; an evidence pane that distinguishes the official English version from an explicitly unreviewed assistant translation of the French quote; visible warnings about exceptions and document version.

**Runnable slice:** the French literal query `discours` selects an explicit drafting recommendation with dependency closure to language originals, exceptions and a version warning. The saved output and run receipt were inspected: a 15,877-byte context contains five evidence-linked claims. A French query with no lexical overlap produces a 3,162-byte omission-only receipt. The acceptance guide was also inspected. Independent final review cleared fixture revision `0a52df09`, reporting 122 tests, Clippy, formatting and all three CLI cases passed, with the EUR-Lex primary sources verified. This demonstrates multilingual evidence preservation and prompt-specific lexical reuse, not cross-language semantic search or an automatically generated speech. The 35 cases in the separate evaluation blueprint remain proposed tests, not executed results.

**Pass criteria:** originals and translations never swap provenance; accented text survives byte-exact roundtrip; publication and legal dates retain their original precision; exceptions and freshness warnings survive compilation; no unsupported claim that the original Article 113 dates are the current legal timetable; the no-overlap control remains visibly empty.

### Workflow 3 A legislative dossier and institutional responsibility map

**User request:** “Explain this instrument's legislative path, which institutions are involved, and which dates are publication, entry into force and application.”

**Inputs:** a pinned legal instrument and its language versions; official procedure records; explicit stage events; dated institutional role evidence where people are involved.

**Output:** a timeline with named stage types and source links, a version tree, and an institutional map whose edges have named roles and supporting evidence. An unresolved stage is visibly unknown.

**Runnable slice:** the query `publication institutions` selects the authored publication record and existing institution nodes for Parliament and Council. Its inspected saved output is 6,436 bytes, with one selected claim confirmed in the run receipt. This is a bounded subset of the workflow. The present core does not demonstrate automatic extraction of a legislative process or an officeholder network.

**Next extension:** add an ELI-DL-shaped proposal adapter and approved native extensions for procedural records. Keep the instrument, its language expression and its downloaded file distinct. Tie a person's occupancy to an institution, post and interval rather than copying a current title into every historical record.

**Pass criteria:** proposal, adoption, publication, entry into force and application cannot be silently equated; a press release cannot override the pinned legal text without an explicit review; document consolidation is distinct from a new lawmaking event; unknown or conflicting dates remain visible; no institution edge is invented from co-mention; every displayed current officeholder requires current supporting evidence.

## Evaluation plan

### Separate structural guarantees from empirical quality

The deterministic kernel can guarantee properties within its declared input contract. It cannot guarantee that a human or model annotated the world correctly. Maintain two scorecards.

**Contract scorecard:** byte integrity, exact span resolution, schema validation, permissions, time eligibility, correction closure, deterministic serialization, budget, refusal and omission behavior. These should pass every fixture; any violation blocks release.

**Quality scorecard:** extraction accuracy, entity resolution, retrieval recall/precision, answer support, multilingual fidelity, usefulness and review effort. These require representative labeled evidence and human assessment. They should never be collapsed into a single “trust score.”

### Build the smallest useful domain benchmark first

Begin with 100 independently reviewed task cases across the three workflows and expand after error analysis. This size is a development starting point, not statistical proof of production reliability. Reserve entire dossiers and document families for held-out testing to reduce near-duplicate leakage. Maintain distinct synthetic adversarial tests and real public-source cases.

Stratify by:

- Exact lexical questions, paraphrases and multi-document questions
- Current view versus historical known-at view
- Explicit dates, partial dates, relative dates and unknown intervals
- Corrections, conflicting sources and legally distinct stages
- Same-name people, renamed institutions and former/current roleholders
- In-language and cross-language evidence retrieval
- Numbers, negation, conditions, exceptions and quoted instructions
- Unanswerable questions, revoked sources, missing dependencies and budget pressure

Use [LongMemEval](https://arxiv.org/abs/2410.10813) for inspiration on updates, temporal reasoning and abstention. The [2026 LongMemEval-V2 project](https://xiaowu0162.github.io/longmemeval-v2/) extends evaluation to long web-agent trajectories; it is relevant if TACITUS later stores procedural work experience. Neither is a substitute for a diplomatic source-grounding benchmark. [BEIR](https://arxiv.org/abs/2104.08663) supports retaining a lexical baseline and testing across domains rather than assuming dense retrieval dominates. [RAGEval](https://aclanthology.org/2025.acl-long.418/) is useful for scenario-specific test design, but generated cases should be independently checked.

### Metrics and release gates

| Area | Measure | Proposed gate |
|---|---|---|
| Evidence integrity | Quoted span resolves to declared immutable bytes | 100% of deterministic cases |
| Correction integrity | Required linked corrections/qualifiers preserved or atomically omitted | 100% of deterministic cases |
| Historical leakage | Evidence unavailable at cutoff appears in answer context | Zero in adversarial tests |
| Permissions | Unauthorized or revoked evidence/metadata exported | Zero in adversarial tests |
| Legal dates | Stage and precision preserved | 100% of fixed regression cases |
| Retrieval | Recall@k, nDCG@k and full supporting-set recall within byte budget | Report by language and task; improve on baseline before default promotion |
| Entity resolution | Precision, recall, false merges and correct NIL decisions | No false merges in release fixture; calibrated production threshold decided from held-out data |
| Citation support | Supported factual claims / all factual claims; citations that actually support their claims | Expert-reviewed; consequential unsupported claim blocks that deliverable |
| Qualifier fidelity | Actor, modality, negation, quantity, condition, exception and scope preserved | No material omission in release workflow cases |
| Abstention | Correct refusal and answerable-question over-refusal | Report both; do not reward refusal alone |
| Translation | Term consistency and expert-rated semantic/qualifier fidelity | Reviewed critical terms; no material drift in release cases |
| Efficiency | p50/p95 latency, actual bytes/tokens, index/build cost and reviewer minutes | Measure on a named machine/corpus; no invented performance target |

Treat a zero-failure test result as evidence about the tested set, not proof of zero production risk. Pre-register task splits, scoring definitions and promotion criteria before comparing retrievers. Report uncertainty and individual failure cases. If model judges assist, disclose their model/version, validate a sample against human ratings and never allow them to be the sole release authority for material errors.

### Fair ablations

Compare: current literal baseline; stronger lexical retrieval; dense-only; hybrid; hybrid plus bounded graph expansion. Keep data versions, access filters, time filters, generator, prompt, evidence budget and review policy constant. Evaluate retrieval separately from generation, then end to end. Include an oracle-evidence condition to distinguish a retrieval failure from a reader failure.

Measure preservation as well as recall: can a shorter capsule retain the same facts, qualifiers, contradictions and citations as the complete dossier? Include repeated export/import, changed prompts, changed language, source revocation and source correction. Do not call a hand-annotated toy fixture a semantic benchmark.

## Prioritized delivery sequence

### P0 Make the existing guarantees visible

Deliver the three fixture-backed evidence views, clear boundary labels, a reproducible offline run, and the contract test scorecard. Present exact source, attribution, original language, known-at/valid-at status, dependency closure and omissions in a legible review surface. Keep the current stable kernel unchanged unless an actual defect is found.

**Exit:** a new reviewer can reproduce the fixture run, understand what was and was not inferred, and inspect every displayed citation. The absence of semantic matching or a live platform connection is obvious.

### P1 Make uploaded documents reliably reviewable

Add one ingestion adapter at a time, beginning with the document formats most useful to the user. Implement original-to-text mapping, extraction proposals, reversible entity resolution and a human review queue. Create the domain benchmark before enabling automated acceptance.

**Exit:** held-out extraction and entity errors are measured; unsupported dates and ambiguous identities remain unresolved; all accepted claims retain verified source mappings.

### P2 Improve semantic reuse and drafting

Add a multilingual retriever behind the same compiler boundary. Evaluate hybrid and graph expansion through controlled ablations. Add claim-planned brief/speech drafting and bilingual terminology review. Store generated text as a derived artifact whose supporting evidence can be traced and invalidated.

**Exit:** measurable workflow gain at an explicit cost and no regression in contract gates. A held-out paraphrase or cross-language query must succeed before the product advertises semantic or cross-language retrieval.

### P3 Add institutional scale only when needed

Add multi-user authorization, tenant isolation, signed export policies, operational observability, versioned standards adapters and a live Foundry integration after credentials, project access and resource approval exist. Introduce a server graph database only if measured traversal, sharing or scale requirements justify it.

**Exit:** end-to-end authorized platform run, reproducible deployment manifest, failure/rollback tests, verified access controls, and independent review. A local adapter smoke test is not this gate.

## Decisions to defer

Do not yet commit to a universal political ontology, automatic adjudication of conflicting truth claims, graph database migration, fine-tuning, unrestricted external-agent memory sharing, global thematic precomputation, or a capsule marketplace. Each adds cost and trust assumptions before the three core workflows are proven.

The decisive near-term question is: can a user safely take a small capsule from one task into another, preserving the original evidence and the qualifications that change its meaning? TACITUS already has a credible foundation for testing that question. The next work should make it usable, evaluate the missing semantic layer, and keep claims about implementation narrower than the evidence.
