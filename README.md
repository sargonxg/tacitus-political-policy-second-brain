# Tacitus Policy Workbench

**A local-first research environment for evidence-grounded policy and political analysis.**

[![Status](https://img.shields.io/badge/status-active%20prototype-yellow)](IMPLEMENTATION_STATUS.md)
[![Application packages](https://img.shields.io/badge/version-0.34.0-blue)](docs/tacitus/CORE_RELEASE_GATE_REPAIR_20261018.md)
[![Upstream](https://img.shields.io/badge/built%20on-EdgeQuake-0F766E)](UPSTREAM.md)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)

The application packages follow version `0.34.0`; native format versions are
independent. Current TACITUS source is unreleased schema 170. Published
`v0.34.0` remains schema 169.

Tacitus Policy Workbench adapts [EdgeQuake](https://github.com/raphaelmansuy/edgequake)—its Rust API, PostgreSQL graph/vector stack, ingestion pipeline, authentication, and Next.js workbench—for policy research. The TACITUS layer adds domain-specific extraction, attributed claims, review receipts, question-bounded context, analytical output contracts, and portable context capsules.

> **Verification status:** active implementation candidate; end-to-end runtime verification is still pending. The new integration has not yet been accepted as a working release. Do not upload confidential material or expose the stack publicly until the documented build, security, and acceptance gates pass.

For a smaller standalone local evidence inspector, see the [TACITUS Workbench run guide](https://github.com/sargonxg/tacitus-workbench/blob/main/README.md). Its recorded examples and opt-in pinned-core execution are separate from this full EdgeQuake application. Neither local path establishes a working hosted PRAXIS attachment bridge.

## Run the capsule-only developer workflow

A reproducible local/cloud-machine path is available for native-v2 creation, strict validation, inspection, layered context and cited documents, with corrected research samples and a separate synthetic authoring example. Start with [the capsule run guide](docs/tacitus/LOCAL_CAPSULE_RELEASE.md).

```bash
python3 scripts/check-capsule-release.py ../tacitus-capsule-output
```

Requires Python3.11+, Node22+ and official Rust1.95.0 with Clippy/rustfmt. No database, model credentials, Docker or Bun are needed for this bounded workflow. The full application status above remains separate.

## 30-second read

- **Problem:** policy work combines documents, claims, authority, funding, commitments, chronology, uncertainty, and reviewed judgments; a generic “second brain” rarely preserves all of them.
- **Approach:** use an existing ingestion and graph stack, then add typed policy objects, source-span context, human review, explicit output modes, and portable capsules.
- **Intended deployment:** self-hosted and local-first, with Ollama as the default model path and no paid-provider dependency required for the core workflow.
- **Current boundary:** architecture and implementation work are present, but clean-host build, startup, ingestion, analysis, review, export, and re-import have not yet been demonstrated together on the current revision.

## Intended analyst workflow

1. Create or select an authenticated workspace.
2. Upload authorized text, Markdown, or text-based PDF material through the inherited ingestion path.
3. Choose a versioned domain vocabulary for public policy, diplomacy and conflict, human rights, or workforce analysis.
4. Ask a question and select an output contract, such as a direct answer, decision brief, competing hypotheses, or position-and-commitment analysis.
5. Compile bounded context from authorized source versions, retrieved chunks, question overlap, and qualification cues.
6. Inspect exact source spans, omissions, attributed assertions, and uncertainty.
7. Review assertions and analytical methods; record who accepted, rejected, imported, or scoped each item.
8. Export a portable capsule, inspect every exported field, and test re-import into a clean workspace.

The workbench is designed to support an analyst, not silently replace one. A citation validator can confirm that a quoted passage exists and is attached to the correct source version; it cannot decide whether the passage substantively justifies a policy judgment.

## What the TACITUS layer adds

| Capability | Purpose |
| --- | --- |
| Domain graph vocabularies | Represent authority, funding, positions, conflict, hypotheses, rights claims, commitments, and workforce assertions with domain-specific fields |
| Question-bounded context | Assemble a reviewable context pack rather than sending an entire workspace indiscriminately to a model |
| Source-span inspection | Preserve exact UTF-8 spans and report material omissions |
| Output contracts | Separate direct answers, decision briefs, competing hypotheses, and commitment/position analysis |
| Review receipts | Record authenticated review decisions and scope confirmations |
| Method governance | Require prerequisites and explicit scope before an imported analytical method can be treated as accepted |
| Dependency and staleness events | Flag downstream objects when source or provenance changes without silently rewriting analyst judgments |
| Portable capsules | Preview, export, transfer, re-ingest, and review evidence and methods in another workspace |
| Local inference path | Use Ollama on a controlled host; provider guards are intended to block inherited paid-provider paths in local-only mode |

See [domain graph tools](docs/tacitus/domain-graphs.md) and [MVP workflow](docs/tacitus/policy-mvp.md) for the detailed design.

## Architecture

```text
browser workbench
      │
      ▼
authenticated EdgeQuake/TACITUS API
      │
      ├── document ingestion and source versions
      ├── graph + vector retrieval
      ├── policy assertion and review records
      ├── context compiler and output contracts
      ├── local model boundary
      └── capsule export / import
      │
      ▼
PostgreSQL + graph/vector extensions + persistent model volumes
```

The same authenticated API is intended to serve the browser and CLI. There is no separate hidden knowledge store for the TACITUS layer.

## Provisional clean-host procedure

The following commands describe the current intended stack workflow. They are **not yet proof of a successful clean installation**.

Requirements and sizing guidance are documented in the [cloud quickstart](docs/tacitus/cloud-quickstart.md) and [self-hosting guide](docs/tacitus/self-hosting.md). No cloud resource is provisioned by this repository.

```bash
git clone https://github.com/sargonxg/tacitus-political-policy-second-brain.git
cd tacitus-political-policy-second-brain

node scripts/tacitus-stack.mjs doctor
node scripts/tacitus-stack.mjs init cloud
node scripts/tacitus-stack.mjs check
node scripts/tacitus-stack.mjs check-source
node scripts/tacitus-stack.mjs build
node scripts/tacitus-stack.mjs download
node scripts/tacitus-stack.mjs up
```

Generated credentials are intended to remain in ignored local configuration. Persistent volumes retain database and model state when the stack is stopped. On a remote host, keep application ports private and use an authenticated tunnel rather than exposing development services directly.

## Release gate

The repository should not be described as a runnable MVP until a clean host can demonstrate and record all of the following on one commit:

- source and dependency checks pass;
- containers build from a clean checkout;
- services start with generated local credentials;
- authentication and workspace selection work;
- a public test document ingests successfully;
- graph and vector retrieval return source-linked results;
- at least one policy output contract produces an inspectable result;
- review receipts persist and enforce scope rules;
- capsule export, clean-workspace import, and evidence re-ingestion succeed;
- shutdown and restart preserve expected state;
- no paid provider or inherited credential is contacted in local-only mode;
- security findings are resolved or explicitly accepted with documented rationale;
- commands, logs, host specification, and commit SHA are published in a verification record.

Until then, [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) is the authoritative status document.

## Security and evidence boundary

The context selector is a transparent heuristic, not a demonstrated improvement in answer quality. Human ratings, paid-provider comparisons, and controlled multi-baseline evaluation have not been run. Full domain-rule enforcement, complete assertion-level temporal tracking, and a distributed job queue remain future work.

The inherited audit gate is still open. The repository documents unresolved insecure-example findings and `RUSTSEC-2023-0071`; GitHub Actions are disabled, and cloud verification must be run explicitly on the selected host. Review [inherited audit triage](docs/tacitus/inherited-audit-triage.json), [historical lab verification](tools/tacitus-policy-lab/VERIFICATION.md), and [upstream attribution](UPSTREAM.md) before deployment.

Do not process classified, privileged, protected, personnel, clinical, or otherwise sensitive records in the prototype. Do not use its outputs for unattended legal, employment, eligibility, security, or other high-impact decisions about individuals.

## Relationship to TACITUS

- **Policy Workbench:** integration lab and analyst-facing research environment.
- [DIALECTICA](https://github.com/sargonxg/A3_DIALECTICAbyTACITUS_v3): portable context-capsule compiler and review engine.
- [AGON](https://github.com/sargonxg/AGON): experimental evidence and conflict-perception engine.
- [KAIROS](https://github.com/sargonxg/KAIROS-temporal-vision-TACITUS): experimental temporal-analysis engine.
- [TCGC](https://github.com/sargonxg/TCGC_TACITUS-Conflict-Grammar-Corpus_BENCHMARK): structural evaluation surface.

## Attribution and license

The retained EdgeQuake code and TACITUS modifications are distributed under the Apache License 2.0; see [LICENSE](LICENSE) and [UPSTREAM.md](UPSTREAM.md). Compatibility with private or unpublished PRAXIS schemas is not claimed.
