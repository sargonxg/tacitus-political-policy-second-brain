# Persistent policy MVP candidate

This implementation extends EdgeQuake's existing API, authentication, PostgreSQL,
ingestion/retrieval and browser application. It is source code awaiting compilation
and cloud/runtime acceptance, not a release. No execution results are claimed below.

## Analyst workflow

1. Log in, choose a workspace and upload authorized text or Markdown. Text PDFs use
   EdgeParse; vision/scanned PDFs are outside the default text-model acceptance scope.
   Wait for document ingestion and inspect errors in Documents.
2. Ask a question. Override the simple English task suggestion with Answer, Decision
   brief, Competing hypotheses or Positions and commitments. Fast context is default;
   deeper mode raises evidence/output budgets. Select documents explicitly for dossiers
   larger than 32 documents. Set known-as-of and world-valid cutoffs when relevant.
3. Open each citation to its exact extracted version/byte span. Source original-file
   hashes remain unknown until bytes are independently verified. Inspect context omissions.
4. Review attributed assertion edges in Graph's accessible table. The retained EdgeQuake
   2D graph is also available; its extracted relationships do not acquire Tacitus human
   review status merely by appearing there.
5. Record an authenticated correction/review rationale. Propose a scoped cognitive method,
   or ask the local model to propose one from pinned source passages. Generation does
   not use an earlier generated answer as new evidence. Bind the method to exact source
   spans, declare failure conditions, then explicitly accept or
   reject it. To reuse, select the method and confirm applicability; unknown prerequisites
   guard use, false prerequisites block it and changed provenance makes it stale.
6. Declare source export rights and a license, then inspect every capsule field/file.
   Export only the exact preview hash. Evidence-included native capsules support fresh
   workspace re-ingestion. References-only packages still disclose assertions/methods and
   cannot fabricate absent evidence. Imported attestations never grant acceptance.
7. Import into a fresh workspace, wait for ingestion, inspect/review the method and reuse
   it on a new question. Imported methods are declarative data, never executable plugins.
8. Upload changed evidence as a new document. In the source inspector, provide its ID
   to record it as a replacement of an earlier source. It does not inherit authorship or
   redistribution attestations. Old passages remain pinned and authorized historical edges
   show `review_required`; dependency review flags claims, methods and recorded analyses.
   It does not automatically rewrite earlier judgments.

The brief harness requires judgments, alternatives, status quo, options, authority,
resources, implementation, affected groups and signposts. The hypotheses harness requires
distinct alternatives and an evidence-pair matrix without invented probabilities. The
commitments harness retains exact wording, conditions, authority/deadline unknowns and
implementation status. Empty commitments are valid when no undertaking is evidenced.

The context compiler uses whole paragraphs, question terms, EdgeQuake retrieved chunks
and qualification cues. Shared origins are labeled and exact duplicate text from the same
origin is deduplicated. This heuristic is not a measured salience improvement. Context
budgets are labeled conservative UTF-8 byte estimates; Ollama token counts are actual
response counts. Hash/quote validation proves integrity, not substantive evidential support.

## Authenticated CLI

Run Node 22+ through the same localhost/SSH-tunneled API. Privately set `TACITUS_TOKEN`
(a real authenticated access token), `TACITUS_TENANT` and `TACITUS_WORKSPACE`; never
commit tokens or paste them into issue/PR logs. Use a shell/session secret mechanism.

```sh
node scripts/tacitus-policy.mjs upload public-document.md
node scripts/tacitus-policy.mjs status
node scripts/tacitus-policy.mjs ask request.json
node scripts/tacitus-policy.mjs graph
node scripts/tacitus-policy.mjs methods
node scripts/tacitus-policy.mjs impact
node scripts/tacitus-policy.mjs preview selection.json
node scripts/tacitus-policy.mjs export reviewed-selection.json new-output.capsule
node scripts/tacitus-policy.mjs import new-output.capsule
```

`request.json` example (document IDs must belong to the selected workspace):

```json
{
  "question": "Which implementation safeguards depend on a separate appropriation?",
  "task": "brief",
  "document_ids": [],
  "deepen": true,
  "known_as_of": null,
  "valid_at": null,
  "method_ids": [],
  "facts": {}
}
```

Additional commands: `inspect SOURCE_ID VERSION START_BYTE END_BYTE`, `sync INPUT.json`,
`propose INPUT.json`, `review INPUT.json`, `metadata INPUT.json`, `replace INPUT.json`,
`analyses`, and `cancel ANALYSIS_ID`. Metadata/replacement input includes `source_id` and
`expected_version`; replacement additionally requires a ready authorized `document_id`.
Review input uses `target_id`, `target_kind`, `expected_fingerprint`, `decision`, `rationale`.
Export selection includes `title`, `source_keys`, `assertion_ids`, `method_ids`, `profile`,
`evidence_mode`, and the preview's `expected_preview_hash`.

The native schema is [capsule v2](../../schemas/tacitus/capsule-v2.schema.json).
Rust semantic validation is authoritative. The generator's `--v2` switch keeps the v1
artifact unchanged; v1 import follows an explicit migration and downgrades attestations.
HTTP endpoints are registered in OpenAPI; the new manual TypeScript client is provisional
until cloud compilation/code generation is verified. Private Praxis fidelity is unclaimed.

## Cloud verification — all NOT RUN

On a reviewed clean checkout of this branch on the chosen Linux host, after installing
the approved prerequisites:

```sh
TACITUS_CLOUD_EXECUTE=1 bash scripts/tacitus-cloud-checks.sh
node scripts/tacitus-stack.mjs init cloud
node scripts/tacitus-stack.mjs check
node scripts/tacitus-stack.mjs build
node scripts/tacitus-stack.mjs download
node scripts/tacitus-stack.mjs up
# Set the authenticated CLI environment privately.
TACITUS_CLOUD_EXECUTE=1 node scripts/tacitus-cloud-acceptance.mjs
node scripts/tacitus-stack.mjs restart
TACITUS_CLOUD_EXECUTE=1 node scripts/tacitus-cloud-acceptance.mjs after-restart
```

The acceptance runner creates synthetic workspaces and a newly randomized marker,
waits for real ingestion, runs all three harnesses, inspects exact citations, reviews a
method, exports/imports through the native API, tests cross-workspace denial and guarded
reuse, introduces changed text and checks dependency impact. It has no mock mode.
Receipts are private ignored `.tacitus/cloud-acceptance.json`; a repeated run refuses to
overwrite them. A timeout/validation failure is a real acceptance failure, not a waived gate.

For the real browser check, set `E2E_USERNAME`, `E2E_PASSWORD`, `TACITUS_TENANT`,
`TACITUS_WORKSPACE` privately and use a synthetic workspace:

```sh
cd edgequake_webui
pnpm exec playwright install chromium
TACITUS_CLOUD_EXECUTE=1 pnpm exec playwright test --config playwright.config.noserver.ts tacitus-policy-real.spec.ts
```

Record separately: source commit, image IDs, OS/RAM/disk/GPU identities, extensions and
migration ledger, actual Ollama environment `OLLAMA_NO_CLOUD=1`, model digests/dimensions,
real embedding/retrieval receipts, internal runtime-network inspection, provider override
rejections, offline repeat, and clean-volume rehearsal. Browser storage/cookies must remain
private. The harness does not by itself prove every authorization/cache/provider route.

Seven inherited insecure examples and RSA `RUSTSEC-2023-0071` still block release.
No finding is waived. Human scoring and all paid/frontier comparisons remain NOT RUN.
The five-baseline controlled evaluation runner is unfinished; no efficiency advantage is
claimed. The first unfinished milestone is cloud compilation and real ingestion/inference.

## Operating limits

Single API instance only: jobs persist and cancellation is polled, while startup marks
old unfinished jobs interrupted. This is not a distributed task-queue implementation.
Selection is capped at 32 documents; each extracted source is capped at 2 MiB, context
at 24 passages and workspace record reads at 2048 entries. Exceeding the record bound
fails visibly instead of silently omitting history. Migration/pagination beyond this MVP,
full domain packs, complete temporal commitment diffs, context decision contracts,
complete memory taxonomies and calibrated evaluations remain unfinished.
