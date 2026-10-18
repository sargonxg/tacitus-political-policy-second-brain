# Checkpoint-based change review

This small lab addition prevents newly available evidence from being silently omitted
when preparing a change-review brief. It does **not** detect semantic corrections, decide
which sources are authoritative, or demonstrate generally better salience.

## Run without installing anything

From the repository root, with Node >=22:

```sh
node tools/tacitus-policy-lab/cli.mjs review-demo
node tools/tacitus-policy-lab/cli.mjs review-demo --max-bytes 32000
node --test tools/tacitus-policy-lab/test.mjs tools/tacitus-policy-lab/benchmark.test.mjs tools/tacitus-policy-lab/change-review.test.mjs
node tools/tacitus-policy-lab/cli.mjs benchmark --iterations 100
```

The demo uses the existing fictional pilot, moves its known/valid cutoff one day forward,
and uses the earlier known cutoff as the review checkpoint. It outputs attributed exact
quotations and prominently names old evidence omitted by the byte budget. No model,
network call, accepted-judgment change, capsule writer or filesystem write is involved.
With `--max-bytes 700`, the new source cannot fit: the command fails with a clear message
and **no brief on stdout**. Increasing the budget to 32,000 includes all eligible demo
statements; it does not make them true or complete the analyst's judgment.

Programmatic entry point: `changeReviewContext(input, sinceKnownAsOf)` in `change-review.mjs`.
`input` has existing lab sources/assertions and a context contract. The checkpoint must
be an integer UTC epoch second at or before the current known-as-of time. This is a lab
workflow parameter, not an added native Assertion/SourceVersion/schema field.

## Selection and failure behavior

1. Run the existing `contextRequest` access, time, hash and span checks at its maximum
   250,000-byte context size. If this cannot fit all eligible records, stop explicitly.
2. An assertion is newly available when the latest of its own availability start and its
   cited sources' availability starts is strictly after the checkpoint. Equal timestamps
   are already known. Future/invalid/restricted/withdrawn records never reach ordering.
3. Place newly available records first, newest first; preserve original input order for
   ties and all older evidence. No labels, text cues, question overlap, profile weights
   or claimed source independence drive this ordering.
4. Reuse the existing compiler under the original byte limit. Report selected new records,
   omitted new records and omitted older records. If any new record is omitted, mark
   `blocked-new-evidence-omitted` and refuse `changeReviewBrief` output. Otherwise keep
   `needs-human-review`, even when every record fits.

The byte limit covers the serialized evidence packet, as before. Review metadata and
rendered Markdown are additional output, not a model-token budget. The function is a
trusted local review aid, not an authorization or authenticated approval boundary.

## Same-fixture comparison

[Current machine-readable run](benchmarks/2026-10-17-change-review-node24-linux.json)
retains input-order, lexical-overlap and all-fit reference arms and adds `change-first`.
All bounded arms receive the same candidates, cutoffs, question and 2,400-byte budget.
Every case has the same explicit earlier checkpoint available to every arm.

The critical-claim and qualification gold file is byte-identical to the preserved
[baseline benchmark](BENCHMARK.md). It was already inspected during development:
**this is an in-sample regression experiment, not a blind or held-out evaluation**.
New-evidence retention separately counts eligible records with availability after the
checkpoint. It is a deterministic timestamp metric, not a human importance score.

| Case | Input / lexical new-evidence retention | Change-first new-evidence retention | Critical coverage, input / lexical / change-first | All-fit critical coverage |
| --- | --- | --- | --- | --- |
| Pilot, before | 0/0 (null) | 0/0 (null) | 2/2 / 2/2 / 2/2 | 2/2 |
| Pilot, after | 0/1 | 1/1 | 2/3 / 2/3 / 2/3 | 3/3 |
| River, before | 0/0 (null) | 0/0 (null) | 1/2 / 1/2 / 1/2 | 2/2 |
| River, after | 0/1 | 1/1 | 1/3 / 1/3 / 1/3 | 3/3 |

Both updates are recovered, but **total critical coverage does not improve**. In the pilot,
the funding note is displaced by the correction. In the river example, the suspension
displaces the older authority statement; the safety condition is still missing. The candidate
also drops the French notice retained by input-order. All candidates preserve their selected
citations and exhibit zero ineligible IDs. The all-fit reference has a larger budget and
must not be described as an equal-resource competitor.

An additional negative test introduces an equally recent, irrelevant paint notice. It
receives priority and can displace useful older context. This is intentional evidence of
the method's limit, not a test claiming irrelevant material is desirable. No full decision
brief should be accepted merely because the timestamp retention gate passes.

## Verification

- 122 Node tests pass: 81 original, 31 benchmark and ten change-review tests.
- Both independent 100-iteration benchmark processes produce identical non-timing results.
- The recorded report includes SHA-256 fingerprints of runtime code, tests and fixtures.
- Candidate p50 compilation times range 0.513–0.642 ms and p95 0.921–1.090 ms on the
  observed Linux x64 / Node v24.19.0 shared cloud runtime (AMD EPYC 9V74 reported CPU).
  Three warmups precede 100 observations per arm/case, with rotating arm order. These
  exclude eligibility preparation, scoring, fixture construction, I/O, models and human
  work. First observed is not cold start. No speed advantage or generalization is claimed.
- No Rust, model, database, browser/full-app acceptance, native capsule round-trip,
  real-world analyst trial or deployment ran as part of this change.

The preserved baseline JSON's fingerprints describe the earlier baseline source snapshot,
not this later candidate. Its original runnable source ZIP remains available separately;
use the new change-review JSON for the current code's fingerprints.

## Remaining product work

Availability timestamps are not a complete revision-event log. Changed restrictions,
withdrawals, expiry, source corrections with unchanged availability, or newly discovered
missing dependencies need the existing product's revision/invalidation and review pipeline.
A source being recent does not verify a supersession relationship or justify changing an
accepted assertion. This helper does not infer those relationships.

Integrate only after independent review of the canonical API/types and authenticated
workspace boundaries. The public Workbench's newer native policy/capsule-v2 candidate and
this lab's legacy native-v1-shaped inputs are distinct. No new database, ontology/schema
migration, graph engine, portable archive or private PRAXIS compatibility is introduced.
The next meaningful evaluation needs held-out dossier/chronology labels and real analyst
review, including historical evidence displaced, maintenance effort and total time to an
acceptable brief.
