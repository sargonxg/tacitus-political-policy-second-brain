# Native-v1 extractive review briefs

An additive Rust module, `tacitus_analysis::cited_brief`, accepts an existing validated native-v1 `Package` and an explicit review request. It uses no model and does not convert native-v1 data to the separate policy-v2 graph contract.

The caller supplies a question, intended workspace, known-as-of and valid-at timestamps, ordered assertion IDs, maximum JSON packet bytes, current revoked source IDs and package revocation state. These access inputs must come from the host's authorized current state, never from imported claims. This library is not an authentication service or a revocation registry.

Selection follows the supplied priority list. It does not infer saliency, answer the question, assess truth or rank semantic relevance. Each retained assertion carries all native fields and exact cited excerpts with full source metadata. The metadata `text` remains empty; the excerpt is bound through the span and original source-text hash. Package scope, limitations, original artifacts and method caveats remain mandatory. Imported review records never grant approval.

Artifact-free and method-free packages can omit claims with explicit records. Packages containing opaque artifacts or methods are atomic: every claim must be selected and eligible, every source must be eligible, and the whole package must fit. Otherwise compilation fails without returning any output. This prevents opaque prose from reintroducing excluded claims. Methods are always downgraded to unreviewed proposals, including for direct library callers. Relations (`dependencies`, `challenges`, `supersedes`) fail closed until a grouped native-v1 resolver exists. References-only packages cannot generate invented excerpts. Expired, unavailable or revoked sources exclude their claims. Unknown and duplicate requested claim IDs fail. A budget too small for mandatory attribution and omission records fails instead of silently truncating.

The packet budget covers exact compact serde_json UTF-8 bytes, excluding the digest wrapper. This is not a token budget or JCS. Plain-text rendering revalidates the packet digest and has its own independent byte limit. It includes a readable view plus the entire qualified JSON appendix; it is deliberately not a compressed summary. Text and artifacts remain untrusted data and are not executed.

## Run

In the isolated delivered three-crate workspace:

```
cargo test --offline --locked --workspace
cargo run --offline -q -p tacitus-analysis --example cited_brief -- selected.tacitus.zip < request.json
cargo run --offline -q -p tacitus-analysis --example cited_brief -- selected.tacitus.zip --text < request.json
```

In the full repository first `cd edgequake`, then use the same example commands. The delivered lockfile belongs to the isolated validation workspace; full-repository tests are not established.

## Verification

54 tests pass in the isolated standalone three-crate release workspace, including 15 new extractive-brief tests. The combined workspace with the separate graph-layer prototype passes 67 tests; that graph module is not required by this change. Three actual native-v1 policy capsules were compiled: NYC heat (4 claims, 25,931 compact packet bytes), compute infrastructure (4 claims, 25,934 bytes), humanitarian access (5 claims, 30,361 bytes). All fit the 80,000-byte packet limit with no omissions. Plain-text review files have separate bounded output. All three actual capsules were also checked with a revoked source, partial selection and a budget one byte below the complete packet; each correctly refused the entire opaque export.

Those capsules contain explicitly labeled assistant-authored paraphrases linked to official sources. Hash/span checks establish integrity of those summaries, not verification of the official source contents, correctness, rights, or policy conclusions. This exporter has no API/provider calls, deployed UI, persistence, or native-v2/v3 migration. No accuracy, latency or compression improvement is claimed.
