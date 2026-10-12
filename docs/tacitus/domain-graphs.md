# Domain graph tools

Tacitus extends EdgeQuake's document ingestion, entity graph and vector retrieval
with a source-bound policy assertion graph. The **Graph** page includes domain
vocabularies and focused views of those assertions. This code is published as an
implementation candidate; build, database, model and browser execution for these
additions have not run. Use the [cloud quickstart](cloud-quickstart.md) on a suitable
host, with public synthetic documents first.

## Domains and questions

| Profile | Example question | Relevant views |
| --- | --- | --- |
| Public policy | Who authorized the programme, what funding stage is documented, and what blocks implementation? | Authority, funding, positions, hypotheses |
| Diplomacy and conflict | Which actors expressed competing positions, what are their conditions, and which explanations remain hypotheses? | Conflict, positions, authority, hypotheses |
| Human rights | Who alleges what, which passages report observations, and which safeguards or authority statements are documented? | Rights claims, authority, hypotheses, positions |
| HR / workforce policy | Which rules, roles, eligibility conditions and implementation dependencies does an employment policy describe? | Workforce, authority, positions, hypotheses |

An ontology defines the kinds of entities and relations to look for. A graph
connects source-described actors, positions, actions and other entities through
those relations. The profiles provide a bounded extraction vocabulary, definitions
and safeguards, rather than an automatic legal judgment or causal model.

## Use a profile

1. Log in, select a workspace and open **Graph**.
2. Select a domain. Inspect its entity/relation definitions and safeguards.
3. To change future extraction, confirm replacement and select **Apply extraction
   profile**. This requires actual owner/admin workspace membership. Choosing a
   domain alone only narrows the graph-tool menu.
4. Upload authorized documents through the existing workbench. Wait for ingestion,
   then generate a cited analysis. EdgeQuake extracts its entity graph; the analysis
   proposes separately attributed Tacitus assertions.
5. Select a graph tool, actor label, review state or time cutoff. These controls
   change server-side selection. Open each edge's exact evidence version and
   record a reason before accepting or rejecting it.

The seven native tool IDs are `authority`, `positions`, `funding`, `conflict`,
`hypotheses`, `rights` and `workforce`. Tools filter existing assertions by canonical
predicates and, for rights/hypotheses, epistemic type. They do not create new facts,
calculate conflict intensity or classify every statement into a domain. A rights
view may include a non-rights-related statement; the analyst determines relevance.
Neither a selected profile nor a filter means an edge is human reviewed.

Actor search matches speaker/subject labels; it does not merge identities or infer
control. Repeated reports do not become independent confirmation. Conditions and
exceptions stay on each assertion. Allegations remain allegations; authorization,
appropriation, disbursement and expenditure remain distinct. Workforce graphs must
not infer personal performance, eligibility or sensitive attributes.

## Versions, time and persistence

The authoritative Rust registry is `tacitus-domain-graphs/1`; each pack/tool has a
stable ID and version. It reuses `tacitus-policy/1` predicates and does not modify
the capsule v1/v2 schemas. Descriptive subject/object type lists guide interpretation;
they are not enforced type constraints. Unknown extraction types are retained.
Applying a profile explicitly clears any prior EdgeQuake preset/typed-edge rules,
replaces entity/relation vocabularies and disables strict type rejection. Other
workspace settings, including provider/model configuration, remain unchanged.

The existing PostgreSQL workspace metadata stores extraction settings. A scoped
event records the profile/version/digest, authenticated reviewer and before/after
settings. The UI compares current settings with that event and exposes drift.
Workspace update and event recording use the existing services in separate writes;
if event recording fails, inspect the current settings before retrying. A profile
does not retroactively reprocess documents or promote existing assertions.

**Known as of** excludes assertions created and reviews performed after the cutoff;
every exact source revision must have been available and published by then.
**Valid at** selects against each source's declared world-validity interval
`[valid_from, valid_until)`. Unspecified validity remains unknown and is not excluded.
These are source-validity controls, not a complete assertion-event timeline.
Inputs use the browser's local time and are sent as UTC Unix seconds. Authorization
is always checked against current access rights, including on historical sources.
Changed current provenance keeps historical edges marked `review_required`.

Imported capsules remain data. They cannot register graph tools, change extraction
profiles, select endpoints or acquire permissions.

## Cloud verification

The source-check command in the cloud quickstart includes native registry/filter
regressions and the API selection validation test. The cloud acceptance runner now
applies a profile to a new synthetic workspace before ingestion, checks its persisted
settings, rejects stale registry digests and unconfirmed replacement, compares all
seven graph views with their expected subsets, checks actor/knowledge/review filters,
and retains changed-evidence review. The real browser test exercises profile controls
and verifies actual filtered responses. All these checks are authored, **NOT RUN**.

Use `node scripts/tacitus-stack.mjs check-source`, then the real stack and acceptance
commands from [cloud quickstart](cloud-quickstart.md). For browser execution, use the
`tacitus-policy-real.spec.ts` commands in [the MVP guide](policy-mvp.md).
Runtime/cloud acceptance and the inherited audit gate remain required before release.
No measured gain in answer salience, efficiency or quality is claimed.
