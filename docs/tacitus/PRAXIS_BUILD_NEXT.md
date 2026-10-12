# PRAXIS execution prompt — finish the Tacitus political/policy workbench

Paste this into the local coding agent that has access to the repository and machine.
Execute implementation, not another plan-only response. Work in tested increments;
never claim a capability that did not run. Preserve working code and explicit limits.

## 0. Destination and observed starting point

Repository: https://github.com/sargonxg/tacitus-political-policy-second-brain
Upstream: https://github.com/raphaelmansuy/edgequake

The repository already exists. Do not create another repository or re-clone EdgeQuake
as a replacement. Inspect remotes, branches, pending changes, instructions and current
status before modifying anything. The inspected base was
`edd78305254a367686bd9e3fa4bd118dc207e05d`; treat this as provenance, not permission
to reset newer work. Review the policy-lab pull request/branch and continue on a
non-destructive working branch. Do not force-push or merge to main without approval.

Existing Rust crates under `edgequake/crates/`:

- tacitus-core: SourceVersion, Span, Assertion, DecisionContract, ReviewReceipt,
  local-model policy and storage/Praxis interfaces.
- tacitus-capsules: bounded native v1 ZIP format, selected-source evidence compiler,
  and preview/pack/import/compile developer CLI.
- tacitus-analysis: model-trait decision brief, hypotheses, commitments, dependency closure.
- tacitus-evals: comparable evaluation records, not demonstrated model gains.

Added `tools/tacitus-policy-lab/` is a dependency-free Node/browser development lab.
It supplies an application ontology, typed relation proposals, semantic review cues,
method proposal/guard logic, explicit-evidence context requests, hypotheses, change
impact, and Rust pack inputs. It does not replace the production Rust kernel, native
capsule format, authenticated API, database, or Next.js application.

Reproduce the lab tests first. Its 81 Node tests previously passed on Linux/Node
22.16.0. The native v1 schema matched blob
`86ee68b3385524a839037eff810d526585550d53`; manifest/data passed structural validation.
Nine component checks used embedded assets with test cryptography because real browser
HTTP navigation was administrator-blocked. That is NOT full browser acceptance.
Cargo/Docker/Ollama and native ZIP round-trip did not run in that environment.
Never carry these limitations forward as completed milestones.

Read IMPLEMENTATION_STATUS.md, UPSTREAM.md, docs/tacitus/*, the crate implementations,
current frontend package/lockfile and schema generator. Preserve unresolved inherited
history/dependency audit findings and the existing Actions state. Do not print possible
secrets, disable security tools, waive findings, or enable releases/CI/cloud publishing.
Prepare remediation by fingerprint/category; request a decision when a genuine approval
is needed. Continue independent local implementation while a publication gate is blocked.

## 1. Product contract and scope

Build a local-first research and judgment workspace for policy analysts, diplomatic
teams and political researchers. The golden path is:

Question -> decision contract -> selected evidence -> proposed atomic claims -> review
-> context compilation -> comparative analysis -> cited brief -> correction -> reusable
cognitive capsule -> changed evidence -> impact review.

Political usefulness means preserving institutional distinctions, alternative explanations,
implementation constraints, affected people's interests, and decisive qualifications.
It does not mean producing agreeable partisan conclusions, profiling individuals for
political targeting, fabricating certainty, or treating a fluent memo as verified analysis.

Use the existing EdgeQuake backend/frontend. Consolidate useful lab interactions into
that frontend; do not ship the laboratory static server as the production multi-user app.
The Rust crates remain authoritative. Retain the lab for fast regression/examples as
appropriate; remove or generate duplicated validation logic once native integration is
proven. No extra Neo4j, Qdrant, Redis, hosted vector database or separate auth system
is needed for the first release.

## 2. Get a genuinely local vertical slice working first

Inspect the actual machine: OS, Docker/Compose, CPU/GPU, free RAM/disk, port conflicts,
Rust/frontend toolchains, and installed local models. Report exact missing prerequisites.
Do not purchase services, install heavyweight models indiscriminately, expose LAN ports,
open tunnels, or alter system security policy. Obtain any required installation approval.

Use source-built modified API/frontend, persistent PostgreSQL with compatible pgvector
and Apache AGE, migrations, and local Ollama. Reuse the retained upstream toolchains and
lockfile until a tested requirement demands change. Never claim the modified UI runs
when actually serving an unchanged upstream prebuilt image.

Configure extraction, query, keyword, summary and embedding roles locally. Disable
optional vision/reranker/decision roles until verified. Check saved connections and request
overrides as well as environment variables: a single cloud fallback invalidates local-only
mode. Set OLLAMA_NO_CLOUD in the actual server environment, restart/verify it, and reject
nonapproved model identities/endpoints. Do not inherit developer cloud API keys.

Start with one small supported local chat model and one local embedding model; inspect
availability, license, structured output, RAM and context behavior rather than assuming
a past model recommendation remains suitable. Pin downloaded identities/digests, vector
dimension, model options and effective context window. Keep concurrency small. Changing
embedding identity requires a new index/re-embedding, never mixing vector spaces.

Build three measurable checks: import a newly created text with a random marker; verify
real local embeddings/graph extraction; ask a question answered from that marker with an
exact citation. Restart and repeat. Mocks must remain restricted to unit tests. Missing
models or services must fail explicitly, not fall back to fake successful output.

Run the app with no required paid API or hosted account. Initial software/model downloads
and explicitly requested source imports may use network; cached evidence and analysis
must work without internet. Verify an isolated offline mode after downloads. Do not equate
"local endpoint" with proven absence of server-side cloud routing or data leakage.

Deliver shell/PowerShell launchers for doctor, setup, up, demo, smoke --live, test, local
benchmark, backup and down. Preserve data on ordinary stop/restart. No destructive reset
without explicit authorization. Record which OS/profile actually ran.

## 3. Canonical persistence and API boundaries

Add versioned namespaced Tacitus tables/migrations in the existing database. Canonical
records: immutable source versions/extraction artifacts, source spans, assertion revisions,
review receipts, ontology/module versions, capsule manifests, memory proposals, compiled
context receipts, analysis artifacts, decisions, revision events and dependencies.

Graph/vector materializations are derived projections. Implement idempotent indexing,
source-revision events, dependent-artifact staleness, and deletion/access-revocation
invalidation. Original source bytes/text hashes and authored assertions cannot be silently
rewritten. Use transactions and optimistic concurrency on review/version updates.

Expose a namespaced `/api/v1/tacitus` route group under the EXISTING authentication and
workspace authorization. Server code sets reviewer identity/origin. Never trust a client
field such as reviewed=true or origin=local_human. Check permissions at initial retrieval,
graph expansion, source preview, cache hit, derived summary, export and direct upstream
endpoints. A front-end filter is not authorization.

Separate "review happened" from "claim accepted": a rejected/caveated receipt cannot
be treated as acceptance. Bind reviews to assertion fingerprints and evidence versions;
check receipt time relative to the requested known-as-of cutoff. Source edits, review
revocation or changed model proposals cannot inherit a stale accepted badge.

## 4. Ontology and semantic layer

Use `ontology.mjs` as an initial APPLICATION vocabulary. Move its machine-readable
registry to the native layer with a version, stable identifiers, definitions, domain/range
constraints, provenance and explicit migrations. Preserve, at minimum:

Actor, Institution, Claim, Position, Interest, Constraint, Leverage, Commitment, Event,
Narrative, Mandate, Authority, FundingStage, Instrument, Option, Hypothesis, Assumption,
Indicator, Forecast, Decision, Judgment, SourceVersion, EvidenceGap and AffectedGroup.

An edge is a projection of an attributed assertion, not an independent fact. Relations
need source assertion IDs, epistemic type, valid/knowledge time, conditions, exceptions
and review state. A relation label alone never establishes control or causation.

Expand versioned domain packs for diplomacy/conflict, multilateral institutions,
public-policy implementation, and political economy. Include explicit membership versus
alignment, authority versus capacity, declared versus inferred interests, and stages of
budget/implementation. Record jurisdiction, authority chain, instrument, expiry, renewal,
and who can make or execute a decision. Do not embed unverified current law as ontology.

The current Rust Assertion wire shape has no unrestricted predicate extension slot and
uses deny_unknown_fields. Introduce any new fields through a real schema version,
source-preserving migration and compatibility tests. Do not append arbitrary keys and
claim old readers remain compatible. Apply the same discipline to the limited v1 Method
fields; prerequisites/exceptions currently carried in lab artifacts need a native model.

Support aliases, multilingual source wording, ambiguous identities, conflicting mappings,
source families, translation provenance, supersession and nonexclusive interpretations.
Entity merges require evidence and review, with an undo/history path. Neither matching
names nor graph proximity demonstrates identity, influence or ideological similarity.

A JSON-LD/RDF export or constrained shape validation may be useful, but is a derived
representation. Do not add an OWL reasoner simply for branding. Document exactly which
constraints and inferences are implemented, and never claim broad semantic reasoning
from a glossary or regex engine.

## 5. Evidence, provenance and temporal correctness

Start with selected TXT/Markdown/paste and text-bearing PDFs through local extraction.
Preserve original bytes, extracted text, both hashes, extractor version, language,
publication/retrieval dates, rights and source family. Citation offsets remain half-open
UTF-8 byte offsets in immutable extracted text. Keep page/paragraph locators when known.

A source establishes what its author said, not automatically that its contents are true.
Preserve reported_statement, observation, allegation, inference, hypothesis, forecast,
value_judgment and recommendation. Keep the original quote distinct from interpretation.
"May consider" is not "has committed"; authorization is not expenditure; association
is not command. Conditions and exceptions must survive extraction, retrieval and drafting.

Distinguish event/world-valid time, publication, retrieval, assertion creation, availability,
review and supersession. A later correction must not improve an answer presented as what
was known earlier. New memory and reviewer corrections must also respect that cutoff.
Unknown dates must remain explicitly unknown rather than fabricated.

Implement source independence as provenance, not repetition counts. Syndicated copies
can improve accessibility but do not automatically multiply independent confirmations.
Assess directness, competence, completeness and incentives without one universal truth score.

Public URL/RSS imports come only after the offline workflow works. Require explicit
source selection, resource limits, redirect validation, SSRF protections and supported
rights. Do not crawl private files/accounts or route confidential dossier text into public
search. Scanned PDFs/vision/OCR are optional local extensions with explicit failure states.

## 6. The policy harness and context compiler

Implement a typed ToolSpec/ModuleSpec registry in the existing analysis layer. Each tool
has id/version, input/output schema, capability requirements, evidence contract, explicit
network/filesystem/model permissions, timeout/budget, reviewer gates, deterministic tests,
and error semantics. Only trusted installed handlers execute. Imported capsules never
install executable plugins or grant themselves tool authority.

Priority tools: evidence.search, evidence.inspect_span, claims.compare, positions.diff,
commitments.inspect, authority.trace, hypotheses.matrix, assumptions.audit,
options.compare, indicators.define, context.compile, memory.propose,
memory.applicability, capsule.validate/preview/export/import, change.impact and brief.draft.
Implement only tools with working handlers and tests; others stay clearly planned.

Compile context from a full decision contract: decision-maker, audience, mandate/scope,
question, objectives, constraints, horizon, known-as-of/valid-at dates, selected sources
and capsules, permitted tools, format, and input/output/overhead budget. Use actual model
token accounting or label estimates honestly. The lab's UTF-8 byte budget is not a tokenizer.

Filter authorization and time BEFORE retrieval/graph expansion, then recheck outputs.
Use tuned lexical/vector retrieval and graph neighborhoods where they help. Verify the
pinned EdgeQuake HTTP/context-only contract with real tests; do not invent endpoints.
Preserve decisive counterevidence and qualifications when truncating. Report selected
records, source versions, applied/guarded methods, unresolved conflicts and omissions.

Salience means potential effect on judgment, action, timing, safeguards or confidence.
Do not substitute token overlap, source frequency or ideological agreement. Use task-type
priorities that can be evaluated, with a direct full-document path when that is better.

One bounded analyst/execution flow is the default. Add a separate adversarial check where
its measured value justifies the cost; not an unconditional swarm. Persist artifacts and
recovery checkpoints, cancellation, retries, tool-call receipts and failure reasons.

## 7. Cognitive memory and capsules

Keep evidence memory, situation memory, analytical memory, decision memory and method
memory distinguishable. Methods include distinctions, cues, heuristics, patterns,
exceptions, red flags, diagnostic questions, language rules, procedures, cases, unknowns
and revision triggers. Store scope, evidence, author/reviewer, creation/availability,
prerequisites, false-positive/failure conditions, dependencies and supersession.

AI output proposes memory; an authenticated human may promote it after review. Feedback
must not train agreement with the user. Preserve rejected proposals, dissent and the
reason a prior assessment changed. Never treat a prior fluent answer as fresh evidence.

Applicability is at least active-candidate / guarded / blocked / stale. Unknown
prerequisites or unknown exceptions must not silently pass. A source's changed version
or digest, ontology change, expired mandate, revised procedure or withdrawn review should
flag dependent memory and compiled artifacts. Reconsideration is not automatic deletion.

Keep native `.capsule` ZIP v1 canonical until a tested migration exists. Support context,
cognitive-method and output-artifact profiles with shared provenance/governance. Retain
manifest identity/version, pinned dependencies, rights, confidentiality, lineage, dates,
source hashes, original language and explicit missing-evidence limitations.

Demonstrate: create -> inspect exact exported fields -> native validate/pack -> fresh
workspace import -> proposal review -> compile -> reuse. References-only and evidence-
included are distinct. Removing source bodies does not automatically declassify quoted
assertions, notes, graph labels or reviewer metadata. Sharing requires preview and rights
checks on the entire derived package, not only the original PDF.

Retain ZIP resource/path/duplicate-name/hash protections. Reject executable payloads and
arbitrary dependency URLs. Imported review attestations remain untrusted. Hashes show
integrity, not truth or signer identity. Do not claim signing without actual key/trust
verification. Offline exports cannot be remotely recalled; communicate this limitation.

Praxis compatibility stays provisional until an authorized canonical schema is supplied
and round-trip tests pass. Do not copy private Praxis source into this public repository.
Build a replaceable adapter with version negotiation, fidelity/loss reports and migration
examples instead of claiming compatibility on the strength of similar field names.

## 8. Four real policy workflows, then extensions

1. Decision brief: supported key judgments; strongest alternatives; options including
status quo; authority and feasibility; constraints/resources; implementation sequence;
affected groups and distributional/rights effects; uncertainty; evidence gaps; signposts.
Keep quotations, inferences, value choices and recommendations visibly distinct.

2. Competing hypotheses: editable hypotheses and evidence rows; diagnosticity and
consistency rationales; absent evidence; source-family dependence; what could discriminate
between hypotheses. Do not turn consistency counts into Bayesian probabilities.

3. Position/commitment tracker: compare original language across dates; speaker authority,
conditions, deadlines, action, acceptance, implementation and withdrawal. Identify the
missing evidence needed to call a change meaningful. Conditional wording must survive.

4. Change-impact review: trace a source correction through assertions, hypotheses,
assumptions, options, memory and briefs; show before/after context and reasons; require
review before changing accepted artifacts. Preserve assessments not affected by the change.

Then expand stakeholder/institution mapping, mandate chains, assumptions checks,
red-teaming, policy-option tradeoff matrices, scenarios/signposts, forecast resolution,
negotiation preparation, source-landscape/narrative analysis and institutional handovers.
Each extension needs a schema, tool handler, evidence contract, real UI and acceptance
fixture. Value weights in option scoring are explicit analyst choices, not facts; provide
sensitivity analysis rather than a decorative single "best option" score.

## 9. Interface and graphics

Use the existing Next.js app and source/graph components. Port successful lab interactions;
do not iframe the lab as a substitute for authenticated integration. Three-zone workbench:
dossier/navigation left, analysis/reading center, source/context inspector right.

Use restrained ivory/slate/teal styling, readable typography, accessible contrast,
keyboard navigation and narrow-screen panel collapse. Separate status from decoration:
local-only, human-reviewed, fresh and verified labels must reflect backend state.

Every citation opens the exact passage/version. Every graph edge opens the attributed
claim, its support, time and review state. Selecting an as-of date changes actual data
selection, not just a label. Show contradictions, unavailable evidence and context-budget
omissions. Provide an accessible table alternative to graph views.

Prioritize useful 2D actor/claim/provenance graphs, timeline lanes, hypothesis matrices,
option tradeoffs and change-impact views. Stable layout and filtering precede optional
3D. Graph distance is not political distance. Avoid fake live feeds, animated globes,
unused controls or unverifiable numerical confidence gauges. No external assets required.

Show meaningful loading/progress/cancel, partial extraction, missing-model and empty states.
Save work durably and explain what persists. Test actual browser navigation, imports,
citations, review, context refresh, capsule round-trip, restart and responsive behavior.
Component screenshots and HTTP unit tests do not replace this end-to-end acceptance.

## 10. Evaluation and acceptance gates

Reproduce unit/schema/contract tests; add the real Rust capsule round-trip and cross-language
UTF-8 cases. Maintain fixtures for conditional language, institution authority, funds,
common-origin reporting, ambiguous entities, conflicting translations, future knowledge,
source withdrawal, stale review, memory exceptions, prompt injection and malformed bundles.

Compare the same local model with: full documents where feasible; unmodified EdgeQuake;
tuned retrieval; Tacitus context; and Tacitus context plus reviewed memory on a later task.
Give baselines equivalent legitimate source access and earlier corrections. Separate cold
and warm costs, including ingestion, indexing, retries, analyst correction and maintenance.

Measure decision-critical coverage, material omissions, supported claims, preserved
qualifications, citation validity, temporal mistakes, uncertainty handling, correction
time and total time to an acceptable brief. Keep automatic checks distinct from human
judgments. Missing human scores are pending, never silently perfect or zero.

Split by dossier and chronology. Keep expected answers out of ingestion/runtime context.
Public synthetic regressions are developer tests, not held-out validation. Claim only
measured improvements for the named task set/model/version/date. Frontier/API arms remain
NOT RUN unless explicitly authorized; do not spend money or advertise superiority.

Release gates: source-built startup; local inference/embeddings; durable ingestion;
authenticated source isolation; cited brief; reviewed memory reuse; native capsule
round-trip; changed-evidence review; restart/offline run; real browser test; clean-clone
reproduction; documented audit disposition. Neither library tests nor a renamed UI is
sufficient. Never weaken tests merely to pass.

## 11. Execution and final report

Work by small pull-request-sized milestones. At each stage: inspect current code ->
write failing acceptance cases -> implement -> test -> record exact commands and results
-> update implementation status. Keep this public repository free of private data,
credentials, runtime exports, model weights and unreviewed raw logs. Review additions
before publication; preserve existing main and production systems.

First execute the lab tests and native round-trip; then complete the actual local baseline
and the smallest integrated source -> claim -> cited brief -> reviewed correction ->
capsule -> fresh-workspace reuse path. Only then broaden the module set.

End with changed files/commit/PR, exact tested launch commands and URLs, hardware/model
identities, verification results, remaining blockers and the first unfinished milestone.
Do not end with a generic plan or mark unfinished capabilities complete. When machine
permissions or required services are unavailable, finish independent code/tests and name
the precise missing runtime evidence. Never claim background progress without execution.

## Primary references to verify while implementing

- https://github.com/raphaelmansuy/edgequake
- https://edgequake.com/
- https://docs.ollama.com/faq
- https://docs.ollama.com/api/generate
- Current repository source, lockfiles and schemas take precedence over remembered APIs.
