# Tacitus — political and policy workbench

Built on [EdgeQuake](https://github.com/raphaelmansuy/edgequake): its Rust API,
PostgreSQL graph/vector stack, ingestion pipeline, authentication and Next.js workbench.
Tacitus adds question-specific policy context, attributed claims, human review and
portable cognitive methods. The marketing website has been removed; the functional
browser application remains.

**Current status: persistent MVP implementation candidate, awaiting cloud verification.**
The new API/UI integration has not been compiled, tested or started. Previous passing
lab/library counts do not validate this implementation. Do not treat it as an accepted
release or upload private evidence. The inherited audit gate remains open.

## What the implementation adds

- Upload authorized documents through EdgeQuake and build its existing entity graph and vector index.
- Ask a question; choose a direct answer, decision brief, competing hypotheses or position/commitment analysis. A simple task suggestion can be overridden.
- Compile bounded context from authorized source versions using question overlap, retrieved chunks and qualification cues. Inspect exact UTF-8 source spans and reported omissions.
- Generate with Ollama running on your own machine or cloud VM. Local-only provider guards reject paid providers, alternate models, saved connections and inherited credentials. Runtime enforcement is still awaiting acceptance tests.
- Store source versions, assertions, analysis jobs, authenticated review receipts, scoped methods and dependency events in the existing PostgreSQL database.
- Review assertions and methods. Rejected or imported reviews cannot grant acceptance; methods need explicit scope confirmation and satisfied prerequisites.
- Preview every exported field, transfer a native v2 capsule into a fresh workspace, re-ingest its evidence, review its methods and reuse them. References-only exports cannot provide missing original text.
- Flag recorded dependencies after source/provenance changes. Authorized stale graph assertions remain visible for review; assessments are not automatically rewritten.

For example, a policy brief must separate a council's authorization from an actual
appropriation or expenditure, preserve conditional willingness, include a status-quo
option and expose its cited passages. The validator checks citation identity and exact
quotes. An analyst still assesses whether the passages substantively support the judgment.

## Run on a suitable host

The storage-constrained development laptop is not an installation target. Prepare a
separate host with Git, Node 22+, Docker Engine/Compose and approximately 8 vCPU,
32 GiB RAM and 100 GiB free disk. CPU inference is supported in the configuration;
a GPU VM such as GCP `g2-standard-8` with an L4 is an optional later choice. These
are sizing recommendations, not measured minimums. No cloud resource is provisioned.

After selecting and reviewing an implementation revision:

```sh
git clone https://github.com/sargonxg/tacitus-political-policy-second-brain.git
cd tacitus-political-policy-second-brain
git switch main
node scripts/tacitus-stack.mjs doctor
node scripts/tacitus-stack.mjs init cloud
node scripts/tacitus-stack.mjs check
node scripts/tacitus-stack.mjs check-source
node scripts/tacitus-stack.mjs build
node scripts/tacitus-stack.mjs download
node scripts/tacitus-stack.mjs up
```

These are **candidate commands, not tested startup proof**. Builds/downloads happen
only when explicitly invoked. Private generated credentials live in ignored
`.tacitus/stack/config.json`. Database and models use persistent volumes; `down`
preserves them. API/database/Ollama use an internal network after model downloads.

On a VM, keep application ports closed and forward them:

```sh
ssh -N -L 3000:127.0.0.1:3000 -L 8080:127.0.0.1:8080 your-user@your-host
```

Intended browser URL: http://localhost:3000. Intended API health URL:
http://localhost:8080/health. Log in with the generated admin credentials, choose
a workspace, upload text/Markdown or text PDFs and wait for ingestion. Scanned/vision
PDFs are not accepted with the default text-only model.

Read [cloud quickstart](docs/tacitus/cloud-quickstart.md), [self-hosting](docs/tacitus/self-hosting.md) and
[the MVP workflow, CLI and cloud verification](docs/tacitus/policy-mvp.md).
The same authenticated API powers the browser and CLI; no separate knowledge store exists.

## Boundaries and evidence

The context selector is a transparent heuristic, not a proven gain in answer quality.
It preserves whole paragraphs and reports omissions; no efficiency or model-superiority
claim is made. Domain-wide ontology packs, complete temporal tracking, a distributed job
queue and controlled multi-baseline evaluation remain future work. Human ratings and
paid-provider comparisons are **NOT RUN**.

Read [implementation status](IMPLEMENTATION_STATUS.md),
[upstream pin and attribution](UPSTREAM.md),
[historical lab verification](tools/tacitus-policy-lab/VERIFICATION.md), and
[inherited audit triage](docs/tacitus/inherited-audit-triage.json).
Seven insecure-example findings and RSA `RUSTSEC-2023-0071` remain unresolved.
GitHub Actions remain disabled; cloud verification must be explicitly run on the chosen host.

The retained EdgeQuake code is Apache-2.0; see [LICENSE](LICENSE).
Native capsule v1 remains supported through an explicit v2 migration. Compatibility
with private canonical Praxis schemas is not claimed.
