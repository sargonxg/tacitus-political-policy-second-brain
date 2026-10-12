# TACITUS

## Political & Policy Second Brain

A local-first political and policy research workbench built on
[EdgeQuake](https://github.com/raphaelmansuy/edgequake). The intended workflow is:

**Frame a decision → import evidence → inspect claims → analyze → review → write a cited brief → save a capsule → reconsider when evidence changes.**

**Current status: early development checkpoint, not a functioning Tacitus application release.**
The upstream backend and frontend are retained. The new Tacitus libraries work independently,
but the branded UI, persistent API/storage integration and real local-model workflow remain unfinished.

The canonical project branch is **main**. `tacitus/development` tracks this checkpoint.
`edgequake-main` is the inherited upstream branch, not the Tacitus project entry point.

## What is implemented and tested

| Component | Current capability | Boundary |
|---|---|---|
| `tacitus-core` | Source versions, immutable hashes/spans, assertions, temporal/decision contracts, review fingerprints and local-model identity policy | Not yet enforced by upstream HTTP routes/provider constructors |
| `tacitus-capsules` | Deterministic bounded ZIP packages; preview, validation, export/import, fresh-workspace reuse and inspectable context compilation | Developer file tools; no capsule UI or verified Praxis compatibility |
| `tacitus-analysis` | Cited brief proposals through a model interface, hypotheses matrix validation, commitment records and change-impact dependency closure | No production model adapter or persisted end-to-end workflows |
| `tacitus-evals` | Comparison records, equal-access checks and explicit pending human ratings | No real benchmark runner/results yet |

**25 deterministic Rust tests passed**, together with scoped Clippy, formatting and schema checks.
A substantive three-source capsule export → fresh-workspace import → lexical compilation passed.
A fresh local checkout also passed the library suite. Three clearly fictional dossiers contain
15 newly authored public CC0 documents; they are not evidence of general model superiority.

Testing occurred on Windows 11 x64, Rust 1.95.0, PowerShell and Windows Git Bash.
Linux/WSL, container profiles, real inference, embeddings, graph ingestion, browser journeys
and offline application restart have not been accepted. Docker/Compose and Ollama were absent
on the test host. **There is no verified application startup command or localhost URL yet.**

## Run the current developer checks

Source-development checks require Rust 1.95; they are not an application installation procedure.
From this repository in PowerShell:

```powershell
.\scripts\tacitus.ps1 doctor
.\scripts\tacitus.ps1 test
```

`doctor` returns exit 2 when Docker is missing. `test` runs the new libraries in a locked,
isolated workspace with cached crates only. For initial development dependency downloads:

```powershell
cargo test --locked --manifest-path edgequake/Cargo.toml -p tacitus-core -p tacitus-capsules -p tacitus-analysis -p tacitus-evals
```

Optional developer schema/file checks require Python; the file example also requires `jsonschema`:

```powershell
python scripts/generate-tacitus-schema.py --check
python scripts/tacitus-file-example.py
```

The file example performs a real local capsule round-trip using intentional fictional data.
It uses **lexical retrieval only** and calls no model, API or external service. Generated
files stay under ignored `.tacitus/`. Normal `setup`, `up`, `demo`, `smoke`, `benchmark`
and `down` launch operations are explicitly NOT IMPLEMENTED.

## Documentation and next milestones

- [Implementation status](IMPLEMENTATION_STATUS.md): implemented, tested, blocked and unfinished work.
- [Verification evidence](docs/tacitus/verification.md): exact commands, results, failures and security findings.
- [Getting started](docs/tacitus/getting-started.md) and [troubleshooting](docs/tacitus/troubleshooting.md).
- [Upstream pin/update procedure](UPSTREAM.md), [architecture](docs/tacitus/architecture.md), [policy domain](docs/tacitus/policy-domain.md).
- [Capsule format](docs/tacitus/capsule-format.md), [JSON Schema](schemas/tacitus/capsule-v1.schema.json), [module authoring](docs/tacitus/module-authoring.md).
- [Security boundaries](docs/tacitus/security.md), [hardware/local models](docs/tacitus/hardware-local-models.md), [evaluation](docs/tacitus/evaluation.md).

Next: prove the unmodified upstream baseline with actual local chat/embedding models,
PostgreSQL extensions and document ingestion/retrieval. Then integrate authenticated
Tacitus API/storage and the existing frontend before accepting the four analyst workflows.
The complete application build brief remains unfinished.

## Publication, privacy and security

Public code does **not** authorize sharing private research or capsules. Only intentional
fictional fixtures are public research examples. Private Praxis code, user documents,
model weights, credentials, uploads, exports and local scan/test artifacts are not included
in the Tacitus additions. Imported capsules remain untrusted data; reviews do not
automatically become locally trusted human actions.

New Tacitus commits pass the secret scan. The retained public upstream history has
unresolved scanner findings associated with examples/development configurations/tests;
the inherited dependency audit reports RSA advisory RUSTSEC-2023-0071 and maintenance
warnings. These are disclosed in the verification report, not suppressed with a blanket
allowlist. No security-readiness or enterprise/classified-data claim is made.

All 24 inherited workflows are quarantined under `.github/workflows-disabled`.
The new library CI definition has read-only permissions and no model downloads.
Repository Actions remain disabled; **CI has not run**. No deployment or tunnel is configured.

## Attribution and license

Built on EdgeQuake, pinned to **v0.34.0** at
`c994bc89c95fd44e191e4f015a2af0e84a27244f`, with upstream public history and
[Apache-2.0 licensing](LICENSE) retained. No upstream endorsement is implied.
See the [change notice](NOTICE-TACITUS.md) and [retained upstream README](docs/tacitus/upstream-readme.md).
Upstream startup instructions are not Tacitus acceptance evidence.

Tacitus code additions are Apache-2.0; the newly authored fictional research fixtures
are CC0-1.0. The provisional capsule format has no verified canonical Praxis interoperability.
