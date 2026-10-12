# Current checkpoint: start here before the long Praxis prompt

The new `tools/tacitus-policy-lab` is a dependency-free deterministic development slice.
It is NOT the model-driven EdgeQuake/Tacitus application or a replacement Rust kernel.
The repository's original implementation-status warnings remain in force for the full app.

2026-10-12 continuation: the 81 Node tests were reproduced on Windows; 6 launcher
contract tests and 28 native Rust tests now pass. Actual Chrome localhost navigation
and selected lab interactions work. Lab-generated native inputs completed the real Rust
preview/pack/validate/import/fresh-workspace compile. Use
`node scripts/tacitus-lab-roundtrip.mjs` to reproduce. Imported methods now lose foreign
acceptance labels; provenance and assertion/source workspace binding are checked.

The user declined application/model installations on the low-space laptop and selected
self-hosting preparation for a host chosen later. See [self-hosting](self-hosting.md).
Docker/source-build startup, database persistence, real inference, provider enforcement
and the integrated Next.js policy workflow still have not run. Statements below describe
the preceding build environment; they are preserved as historical evidence.

Current verified counts supersede earlier draft counts in planning text: **81 Node tests
and 12 isolated browser-component checks passed**. Native manifest/data structurally match
the inspected JSON Schema. Browser localhost navigation was blocked; components were
therefore tested with embedded assets and test-only cryptography. Native Rust, Docker,
Ollama, database/API integration, Next.js and full browser E2E remain NOT RUN here.
See `tools/tacitus-policy-lab/VERIFICATION.md` for the exact boundaries.

## Immediate commands from the root

```sh
node tools/tacitus-policy-lab/cli.mjs doctor
node --test tools/tacitus-policy-lab/test.mjs
node tools/tacitus-policy-lab/cli.mjs serve
```

Open http://127.0.0.1:4317. Node 22+; no npm installation required. Save tab state to an
explicit file before closing. No model is connected, and generated text is extractive
preparation, not an AI analytical answer.

## Next native integration sequence

1. Review the proposed branch changes and preserve the inherited audit/publication gate.
   Do not enable Actions, merge main, waive findings or expose private data automatically.
2. Reproduce lab tests and actual browser HTTP behavior on the development machine.
3. Execute the native Rust preview/pack/validate/import/compile round-trip on generated
   inputs. Resolve schema fidelity before claiming native/Praxis compatibility.
4. Complete the actual EdgeQuake local model/embedding baseline with a random novel source.
5. Implement canonical persistence and authenticated API routes in the existing stack.
6. Port policy context, evidence inspection, reviewed memory and capsule workflows into
   the existing frontend. Keep the lab as an optional regression tool, not a second app.
7. Test source -> cited brief -> reviewed correction -> memory -> capsule -> fresh-workspace
   reuse -> changed-evidence review, then broaden policy modules and fair model evaluations.

The extensive execution contract is `docs/tacitus/PRAXIS_BUILD_NEXT.md`. Read it after this
checkpoint; do not repeat repository creation or replace the existing EdgeQuake fork.
