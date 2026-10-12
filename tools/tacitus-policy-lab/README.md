# TACITUS Policy Lab 0.1

A working deterministic development slice in the EdgeQuake-based repository, **not the
finished model-driven Tacitus application**. Node.js 22 or later is required. No npm
installation, Docker, Rust toolchain, model download, paid API or cloud account is
required for this laboratory. Hardware/electricity remain the user's responsibility.

The existing Rust crates remain the intended authoritative production implementation.
The lab is a small, testable interface and development adapter, not another database or
replacement knowledge graph.

## Run from the repository root

```sh
node tools/tacitus-policy-lab/cli.mjs doctor
node --test tools/tacitus-policy-lab/test.mjs
node tools/tacitus-policy-lab/cli.mjs demo
node tools/tacitus-policy-lab/cli.mjs serve
```

Open **http://127.0.0.1:4317**. Use `serve --port 4318` for a port conflict. Ctrl+C stops it.
The command syntax also works in PowerShell, but this build was tested on Linux only.
No `npm install` is necessary. The package also provides `npm --prefix tools/tacitus-policy-lab test`.

The server binds to loopback, validates Host, serves only an asset allowlist, and has
no write/private-file API. Data entered in the interface stays in the browser tab.
**Save session explicitly before closing.** Session files and references-only pack
inputs may still contain sensitive quotations. Inspect before sharing; never commit them.

## Working capabilities

- 24 policy concepts, 12 typed relation definitions and source-assertion-referenced
  relation proposals. This is an application ontology, not an OWL reasoner or an
  endorsed international standard.
- Six trusted handlers for semantic review cues, extractive brief preparation,
  competing-hypothesis diagnostics, dependency impact, method proposals and native
  capsule pack inputs. No imported executable handlers are loaded.
- Immutable-text SHA-256 checks, attribution, source families, and exact half-open
  UTF-8 byte citation spans with original-language metadata.
- Explicitly selected context with workspace, restricted/withdrawn and known/valid-time
  filters; byte-budget accounting, omissions and reproducible receipts. This does not
  implement vector retrieval, actual model tokenization or a learned salience ranker.
- English lexical review cues for conditional commitments, funding stages, attributed
  allegations, affiliation/control, causal language and embedded instructions.
  Cues are not semantic entailment, truth verification or sufficient injection defense.
- Cognitive method proposals with provenance, scope, temporal/exception/prerequisite
  gates and staleness. Unknown conditions stay guarded. No authenticated human review.
- Editable hypothesis consistency values and declared source-family counts, not
  calibrated probabilities or verified independence. Transitive dependency impact
  identifies what needs review without rewriting accepted judgments.
- Native Rust v1 Manifest/CapsuleData input preparation, optional authorized evidence,
  references-only limitations, and explicit import/export of local lab sessions.
  The lab does NOT create or validate a second ZIP capsule format.

## Try it

Load the fictional dossier, prepare context and inspect a source citation. The initial
cutoff excludes the later correction. Set both UTC time fields to a day later and
prepare again to include it. Conditional participation and budget authorization remain
attributed source wording rather than upgraded facts. Inspect guarded memory and the
hypothesis matrix. Paste a new source with explicit author/origin/time and prepare a
new context; use a cutoff after its actual retrieval time.

All checked-in demo documents are newly authored, fictional CC0-1.0 fixtures. No real
internal policy material, private Praxis source, personal records or API keys are included.

## Native capsule handoff

```sh
node tools/tacitus-policy-lab/cli.mjs demo --out .tacitus/policy-lab-demo
```

The destination must be new. The command refuses overwrite and creates manifest.json,
data.json, evidence/, request.json, session.json and brief.md. The request/session are
lab artifacts, not new canonical capsule schemas. The browser exports combined
references-only pack-input JSON, not a validated .capsule file.

Use the existing Rust toolchain/CLI to finish the native path:

```sh
cargo run --locked --manifest-path edgequake/Cargo.toml -p tacitus-capsules -- preview .tacitus/policy-lab-demo/manifest.json .tacitus/policy-lab-demo/data.json .tacitus/policy-lab-demo
cargo run --locked --manifest-path edgequake/Cargo.toml -p tacitus-capsules -- pack .tacitus/policy-lab-demo/manifest.json .tacitus/policy-lab-demo/data.json .tacitus/policy-lab-demo .tacitus/policy-lab-demo.capsule
cargo run --locked --manifest-path edgequake/Cargo.toml -p tacitus-capsules -- validate .tacitus/policy-lab-demo.capsule
```

These native command forms were inspected in the repository, **not executed here**:
Cargo is unavailable in the build environment. Generated manifest/data passed the exact
native JSON Schema; native Rust semantic validation and ZIP round-trip remain necessary.
Methods export as proposals, reviews remain empty, and imported lab sessions cannot
import review authority. Private canonical Praxis compatibility is unverified.

## Verified and unverified

Read [VERIFICATION.md](VERIFICATION.md). **81 Node tests passed** on Linux/Node 22.16.0,
including real local HTTP server and CLI file operations. Schema validation passed.
**12 isolated browser-component checks passed**, including source inspection, temporal
filtering, novel-source input and session/pack downloads. The component test used
embedded local assets and test-only cryptography because browser navigation to localhost
was administrator-blocked. It is NOT actual browser HTTP/CSP/module-loading E2E.

No Cargo/Docker/Ollama live run, Next.js build, authenticated API integration, PostgreSQL
persistence, graph/vector retrieval, LLM generation, native Rust capsule round-trip or
frontier comparison has run here. Existing audit findings and Actions settings remain
unchanged. Neither these tests nor a draft PR establish production readiness.

Continue with [the checkpoint](../../docs/tacitus/POLICY_LAB_CHECKPOINT.md) and
[the extensive Praxis implementation prompt](../../docs/tacitus/PRAXIS_BUILD_NEXT.md).
Port useful interactions/golden examples into the existing authenticated API/frontend;
implement canonical invariants in the Rust crates and avoid permanent duplicate engines.

Built on EdgeQuake. Upstream Apache-2.0 licensing/attribution is retained. New lab code
is Apache-2.0; the fictional fixture text is intentionally CC0-1.0.
