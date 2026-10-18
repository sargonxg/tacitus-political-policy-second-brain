# Matched retrieval and synthetic collection checks

## What the experiment establishes

This is a deterministic evidence-retention harness. It invokes the actual reviewed Rust knowledge_capture CLI and compares its context with two independent corpus-wide lexical baselines: whole-source retrieval and line-chunk retrieval. The baselines use simple term overlap, not BM25, embeddings or a tuned production retriever. No generator/model is invoked. No generated-answer quality, semantic correctness or production latency claim follows.

Six fixed queries over the same immutable synthetic and historical EN/FR dossiers are tested at 8,192, 16,384 and 32,768 serialized UTF-8 bytes. All three methods receive the same original source records, question and temporal/access policy. The capsule additionally uses explicitly authored claim/entity/dependency annotations: that human information and labor is a real input cost, not a free algorithmic gain. Corpus records are checked through native packing before evaluation. Expected support sets are illustrative author-specified cases, not independent held-out judgments.

The frozen initial 18-case result is retained unchanged. No retrieval implementation was improved after observing it. Timings are variable; deterministic payload hashes/coverage must reproduce exactly.

## Results, including the regression

Among 15 evidence-bearing query/budget pairs, capsule exact support-set preservation has 5 wins, 7 ties and 3 losses against each baseline. The three no-match cases are excluded from that tally and reported separately.

At 16 KiB:

- `withdraws`: capsule retains the linked original commitment and withdrawal. Both lexical baselines retrieve only the withdrawal.
- `heat exercise`: all methods retain the relevant source evidence. The line-chunk baseline avoids unrelated source lines without needing a graph.
- `discours`: the capsule retains five linked records through a manually authored French cue and dependency annotations. Baselines have no matching original-source token. This illustrates the value and cost of explicit annotation; it is not cross-language semantic retrieval.
- `applicable`: both baselines retrieve the original French evidence; the capsule matcher misses it because it searches assertion/label text rather than the original span. This is a genuine recall regression and should be addressed before claiming better multilingual retrieval.
- `publication institutions`: all methods retain the required source evidence.
- The unrelated astronomy query yields no evidence in all methods.

Coverage means required exact source spans are present (baseline) or the exact full annotated assertion is present (capsule). It is deliberately not semantic support or factual answer accuracy. Baseline source coverage does not prove that an analyst's separately authored interpretation, conditions or recommendation was conveyed. The formats carry different explanatory metadata, so byte differences are informative, not a proof of superior efficiency.

## Collection scale and a bounded update

Separate synthetic experiments create 100 and 300 documents, 200 and 600 line chunks, and 4 and 12 native-v2 shards containing 25 sources each. The existing 32-source/256-assertion limits are preserved. Routing scores every chunk across the collection before selecting a shard, with no hand-selected target shard.

For the predeclared exact programme query, capsule context retains both the target commitment and proposed withdrawal in 12,642/12,646 bytes including the collection wrapper. Whole-document and chunk baselines retain the same target evidence in fewer bytes. A single source revision changes/rebuilds one shard, updates its content-hash manifest and leaves the original archive available. The changed source and graph move to recording checkpoint 301; the lexical index entries are refreshed and route to revision 2. A request at checkpoint 300 refuses the new shard. The original archive remains available for that earlier snapshot.

This demonstrates synthetic collection mechanics and one incremental update. It is not a production persistent registry, atomic update service, real document extraction-quality test, arbitrary multi-shard query allocator or authorization service. The exact-key query selects one shard. A broad query across many shards needs an explicit collection-level budget allocator and access/currentness checks before production use.

Build, lookup, reuse and revision-repack timings are reported as environment-specific samples. They include relevant Python or subprocess work. They must not be compared as a fair Rust-versus-Python speed contest. Python peak RSS excludes child-process memory and is a cumulative process peak. No model tokens, cost, drafting quality or reviewer-time measurements were performed.

## Reproduce

Use the repository source at the public commit in frozen/environment-final.json. Prepare the isolated capsule workspace using scripts/prepare-capsule-workspace.py, then build the knowledge_capture example with Rust 1.95.0 and the locked dependencies. Set BINARY to that executable and FIXTURES to the repository's edgequake/crates/tacitus-analysis/tests/fixtures directory.

    export TACITUS_FIXTURES="$FIXTURES"
    python3 -m unittest -v test_compare
    python3 compare.py --binary "$BINARY" --fixtures "$FIXTURES" --output fresh-comparison.json
    python3 scale.py --binary "$BINARY" --template "$FIXTURES/knowledge/capture.json" --output fresh-scale.json

Output files must not already exist. Each comparison payload is repeated three times and hashed for exact determinism. Scale archives and manifests are content hashed. frozen/deterministic-final.json removes timing and process-RSS fields only; environment-final.json pins source, fixtures, harness and binary. Generated synthetic test data has no private or third-party source material.

The broader 35-case evaluation blueprint remains proposed. This harness exercises a narrow subset of its themes, including source preservation, correction closure, unknown-time refusal and multilingual omission. It does not claim all 35 acceptance cases have run.
