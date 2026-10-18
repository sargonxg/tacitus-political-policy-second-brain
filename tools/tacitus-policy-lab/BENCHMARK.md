# Policy-lab synthetic benchmark

This records the preserved three-arm baseline. The later [change-review experiment](CHANGE_REVIEW.md)
adds a fourth arm; its separate report contains fingerprints for the current code.

A runnable, dependency-free regression harness for **evidence selection and change-impact
review queues**, not an evaluation of an LLM, political truth, native capsules or the
full Workbench. Node >=22 is required. No install, account or model is needed.

## Reproduce

From the repository root:

```sh
node --test tools/tacitus-policy-lab/test.mjs tools/tacitus-policy-lab/benchmark.test.mjs
node tools/tacitus-policy-lab/cli.mjs benchmark --iterations 100
node tools/tacitus-policy-lab/cli.mjs benchmark --iterations 100 --out /tmp/new-policy-benchmark.json
```

`--out` must name a **new file in an existing directory**. Existing files are never
overwritten; default output is JSON on stdout. Iterations accept integers 1–1000
(default 25). Duplicate/unknown flags fail. On Windows, choose an existing local
output directory. `npm --prefix tools/tacitus-policy-lab test` also runs both suites;
`npm --prefix tools/tacitus-policy-lab run benchmark -- --iterations 100` runs the harness.
No npm installation is needed. The saved report contains only checked-in fictional
fixtures' IDs/metrics, source fingerprints, and the runtime's reported CPU/platform.

## Inputs, labels and comparisons

Two openly developer-authored, fictional CC0 dossiers each have five sources, evaluated
before and after a correction/suspension. The existing pilot fixture covers conditional
commitment, authorization versus spending and two copies from one declared source family.
The new river fixture covers advisory versus executive authority, a safety prerequisite,
a French notice and a later suspension. Nothing here states actual law or political facts.

All tasks ask a fixed policy question at explicit known-as-of and valid-at cutoffs. Each
bounded context gets **2,400 serialized UTF-8 bytes**, including the request envelope,
citations and review cues. This is not model token accounting. The budget is an illustrative
fixture choice, not a measured deployment optimum. No budget/ranker was tuned to create a
reported win. These are public development examples, **not held-out validation**.

- `input-order`: existing `contextRequest` greedy admission in fixture assertion order.
- `lexical-overlap`: the same compiler after sorting by count of unique question tokens
  present in the proposition. Tokens are lowercased Unicode letter/number runs; no stemming,
  translation, stop-word removal, learning or domain priors. Ties retain original order.
  This intentionally simple comparator is not a salience model.
- `all-eligible-reference`: the same compiler at its 250,000-byte maximum, verified to fit
  all eligible assertions. This is an all-fit reference, **not an equal-budget competitor,
  unlimited API, full-document baseline, or upper bound on analytical quality**.

All selectors receive the same legitimate evidence. Existing lab access/time filtering
runs first, and a truncated eligibility reference fails rather than hiding candidates.
Preparation time is recorded separately. The later evidence is additive: prior caveated
statements remain available, and post-correction labels require both history and correction.
No automatic supersession or accepted judgment rewrite occurs.

`benchmark-gold.json` is used only by the scoring layer, never imported by the selection
module. Its critical claim IDs, eligible IDs and literal qualification labels were authored
for these examples; they are not independently adjudicated. They contain no duplicate
critical claim IDs, and reprints do not earn extra critical coverage. To evaluate another
selector, add its trusted local implementation to `benchmark-selection.mjs` and retain
all existing comparators. Do not load executable plugins from a dataset or capsule.

## Metric definitions

Every fraction includes numerator, denominator and rate. A zero denominator produces
`null`, not an invented perfect score.

- Critical claim coverage: unique gold critical IDs selected with the original literal
  proposition, epistemic type and bound valid citation, divided by all critical IDs.
  This is fixture-label coverage, not semantic entailment or a quality judgment.
- Qualification coverage: required gold substrings retained in those supported statements
  divided by all required qualifiers. Selected-qualification integrity counts required qualifier occurrences
  across selected rows, including duplicate rows, and flags corrupted copies. Reporting both prevents perfect selected-output integrity
  from hiding omissions. In these exact-extraction fixtures each critical ID has one
  qualifier, so qualification coverage equals critical coverage; this is not an independent
  semantic success measure.
- Citation integrity: quotes matching the existing `passage` operation against immutable
  source/version/hash/language/UTF-8 spans and declared source family, divided by citations.
  Source text hashes are checked by the existing lab validator. Literal claim support is
  scored separately: a valid quote cannot rescue a rewritten proposition.
- Ineligible IDs: selected IDs absent from the task's independently declared eligible set.
  The current fixtures use this to expose later-evidence leakage. Separate tests exercise
  restricted/withdrawn data and independent knowledge/valid-time boundaries.
- Declared source families: distinct family strings behind valid citations; **not verified
  independence**. Duplicate output rows are explicitly counted.
- Change-impact queues: eight reviewable records, with four independently labeled affected
  records. Source seed `delegation@1` is excluded from the reviewable universe. Compare
  review-all with existing transitive `dependencyImpact`, then deliberately remove the
  only link into `participation-assumption`. Recall, precision, missed IDs and queue sizes
  are all retained. These are work-item counts, **not measured analyst effort saved**.

## Recorded run: 2026-10-17

[Machine-readable report](benchmarks/2026-10-17-node24-linux.json), generated on Linux x64,
Node v24.19.0, reported CPU AMD EPYC 9V74 80-Core Processor. Source/fixture SHA-256 values
in the report identify what ran. Repository base: `fbda010399476409fc409dd6ddddcbcd53071b3c`.

| Case | Input-order critical / qualification coverage | Lexical critical / qualification coverage | All-fit critical / qualification coverage |
| --- | --- | --- | --- |
| Pilot, before | 2/2; 2/2 | 2/2; 2/2 | 2/2; 2/2 |
| Pilot, after | 2/3; 2/3 | 2/3; 2/3 | 3/3; 3/3 |
| River, before | 1/2; 1/2 | 1/2; 1/2 | 2/2; 2/2 |
| River, after | 1/3; 1/3 | 1/3; 1/3 | 3/3; 3/3 |

Both bounded arms select exactly the same records. **No lexical improvement was observed.**
They miss the later correction/suspension; river selection also misses the safety condition.
All selected citations are intact (input-order and lexical: 2,2,3,3 per case; reference:
4,5,4,5), and every arm has zero ineligible IDs. Perfect quote integrity therefore
coexists with serious material omissions. All-fit coverage benefits from a larger budget.

| Change comparator | Queued records | Affected recall | Queue precision | Missed affected |
| --- | --- | --- | --- | --- |
| Review all | 8/8 | 4/4 | 4/8 | 0 |
| Declared dependency closure | 4/8 | 4/4 | 4/4 | 0 |
| Missing-link negative control | 1/8 | 1/4 | 1/1 | 3 |

The incomplete graph looks efficient and precise while missing three affected records.
Dependency completeness must be checked; these counts do not include the work of creating,
maintaining or reviewing the graph, or reviewing records with unequal difficulty.

Each arm/case has three warmups and 100 measured compilations with rotating arm order.
Observed p50 values range 0.509–0.743 ms and p95 0.756–1.273 ms on this one shared cloud
runtime. Timings include hashing/serialization, exclude fixture construction, scoring,
output I/O, eligibility preparation, model calls and human work. The separate first
observed value is **not a cold-start measurement**. Do not infer speed superiority or
production latency from these tiny fixtures; no statistical significance is claimed.

## Verification and remaining gates

- 111 Node tests passed: 81 existing plus 30 benchmark tests.
- Two reports in the tests match exactly after removing observed timing fields.
- Tests cover equal budgets, actual serialized bytes, no mutation, gold isolation,
  chronology, half-open intervals, restricted/withdrawn sources, quote/hash/span/version/
  language corruption, fabricated propositions, missing qualifications, empty output,
  duplicate records/common origin, missing dependencies, strict CLI options and overwrite refusal.
- No canonical Rust compiler/validator, native ZIP writer, production code, model, cloud
  account, schema, dependency or security configuration changed.

Next meaningful evidence would be independent analyst labels on held-out dossiers and
chronologies, then a same-model comparison including full documents, unmodified retrieval,
question-bounded context and reviewed memory. Include ingestion/indexing, correction and
human review costs. Human quality, analyst minutes, model quality/latency and production
performance are explicitly `null` in this report. Those gates have not run.

Implementation uses the installed Node [test runner](https://nodejs.org/docs/latest-v24.x/api/test.html)
and [performance hooks](https://nodejs.org/docs/latest-v24.x/api/perf_hooks.html), with no extra packages.
