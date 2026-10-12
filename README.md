# TACITUS

Political & Policy Second Brain — an independent workbench built on EdgeQuake.

**Development checkpoint, not a working application release.** The public fork retains
EdgeQuake; the Tacitus runtime/UI/storage are not integrated. Live baseline acceptance
is blocked on the tested Windows machine by missing Docker/Compose and Ollama.

The four new Rust crates implement source/assertion/review invariants, a scoped evidence
compiler, deterministic portable capsule file tools, proposed analysis components and
evaluation records. Developer tests and a substantive synthetic file round-trip run;
no real model benchmark, browser workflow or offline container acceptance has run.

- [Current implementation status](IMPLEMENTATION_STATUS.md)
- [Developer commands](docs/tacitus/getting-started.md)
- [Exact upstream pin and update procedure](UPSTREAM.md)
- [Architecture](docs/tacitus/architecture.md) and [policy domain](docs/tacitus/policy-domain.md)
- [Capsule format](docs/tacitus/capsule-format.md), [module authoring](docs/tacitus/module-authoring.md)
- [Security](docs/tacitus/security.md), [hardware/models](docs/tacitus/hardware-local-models.md)
- [Evaluation](docs/tacitus/evaluation.md), [troubleshooting](docs/tacitus/troubleshooting.md)
- [Verification results](docs/tacitus/verification.md)

Public source code does not authorize sharing research documents or capsules. Only the
newly authored fictional CC0 fixtures are intentional public research examples. The
provisional capsule format has no verified Praxis compatibility. No private Praxis
source or research data is included.

## Attribution

Built on [EdgeQuake](https://github.com/raphaelmansuy/edgequake), retaining its Apache-2.0
license, public history and applicable notices. No upstream endorsement is implied.
The [original upstream README](docs/tacitus/upstream-readme.md) is retained as upstream
documentation; its startup instructions are not Tacitus acceptance evidence.
