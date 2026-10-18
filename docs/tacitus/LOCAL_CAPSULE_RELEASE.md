# Run the TACITUS capsule developer release

This is the runnable **capsule-only developer workflow**. It creates portable native-v2 research records and produces deterministic cited review documents. It requires no database, model credentials, paid model, Docker or Bun. The larger EdgeQuake application and hosted marketplace have separate uncompleted acceptance gates.

## Local computer or a fresh cloud development machine

Use a machine you control, Python3.11+, Node22+ and the official Rust1.95.0 toolchain with Clippy and rustfmt. Obtain Rust through https://rustup.rs/ and Node through https://nodejs.org/ . Installing those tools is a separate machine-owner action. The commands below do not install them, change settings or start a public server.

Clone this repository, check out the reviewed release commit, then run from its root:

```
python3 scripts/check-capsule-release.py ../tacitus-capsule-output
```

The output directory must be new. The script refuses to overwrite it. Initial Cargo resolution downloads only the locked official-registry dependencies of four TACITUS crates. For a populated cache:

```
python3 scripts/check-capsule-release.py ../tacitus-capsule-output-offline --offline
```

The same command works on a cloud development machine with those prerequisites. It does not provision a cloud service, bind a public address or provide hosted authentication. No secrets should be added to the fixtures.

## What the command verifies and produces

1. Copies exact capsule source files into an isolated workspace and uses a capsule-only lock whose package identities, sources and checksums are verified against the original repository lock. It records source hashes in `workspace/source-receipt.json`. It does not compile unrelated application packages or run the PDFium download path.
2. Runs four-crate tests, Clippy with warnings denied, formatting, example builds, the Node policy-lab/catalog tests and schema consistency checks.
3. Authors the fictional cooling-pilot note, strict-validates its archive, imports and inspects it, then exports JSON/text cited reviews. The native review document includes its layered context, source metadata, qualifications and temporal status. Null publication/world-validity remains unknown; a verified-date request refuses it. The hypothesis and exception survive.
4. Creates all three corrected source-linked policy capsules: NYC heat resilience, compute infrastructure governance and humanitarian-access analysis. Their five claims each appear in full qualified review documents under `real-capsules/`. The embedded evidence is original TACITUS paraphrase/analysis with publisher URLs, not authenticated publisher text.
5. Generates the scoped compact examples and byte-budget results under `compact/`. At the reviewed 32 KiB budget each real example retains three findings; 16 KiB is an omission-only receipt with zero findings; 8 KiB refuses. Zero findings must not be treated as an answer.
6. Saves command logs and a final `receipt.json`. A successful receipt establishes these structural checks only.

Inspect `workspace/evidence/review.txt` for the fictional example and `real-capsules/*-review.txt` for the source-linked cases. Inspect the JSON equivalents to see the graph/context package and exact source references. No LLM writes these documents.

## Individual authoring and review

After the check, use the generated workspace's `target/debug/examples` binaries (or the explicitly configured Cargo target directory). See [native-v2 authoring](AUTHOR_NATIVE_V2.md) for manual commands. The authoring declaration is for original authorized summary evidence only. It does not establish ownership, license eligibility, source truth or permission to publish. No automatic native-v1 conversion occurs.

The Node catalog/UI remains a separate v1 developer tool. It rejects native-v2 capsules; there is no claim of a v2 web marketplace bridge. Its HTTP/security tests are included, but visual/keyboard/accessibility acceptance remains outstanding.

## Release boundaries

- Original summary licenses and rights are declarations. Hashes bind bytes, not truth, authorship, endorsement or expert review.
- Unknown source validity cannot satisfy verified-date requests. Observation periods and forecasts are separate from publication/recording dates.
- Opaque artifacts remain atomic in the full review. Compact export uses explicit reviewed profiles/mappings and preserves the specified guards/closure; it is not arbitrary marketplace ingestion.
- Hosted multi-user permissions, payments, public uploads, production database persistence, live PRAXIS attachment and deployment are not provided by this run path.
- Full-application builds additionally require PDFium/runtime/WebUI prerequisites. The four internal TACITUS crate package versions follow the application release train. Native formats and integer package versions are separate contracts. Do not bypass those gates by calling this a full application release.
- Repository CI and required review/merge approvals remain required. The library workflow now includes stacked PR base branches without expanding its `contents: read` permission. Absence of checks is not success.

## Source history

This consolidation includes PR6–13. The integration retains both native-v1 cited briefs and native-v2 workflow modules. Their prior frozen receipts remain historical evidence; the fresh release receipt is the reproducible acceptance record for this combined source. No source-backed example is converted into a claim of expert-reviewed advice.
