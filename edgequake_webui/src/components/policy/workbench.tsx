"use client";
import { useEffect, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import {
  getDocuments,
  uploadFile,
  uploadPdfDocument,
} from "@/lib/api/edgequake";
import {
  policy,
  type Analysis,
  type Span,
  type PolicyTask,
} from "@/lib/api/policy";
import { SourceInspector } from "./source-inspector";
import { CapsulePanel } from "./capsule-panel";
import { useTenantStore } from "@/stores/use-tenant-store";

const field = "block w-full rounded border bg-background p-2";
const button =
  "rounded border px-3 py-2 focus-visible:ring-2 disabled:opacity-50";
const message = (e: unknown) =>
  e instanceof Error ? e.message : "Request failed";
function cutoff(value: string) {
  return value ? Math.floor(new Date(value).getTime() / 1000) : null;
}

export function PolicyWorkbench() {
  const { selectedTenantId, selectedWorkspaceId, _hasHydrated } =
    useTenantStore();
  const cache = useQueryClient();
  const scope = [selectedTenantId, selectedWorkspaceId];
  const enabled = _hasHydrated && !!selectedTenantId && !!selectedWorkspaceId;
  const [question, setQuestion] = useState("");
  const [task, setTask] = useState<PolicyTask>("auto");
  const [deepen, setDeepen] = useState(false);
  const [known, setKnown] = useState("");
  const [valid, setValid] = useState("");
  const [documents, setDocuments] = useState<string[]>([]);
  const [methods, setMethods] = useState<string[]>([]);
  const [confirmed, setConfirmed] = useState(false);
  const [job, setJob] = useState<string | null>(null);
  const [span, setSpan] = useState<Span | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [description, setDescription] = useState("");
  const [methodScope, setMethodScope] = useState("");
  const [failure, setFailure] = useState("");
  const [rationale, setRationale] = useState("");
  useEffect(() => {
    setDocuments([]);
    setMethods([]);
    setConfirmed(false);
    setJob(null);
    setSpan(null);
    setError("");
    setQuestion("");
    setDescription("");
    setMethodScope("");
    setFailure("");
    setRationale("");
    setKnown("");
    setValid("");
  }, [selectedTenantId, selectedWorkspaceId]);
  const docs = useQuery({
    queryKey: ["policy-documents", ...scope],
    queryFn: () => getDocuments({ page_size: 100 }),
    enabled,
    refetchInterval: 5000,
  });
  const status = useQuery({
    queryKey: ["policy-status", ...scope],
    queryFn: policy.status,
    enabled,
  });
  const memory = useQuery({
    queryKey: ["policy-methods", ...scope],
    queryFn: policy.methods,
    enabled,
  });
  const history = useQuery({
    queryKey: ["policy-history", ...scope],
    queryFn: policy.analyses,
    enabled,
  });
  const impact = useQuery({
    queryKey: ["policy-impact", ...scope],
    queryFn: policy.impact,
    enabled,
  });
  const analysis = useQuery({
    queryKey: ["policy-analysis", ...scope, job],
    queryFn: () => policy.analysis(job!),
    enabled: enabled && !!job,
    refetchInterval: (q) =>
      q.state.data &&
      ["completed", "failed", "cancelled", "interrupted"].includes(
        q.state.data.status,
      )
        ? false
        : 1500,
  });
  const data = analysis.data ?? null;
  useEffect(() => {
    if (
      data &&
      ["completed", "failed", "cancelled", "interrupted"].includes(data.status)
    ) {
      void cache.invalidateQueries({ queryKey: ["policy-history", ...scope] });
      void cache.invalidateQueries({ queryKey: ["policy-impact", ...scope] });
    }
  }, [data?.status, cache, selectedTenantId, selectedWorkspaceId]);
  function refresh() {
    void cache.invalidateQueries({ queryKey: ["policy-documents", ...scope] });
    void cache.invalidateQueries({ queryKey: ["policy-methods", ...scope] });
    void cache.invalidateQueries({ queryKey: ["policy-impact", ...scope] });
  }
  async function ask() {
    setBusy(true);
    setError("");
    setSpan(null);
    try {
      const result = await policy.ask({
        question,
        task,
        deepen,
        document_ids: documents,
        known_as_of: cutoff(known),
        valid_at: cutoff(valid),
        method_ids: methods,
        facts: confirmed ? { scope_confirmed: true } : {},
      });
      setJob(result.id);
    } catch (e) {
      setError(message(e));
    } finally {
      setBusy(false);
    }
  }
  async function upload(file: File) {
    setBusy(true);
    setError("");
    try {
      if (file.name.toLowerCase().endsWith(".pdf"))
        await uploadPdfDocument(file, {
          enable_vision: false,
          pdf_parser_backend: "edgeparse",
          analyze_inline_images: false,
        });
      else await uploadFile(file);
      refresh();
    } catch (e) {
      setError(message(e));
    } finally {
      setBusy(false);
    }
  }
  async function propose() {
    if (!data?.context) return;
    setBusy(true);
    setError("");
    try {
      await policy.propose({
        description,
        kind: "procedure",
        scope: methodScope,
        evidence: data.context.passages.map((p) => p.span),
        prerequisites: ["scope_confirmed"],
        failure_conditions: failure
          .split("\n")
          .map((s) => s.trim())
          .filter(Boolean),
      });
      setDescription("");
      refresh();
    } catch (e) {
      setError(message(e));
    } finally {
      setBusy(false);
    }
  }
  async function review(id: string, fingerprint: string, decision: string) {
    setBusy(true);
    setError("");
    try {
      await policy.review({
        target_id: id,
        target_kind: "method",
        expected_fingerprint: fingerprint,
        decision,
        rationale,
      });
      refresh();
    } catch (e) {
      setError(message(e));
    } finally {
      setBusy(false);
    }
  }
  const running =
    !!data &&
    !["completed", "failed", "cancelled", "interrupted"].includes(data.status);
  const errors =
    error ||
    analysis.error?.message ||
    status.error?.message ||
    docs.error?.message ||
    memory.error?.message ||
    history.error?.message ||
    impact.error?.message;
  return (
    <main className="h-full overflow-auto p-page space-y-5">
      <header>
        <h1 className="text-2xl font-semibold">Ask Tacitus</h1>
        <p className="text-sm text-muted-foreground">
          Policy analysis built on EdgeQuake. Upload evidence, inspect passages
          and review proposed knowledge.
        </p>
        {status.data && (
          <p className="mt-2 text-sm">
            {status.data.provider} · {status.data.model} ·{" "}
            {status.data.local_only_enforced
              ? "Server enforces local models"
              : "Local-only policy disabled"}{" "}
            · runtime verification pending
          </p>
        )}
      </header>
      {!enabled && <p role="status">Select a tenant and workspace to begin.</p>}
      {errors && (
        <p role="alert" className="rounded border p-3">
          {errors}
        </p>
      )}
      <div className="grid min-w-0 gap-5 xl:grid-cols-[minmax(0,2fr)_minmax(280px,1fr)]">
        <div className="min-w-0 space-y-5">
          <form
            className="space-y-3"
            onSubmit={(e) => {
              e.preventDefault();
              void ask();
            }}
          >
            <label className="block font-medium">
              Question
              <textarea
                required
                maxLength={4000}
                rows={3}
                className={`${field} mt-1`}
                value={question}
                onChange={(e) => setQuestion(e.target.value)}
                placeholder="What could change the decision, its timing or its safeguards?"
              />
            </label>
            <div className="flex flex-wrap gap-3">
              <label>
                Analysis
                <select
                  value={task}
                  className={field}
                  onChange={(e) => setTask(e.target.value as PolicyTask)}
                >
                  <option value="auto">Suggest from question</option>
                  <option value="answer">Direct answer</option>
                  <option value="brief">Decision brief</option>
                  <option value="hypotheses">Competing hypotheses</option>
                  <option value="commitments">Positions and commitments</option>
                </select>
              </label>
              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={deepen}
                  onChange={(e) => setDeepen(e.target.checked)}
                />
                Deeper context and longer answer
              </label>
            </div>
            <details>
              <summary className="cursor-pointer">
                Evidence selection and time
              </summary>
              <div className="mt-2 space-y-2">
                <p className="text-sm">
                  No explicit selection uses up to 32 ready documents. Select a
                  dossier when more are available. Context omissions are
                  reported.
                </p>
                {docs.data?.items.map((d) => (
                  <label key={d.id} className="flex gap-2 text-sm">
                    <input
                      type="checkbox"
                      disabled={
                        !["completed", "indexed"].includes(d.status ?? "")
                      }
                      checked={documents.includes(d.id)}
                      onChange={(e) =>
                        setDocuments(
                          e.target.checked
                            ? [...documents, d.id]
                            : documents.filter((id) => id !== d.id),
                        )
                      }
                    />
                    {d.title ?? d.file_name ?? d.id} · {d.status}
                    {d.error_message && ` · ${d.error_message}`}
                  </label>
                ))}
                {(docs.data?.total ?? 0) > 100 && (
                  <p>
                    First 100 documents shown; use Documents to inspect the full
                    collection.
                  </p>
                )}
                <label className="block">
                  Known as of
                  <input
                    type="datetime-local"
                    className={field}
                    value={known}
                    onChange={(e) => setKnown(e.target.value)}
                  />
                </label>
                <label className="block">
                  World valid at
                  <input
                    type="datetime-local"
                    className={field}
                    value={valid}
                    onChange={(e) => setValid(e.target.value)}
                  />
                </label>
              </div>
            </details>
            <button
              className={`${button} bg-primary text-primary-foreground`}
              disabled={!enabled || busy || running || !question.trim()}
            >
              Generate cited analysis
            </button>
            {running && (
              <button
                type="button"
                className={button}
                onClick={() => {
                  void policy
                    .cancel(data!.id)
                    .then(() => analysis.refetch())
                    .catch((e) => setError(message(e)));
                }}
              >
                Cancel analysis
              </button>
            )}
          </form>
          <label className="block text-sm">
            Upload authorized evidence
            <input
              type="file"
              disabled={!enabled || busy}
              className="mt-2 block max-w-full"
              accept=".txt,.md,.pdf,.csv,.json"
              onChange={(e) => {
                const file = e.target.files?.[0];
                if (file) void upload(file);
                e.target.value = "";
              }}
            />
          </label>
          {data && <Answer data={data} inspect={setSpan} />}
          <details>
            <summary className="cursor-pointer font-medium">
              Reviewed cognitive methods
            </summary>
            <div className="mt-3 space-y-3">
              <p className="text-sm">
                A method is a scoped analytical procedure, not new evidence.
                Unknown prerequisites guard reuse; changed provenance makes it
                stale.
              </p>
              <label className="flex gap-2 text-sm">
                <input
                  type="checkbox"
                  checked={confirmed}
                  onChange={(e) => setConfirmed(e.target.checked)}
                />
                I confirm the selected methods apply to this question and scope
              </label>
              {(memory.data?.methods ?? []).map((entry) => (
                <article
                  className="rounded border p-3 space-y-2"
                  key={entry.method.id}
                >
                  <label className="flex gap-2">
                    <input
                      type="checkbox"
                      checked={methods.includes(entry.method.id)}
                      onChange={(e) =>
                        setMethods(
                          e.target.checked
                            ? [...methods, entry.method.id]
                            : methods.filter((id) => id !== entry.method.id),
                        )
                      }
                    />
                    {entry.method.description}
                  </label>
                  <p className="text-sm">
                    Scope: {entry.method.scope} · {entry.applicability.state} ·{" "}
                    {entry.review?.decision ?? "unreviewed"}
                  </p>
                  <p className="text-sm">
                    Failure conditions:{" "}
                    {entry.method.failure_conditions.join("; ")}
                  </p>
                  {entry.method.evidence.map((s, i) => (
                    <button
                      className={button}
                      key={i}
                      onClick={() => setSpan(s)}
                    >
                      Inspect provenance {i + 1}
                    </button>
                  ))}
                  <div className="flex gap-2">
                    <button
                      className={button}
                      disabled={busy || !rationale.trim()}
                      onClick={() =>
                        void review(
                          entry.method.id,
                          entry.fingerprint,
                          "accept",
                        )
                      }
                    >
                      Accept method
                    </button>
                    <button
                      className={button}
                      disabled={busy || !rationale.trim()}
                      onClick={() =>
                        void review(
                          entry.method.id,
                          entry.fingerprint,
                          "reject",
                        )
                      }
                    >
                      Reject method
                    </button>
                  </div>
                </article>
              ))}
              <label className="block text-sm">
                Human correction / review rationale
                <textarea
                  className={field}
                  value={rationale}
                  onChange={(e) => setRationale(e.target.value)}
                />
              </label>
              <fieldset disabled={!data?.result || busy} className="space-y-2">
                <legend>Propose a method from this analysis</legend>
                <label className="block text-sm">
                  Procedure
                  <textarea
                    className={field}
                    value={description}
                    onChange={(e) => setDescription(e.target.value)}
                  />
                </label>
                <label className="block text-sm">
                  Scope
                  <input
                    className={field}
                    value={methodScope}
                    onChange={(e) => setMethodScope(e.target.value)}
                  />
                </label>
                <label className="block text-sm">
                  Failure conditions, one per line
                  <textarea
                    className={field}
                    value={failure}
                    onChange={(e) => setFailure(e.target.value)}
                  />
                </label>
                <button
                  className={button}
                  disabled={
                    !description.trim() ||
                    !methodScope.trim() ||
                    !failure.trim()
                  }
                  onClick={() => void propose()}
                >
                  Save method proposal
                </button>
                <button
                  className={button}
                  disabled={!methodScope.trim() || !data?.result || busy}
                  onClick={async () => {
                    if (!data) return;
                    setBusy(true);
                    setError("");
                    try {
                      await policy.suggest({
                        analysis_id: data.id,
                        scope: methodScope,
                      });
                      refresh();
                    } catch (e) {
                      setError(message(e));
                    } finally {
                      setBusy(false);
                    }
                  }}
                >
                  Ask local model to propose a scoped method
                </button>
                {busy && (
                  <p role="status" className="text-sm">
                    Working with the server; model proposals remain unreviewed.
                  </p>
                )}
              </fieldset>
            </div>
          </details>
          <details>
            <summary className="cursor-pointer font-medium">
              Cognitive capsules
            </summary>
            {enabled && (
              <CapsulePanel
                key={scope.join(":")}
                analysis={data}
                methods={memory.data?.methods ?? []}
                onImported={refresh}
              />
            )}
          </details>
        </div>
        <div className="min-w-0 space-y-4">
          {span && (
            <SourceInspector span={span} onClose={() => setSpan(null)} />
          )}
          <section aria-label="Change impact">
            <h2 className="font-semibold">Review after change</h2>
            <button
              className={`${button} my-2`}
              disabled={!enabled}
              onClick={() => void impact.refetch()}
            >
              Check dependencies
            </button>
            {impact.data?.affected.map((a, i) => (
              <p className="my-2 break-all text-sm" key={`${a.id}:${i}`}>
                {a.kind} {a.id}: {a.reason}
              </p>
            ))}
            {impact.data && !impact.data.affected.length && (
              <p className="text-sm">
                No affected recorded dependencies found. This does not establish
                complete coverage.
              </p>
            )}
          </section>
          <section>
            <h2 className="font-semibold">Previous analyses</h2>
            {history.data?.analyses.map((a) => (
              <button
                key={a.id}
                className="my-2 block w-full rounded border p-2 text-left text-sm"
                onClick={() => {
                  setSpan(null);
                  setJob(a.id);
                }}
              >
                {a.question} · {a.status}
              </button>
            ))}
          </section>
        </div>
      </div>
    </main>
  );
}

function Answer({
  data,
  inspect,
}: {
  data: Analysis;
  inspect: (span: Span) => void;
}) {
  const result = data.result;
  function cite(id: string) {
    const p = data.context?.passages.find((p) => p.id === id);
    return p ? (
      <button
        className="rounded border px-2 py-1 text-sm focus-visible:ring-2"
        key={id}
        onClick={() => inspect(p.span)}
      >
        {id}: inspect passage
      </button>
    ) : null;
  }
  return (
    <section aria-label="Policy analysis" className="space-y-4 border-t pt-4">
      <p role="status">
        {data.status}
        {data.freshness && ` · ${data.freshness}`}
      </p>
      {data.error && <p role="alert">{data.error}</p>}
      {result && (
        <>
          <p className="text-sm">
            {result.task} · {result.model.model} · {result.model.prompt_tokens}{" "}
            input / {result.model.output_tokens} output tokens ·{" "}
            {result.review_state}
          </p>
          <p className="text-sm text-muted-foreground">
            {result.citation_validation}
          </p>
          {result.answer.sections.map((s, i) => (
            <article key={i}>
              <h2 className="font-semibold">{s.heading}</h2>
              <p className="whitespace-pre-wrap">{s.text}</p>
              <p className="my-2 text-sm text-muted-foreground">
                {s.qualification}
              </p>
              <div className="flex flex-wrap gap-2">
                {s.citations.map(cite)}
              </div>
            </article>
          ))}
          {!!result.answer.matrix.length && (
            <div className="overflow-auto">
              <table className="w-full text-left text-sm">
                <caption className="text-left font-semibold">
                  Competing hypotheses
                </caption>
                <thead>
                  <tr>
                    {[
                      "Hypothesis",
                      "Evidence",
                      "Consistency",
                      "Diagnosticity",
                    ].map((s) => (
                      <th className="p-2" key={s}>
                        {s}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {result.answer.matrix.map((c, i) => (
                    <tr className="border-t" key={i}>
                      <td className="p-2">{c.hypothesis}</td>
                      <td>{cite(c.citation)}</td>
                      <td>{c.consistency}</td>
                      <td>{c.diagnosticity}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
          {result.answer.positions.map((p, i) => (
            <article className="rounded border p-3" key={i}>
              <h3 className="font-medium">
                {p.actor}: {p.action}
              </h3>
              <blockquote className="my-2 whitespace-pre-wrap">
                {p.original_wording}
              </blockquote>
              <p className="text-sm">
                Authority: {p.authority ?? "unknown"} · Deadline:{" "}
                {p.deadline ?? "unknown"}
              </p>
              <p className="text-sm">
                Conditions: {p.conditions.join("; ") || "none recorded"} ·
                Implementation: {p.implementation}
              </p>
              {cite(p.citation)}
            </article>
          ))}
          <h2 className="font-semibold">Uncertainties</h2>
          {result.answer.uncertainties.map((u, i) => (
            <p className="text-sm" key={i}>
              {u}
            </p>
          ))}
        </>
      )}
      {data.context && (
        <details>
          <summary className="cursor-pointer">
            Context selection and omissions
          </summary>
          <p className="text-sm">{data.context.budget_unit}</p>
          {data.context.passages.map((p) => (
            <div className="my-2" key={p.id}>
              {cite(p.id)}
              <p className="text-xs">
                {p.selection_reason} · origin {p.source_family}
              </p>
            </div>
          ))}
          {data.context.omissions.map((o, i) => (
            <p className="text-sm" key={i}>
              {o}
            </p>
          ))}
        </details>
      )}
    </section>
  );
}
