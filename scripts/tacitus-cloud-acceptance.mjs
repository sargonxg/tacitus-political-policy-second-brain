// Run ONLY on the chosen host against real PostgreSQL/Ollama. Creates new test workspaces.
import assert from "node:assert/strict";
import { randomUUID } from "node:crypto";
import { readFile, writeFile, mkdir } from "node:fs/promises";
import { request, waitAnalysis } from "./tacitus-policy.mjs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
const receiptPath = ".tacitus/cloud-acceptance.json";
const ready = async (id) => {
  const deadline = Date.now() + 650000;
  while (Date.now() < deadline) {
    const d = await request(`/documents/${id}`);
    if (["completed", "indexed"].includes(d.status)) {
      assert.ok(d.chunk_count > 0, "real chunks required");
      assert.ok(d.entity_count > 0, "real entity extraction required");
      return d;
    }
    if (["failed", "partial_failure", "cancelled"].includes(d.status))
      throw new Error(`Ingestion ${d.status}: ${d.error_message ?? ""}`);
    await new Promise((r) => setTimeout(r, 2000));
  }
  throw new Error("Real ingestion timeout");
};
async function ask(question, task, document_ids, extra = {}) {
  const job = await request("/tacitus/analyses", {
    question,
    task,
    document_ids,
    deepen: true,
    ...extra,
  });
  const result = await waitAnalysis(job.id);
  assert.ok(result.result.model.prompt_tokens > 0);
  assert.ok(result.result.model.output_tokens > 0);
  assert.ok(result.result.model.model_digest);
  assert.ok(result.context.passages.length);
  for (const section of result.result.answer.sections) {
    assert.ok(section.citations.length);
    for (const id of section.citations)
      assert.ok(result.context.passages.some((p) => p.id === id));
  }
  return result;
}
export async function runAcceptance(phase = process.argv[2]) {
  if (process.env.TACITUS_CLOUD_EXECUTE !== "1")
    throw new Error(
      "Explicitly set TACITUS_CLOUD_EXECUTE=1 on the chosen host; this writes synthetic test data",
    );
  if (phase === "after-restart") {
    const r = JSON.parse(await readFile(receiptPath, "utf8"));
    process.env.TACITUS_WORKSPACE = r.workspace;
    const prior = await request(`/tacitus/analyses/${r.analysis}`);
    assert.equal(prior.status, "completed");
    const detail = await request(
      `/tacitus/sources/${r.source_id}?version=${r.source_version}&start=${r.start}&end=${r.end}`,
    );
    assert.ok(detail.passage.includes(r.marker));
    const repeated = await ask(
      `What is the ${r.marker} programme authorization condition?`,
      "answer",
      [r.document],
    );
    assert.ok(repeated.context.passages.some((p) => p.text.includes(r.marker)));
    console.log(
      JSON.stringify(
        {
          phase: "after-restart",
          persisted_analysis: r.analysis,
          new_analysis: repeated.id,
          real_model: repeated.result.model,
          offline_network_verification:
            "Record actual Docker internal network/environment separately",
        },
        null,
        2,
      ),
    );
    return;
  }
  const status = await request("/tacitus/status");
  assert.equal(status.local_only_enforced, true);
  const tenant = process.env.TACITUS_TENANT;
  const run = randomUUID();
  const marker = `POLICY_${run.replaceAll("-", "")}`;
  async function workspace(suffix) {
    return request(`/tenants/${tenant}/workspaces`, {
      name: `Policy acceptance ${suffix} ${run}`,
      slug: `policy-${suffix}-${run}`,
      llm_provider: "ollama",
      llm_model: status.model,
      embedding_provider: "ollama",
      embedding_model: status.embedding_model,
      embedding_dimension: 768,
      pdf_parser_backend: "edgeparse",
    });
  }
  const first = await workspace("origin");
  process.env.TACITUS_WORKSPACE = first.id;
  const registry = await request("/tacitus/ontologies");
  assert.equal(registry.registry.version, "tacitus-domain-graphs/1");
  assert.equal(registry.registry.packs.length, 4);
  const pack = registry.registry.packs.find((p) => p.id === "public-policy");
  assert.ok(pack);
  const applyProfile = {
    id: pack.id,
    version: pack.version,
    registry_digest: registry.registry_digest,
    replace_extraction_configuration: true,
  };
  await assert.rejects(
    request("/tacitus/ontologies/apply", {
      ...applyProfile,
      registry_digest: "stale",
    }),
    /409/,
  );
  await assert.rejects(
    request("/tacitus/ontologies/apply", {
      ...applyProfile,
      replace_extraction_configuration: false,
    }),
    /409/,
  );
  const applied = await request("/tacitus/ontologies/apply", applyProfile);
  assert.equal(applied.existing_assertions_changed, false);
  const recorded = await request("/tacitus/ontologies");
  assert.equal(recorded.configuration_matches_recorded_profile, true);
  assert.equal(recorded.workspace_configuration.entity_types_strict, false);
  assert.deepEqual(
    recorded.workspace_configuration.entity_types,
    pack.entities.map((e) => e.extraction_type),
  );
  await assert.rejects(request("/tacitus/graph?tool=imported-plugin"), /400/);
  await assert.rejects(request("/tacitus/graph?review_state=verified"), /400/);
  const text = `Fictional public acceptance fixture, CC0.\n\nThe Council authorized the ${marker} programme. Expenditure remains conditional on a separate appropriation. Minister Ada stated that the ministry would participate only after the appropriation is approved; discussion is not acceptance.\n\nThe Assembly has not approved the appropriation. Funding and implementation remain unknown. The status quo is to wait for authority and resources.`;
  const upload = await request("/documents", {
    title: `Public synthetic ${marker}`,
    content: text,
    async_processing: true,
  });
  await ready(upload.document_id);
  const answer = await ask(
    `What is the ${marker} programme authorization condition?`,
    "answer",
    [upload.document_id],
  );
  const passage = answer.context.passages.find((p) => p.text.includes(marker));
  assert.ok(
    answer.result.retrieval_chunks > 0,
    "Real EdgeQuake retrieval must return indexed chunks; lexical context alone is insufficient acceptance",
  );
  assert.ok(passage, "new random marker must come from real retrieval/context");
  const [sourceId, version] = passage.span.source.split("@");
  const exact = await request(
    `/tacitus/sources/${sourceId}?version=${version}&start=${passage.span.start}&end=${passage.span.end}`,
  );
  assert.equal(exact.passage, passage.text);
  const revisedSource = await request(
    `/tacitus/sources/${sourceId}`,
    {
      expected_version: Number(version),
      author: "Synthetic Council",
      language: "en",
      published_at: null,
      valid_from: 100,
      valid_until: 200,
      source_family: marker,
      redistributable: true,
      license: "CC0",
    },
    "PATCH",
  );
  const brief = await ask(
    `Prepare a decision brief on ${marker}: whether to begin implementation or maintain status quo.`,
    "brief",
    [upload.document_id],
  );
  assert.ok(
    brief.result.assertions.length,
    "At least one model-proposed attributed assertion must project into the policy graph",
  );
  const hypotheses = await ask(
    `Compare explanations for the ${marker} programme delay.`,
    "hypotheses",
    [upload.document_id],
  );
  assert.ok(hypotheses.result.answer.matrix.length >= 2);
  const allGraph = await request("/tacitus/graph");
  assert.ok(allGraph.edges.length);
  const revisedKey = `${sourceId}@${revisedSource.source.version}`;
  assert.ok(
    allGraph.edges.some((e) =>
      e.assertion.evidence.some((s) => s.source === revisedKey),
    ),
  );
  for (const [at, inside] of [
    [99, false],
    [100, true],
    [199, true],
    [200, false],
  ]) {
    const temporal = await request(`/tacitus/graph?valid_at=${at}`);
    const expected = allGraph.edges.filter(
      (e) =>
        inside || !e.assertion.evidence.some((s) => s.source === revisedKey),
    );
    assert.deepEqual(
      temporal.edges.map((e) => e.id).sort(),
      expected.map((e) => e.id).sort(),
      "World-validity intervals are half-open; unknown validity remains eligible",
    );
  }
  const beforeKnowledge = await request("/tacitus/graph?known_as_of=0");
  assert.equal(beforeKnowledge.edges.length, 0);
  for (const tool of registry.registry.tools) {
    const filtered = await request(
      `/tacitus/graph?tool=${encodeURIComponent(tool.id)}`,
    );
    assert.equal(filtered.tool.id, tool.id);
    const expected = allGraph.edges.filter(
      (e) =>
        tool.predicates.includes(e.assertion.predicate) &&
        (!tool.epistemic_types.length ||
          tool.epistemic_types.includes(e.assertion.epistemic_type)),
    );
    assert.deepEqual(
      filtered.edges.map((e) => e.id).sort(),
      expected.map((e) => e.id).sort(),
    );
  }
  const actor = allGraph.edges[0].assertion.speaker;
  const actorGraph = await request(
    `/tacitus/graph?actor=${encodeURIComponent(actor)}`,
  );
  assert.ok(actorGraph.edges.length);
  assert.ok(
    actorGraph.edges.every((e) =>
      [e.assertion.subject, e.assertion.speaker].some((label) =>
        label.toLowerCase().includes(actor.toLowerCase()),
      ),
    ),
  );
  const reviewableEdge = allGraph.edges.find(
    (e) => e.review_state === "proposed",
  );
  assert.ok(
    reviewableEdge,
    "Current exact evidence is required for assertion review",
  );
  const edgeReview = {
    target_kind: "assertion",
    target_id: reviewableEdge.id,
    expected_fingerprint: reviewableEdge.fingerprint,
    rationale:
      "Authenticated synthetic review-path test; human quality ratings remain pending.",
  };
  await request("/tacitus/reviews", { ...edgeReview, decision: "reject" });
  const rejectedGraph = await request("/tacitus/graph?review_state=rejected");
  assert.ok(rejectedGraph.edges.some((e) => e.id === reviewableEdge.id));
  assert.ok(rejectedGraph.edges.every((e) => e.review_state === "rejected"));
  await request("/tacitus/reviews", { ...edgeReview, decision: "accept" });
  const reviewedGraph = await request(
    "/tacitus/graph?review_state=human_reviewed",
  );
  assert.ok(reviewedGraph.edges.some((e) => e.id === reviewableEdge.id));
  const commitments = await ask(
    `What conditional participation did Minister Ada state for ${marker}?`,
    "commitments",
    [upload.document_id],
  );
  assert.ok(
    commitments.result.answer.positions.some((p) => p.conditions.length),
  );
  const evidence = brief.context.passages.map((p) => p.span);
  const proposal = await request("/tacitus/methods", {
    description:
      "Distinguish authorization, appropriation and expenditure before judging programme feasibility.",
    kind: "distinction",
    scope: "Public-policy implementation and budget authority",
    evidence,
    prerequisites: ["scope_confirmed"],
    failure_conditions: [
      "Budget stages are unspecified",
      "Source provenance changes",
    ],
  });
  const review = {
    target_kind: "method",
    target_id: proposal.method.id,
    expected_fingerprint: proposal.fingerprint,
    decision: "accept",
    rationale:
      "Human acceptance of this synthetic method for the budget-stage test; not a factual attestation.",
  };
  await request("/tacitus/reviews", review);
  await assert.rejects(
    ask(`Assess ${marker} safeguards`, "answer", [upload.document_id], {
      method_ids: [proposal.method.id],
    }),
    /guarded/,
  );
  const reuse = await ask(
    `Assess ${marker} safeguards`,
    "answer",
    [upload.document_id],
    { method_ids: [proposal.method.id], facts: { scope_confirmed: true } },
  );
  assert.ok(reuse.context.methods.some((m) => m.id === proposal.method.id));
  const selection = {
    title: "Synthetic budget-stage capsule",
    source_keys: [...new Set(evidence.map((e) => e.source))],
    method_ids: [proposal.method.id],
    assertion_ids: brief.result.assertions.map((a) => a.id),
    profile: "cognitive_method",
    evidence_mode: "evidence_included",
  };
  const preview = await request("/tacitus/capsules/preview", selection);
  const exported = await request("/tacitus/capsules/export", {
    ...selection,
    expected_preview_hash: preview.preview_hash,
  });
  assert.ok(exported.capsule_base64);
  const second = await workspace("import");
  process.env.TACITUS_WORKSPACE = second.id;
  await assert.rejects(request(`/tacitus/analyses/${brief.id}`), /404/);
  await assert.rejects(
    request(`/tacitus/sources/${sourceId}?version=${version}`),
    /404/,
  );
  const imported = await request("/tacitus/capsules/import", {
    capsule_base64: exported.capsule_base64,
  });
  assert.equal(imported.review_state, "unreviewed_proposals");
  for (const d of imported.documents)
    await ready(d.duplicate_of ?? d.document_id);
  const memory = await request("/tacitus/methods");
  const entry = memory.methods.find(
    (m) => m.method.id === imported.method_ids[0],
  );
  assert.ok(entry);
  assert.notEqual(entry.applicability.state, "active");
  await request("/tacitus/reviews", {
    ...review,
    target_id: entry.method.id,
    expected_fingerprint: entry.fingerprint,
    rationale:
      "Reviewed fresh-workspace budget-stage procedure and exact imported evidence.",
  });
  const transferred = await ask(
    `What safeguards matter for ${marker} implementation?`,
    "answer",
    imported.documents.map((d) => d.duplicate_of ?? d.document_id),
    { method_ids: [entry.method.id], facts: { scope_confirmed: true } },
  );
  assert.ok(transferred.context.methods.length);
  process.env.TACITUS_WORKSPACE = first.id;
  const current = (await request("/tacitus/sources")).sources.find(
    (s) => s.id === sourceId,
  );
  assert.ok(current);
  const changed = await request("/documents", {
    title: `Changed ${marker}`,
    content: `Public synthetic test, CC0.\n\nThe Assembly approved the ${marker} appropriation after the Council authorization. Minister Ada confirmed participation subject to procurement safeguards. Expenditure and implementation remain undocumented.`,
    async_processing: true,
  });
  await ready(changed.document_id);
  await request(`/tacitus/sources/${sourceId}/replacement`, {
    expected_version: current.version,
    document_id: changed.document_id,
  });
  const impact = await request("/tacitus/impact");
  assert.ok(
    impact.affected.some(
      (a) => a.kind === "method" && a.id === proposal.method.id,
    ),
  );
  assert.ok(
    impact.affected.some((a) => a.kind === "analysis" && a.id === brief.id),
  );
  const graph = await request("/tacitus/graph");
  assert.ok(graph.edges.some((e) => e.review_state === "review_required"));
  const staleGraph = await request(
    "/tacitus/graph?review_state=review_required",
  );
  assert.ok(staleGraph.edges.length);
  assert.ok(
    staleGraph.edges.every((e) => e.review_state === "review_required"),
  );
  assert.ok(staleGraph.edges.some((e) => e.id === reviewableEdge.id));
  const noStaleBadge = await request(
    "/tacitus/graph?review_state=human_reviewed",
  );
  assert.ok(noStaleBadge.edges.every((e) => e.id !== reviewableEdge.id));
  const receipt = {
    tenant,
    workspace: first.id,
    fresh_workspace: second.id,
    analysis: answer.id,
    document: changed.document_id,
    source_id: sourceId,
    source_version: Number(version),
    start: passage.span.start,
    end: passage.span.end,
    marker,
    model: answer.result.model,
    domain_registry: registry.registry.version,
    extraction_profile: pack.id,
    graph_tools: registry.registry.tools.map((t) => t.id),
    brief: brief.id,
    hypotheses: hypotheses.id,
    commitments: commitments.id,
    transferred: transferred.id,
    changed_evidence:
      "newly ingested text replacing the earlier source; dependency review required",
    human_ratings: "PENDING",
    frontier_comparison: "NOT RUN",
  };
  await mkdir(".tacitus", { recursive: true });
  await writeFile(receiptPath, JSON.stringify(receipt, null, 2), {
    flag: "wx",
    mode: 0o600,
  });
  console.log(JSON.stringify(receipt, null, 2));
}
if (
  process.argv[1] &&
  resolve(process.argv[1]) === fileURLToPath(import.meta.url)
)
  runAcceptance().catch((e) => {
    console.error(e.message);
    process.exitCode = 1;
  });
