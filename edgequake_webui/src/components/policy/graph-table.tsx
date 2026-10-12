"use client";
import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { policy, type Span } from "@/lib/api/policy";
import { useTenantStore } from "@/stores/use-tenant-store";
import { SourceInspector } from "./source-inspector";
import { DomainProfile } from "./domain-profile";
export function PolicyGraphTable() {
  const { selectedTenantId, selectedWorkspaceId } = useTenantStore();
  return <PolicyGraphView key={`${selectedTenantId}:${selectedWorkspaceId}`} />;
}
function PolicyGraphView() {
  const { selectedTenantId, selectedWorkspaceId } = useTenantStore();
  const [span, setSpan] = useState<Span | null>(null);
  const [reason, setReason] = useState("");
  const [error, setError] = useState("");
  const [domain, setDomain] = useState("");
  const [tool, setTool] = useState("");
  const [actor, setActor] = useState("");
  const [reviewState, setReviewState] = useState("");
  const [known, setKnown] = useState("");
  const [valid, setValid] = useState("");
  const [pendingReview, setPendingReview] = useState(false);
  const registry = useQuery({
    queryKey: ["policy-ontologies", selectedTenantId, selectedWorkspaceId],
    queryFn: policy.ontologies,
    enabled: !!selectedTenantId && !!selectedWorkspaceId,
  });
  const filter = {
    tool,
    actor,
    review_state: reviewState,
    known_as_of: known
      ? Math.floor(new Date(known).getTime() / 1000)
      : undefined,
    valid_at: valid ? Math.floor(new Date(valid).getTime() / 1000) : undefined,
  };
  useEffect(() => {
    setSpan(null);
    setReason("");
    setError("");
  }, [selectedTenantId, selectedWorkspaceId]);
  const query = useQuery({
    queryKey: ["policy-graph", selectedTenantId, selectedWorkspaceId, filter],
    queryFn: () => policy.graph(filter),
    enabled: !!selectedTenantId && !!selectedWorkspaceId,
  });
  async function review(id: string, fingerprint: string, decision: string) {
    if (pendingReview) return;
    setPendingReview(true);
    try {
      await policy.review({
        target_kind: "assertion",
        target_id: id,
        expected_fingerprint: fingerprint,
        decision,
        rationale: reason,
      });
      setError("");
      await query.refetch();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Review failed");
    } finally {
      setPendingReview(false);
    }
  }
  return (
    <section className="space-y-3 p-4">
      <h2 className="font-semibold">Attributed policy graph</h2>
      <p className="text-sm">
        Edges project source-bound assertions. Matching names are not merged;
        source changes require review.
      </p>
      {(error || query.error || registry.error) && (
        <p role="alert">
          {error || query.error?.message || registry.error?.message}
        </p>
      )}
      {registry.isLoading && <p role="status">Loading domain registry…</p>}
      {registry.data && (
        <DomainProfile
          data={registry.data}
          selected={domain}
          onSelect={(id) => {
            setDomain(id);
            setTool("");
            setSpan(null);
          }}
          onApplied={() => registry.refetch()}
        />
      )}
      <div className="flex flex-wrap gap-3 text-sm">
        <label>
          Graph tool
          <select
            aria-label="Graph tool"
            value={tool}
            onChange={(e) => {
              setTool(e.target.value);
              setSpan(null);
            }}
            className="block rounded border bg-background p-2"
          >
            <option value="">All attributed assertions</option>
            {registry.data?.registry.tools
              .filter(
                (t) =>
                  !domain ||
                  registry.data?.registry.packs
                    .find((p) => p.id === domain)
                    ?.tools.includes(t.id),
              )
              .map((t) => (
                <option key={t.id} value={t.id}>
                  {t.name}
                </option>
              ))}
          </select>
        </label>
        <label>
          Actor label (does not merge identities)
          <input
            value={actor}
            maxLength={100}
            onChange={(e) => setActor(e.target.value)}
            className="block rounded border bg-background p-2"
          />
        </label>
        <label>
          Review state
          <select
            value={reviewState}
            onChange={(e) => setReviewState(e.target.value)}
            className="block rounded border bg-background p-2"
          >
            <option value="">All states</option>
            {["proposed", "human_reviewed", "rejected", "review_required"].map(
              (s) => (
                <option key={s} value={s}>
                  {s}
                </option>
              ),
            )}
          </select>
        </label>
        <label>
          Known as of (your local time)
          <input
            type="datetime-local"
            value={known}
            onChange={(e) => setKnown(e.target.value)}
            className="block rounded border bg-background p-2"
          />
        </label>
        <label>
          Valid at (your local time)
          <input
            type="datetime-local"
            value={valid}
            onChange={(e) => setValid(e.target.value)}
            className="block rounded border bg-background p-2"
          />
        </label>
      </div>
      {query.isFetching && (
        <p role="status">Selecting authorized graph assertions…</p>
      )}
      {query.data?.tool && (
        <p className="text-sm">
          {query.data.tool.description} {query.data.tool.safeguard}
        </p>
      )}
      {query.data && (
        <p className="text-sm text-muted-foreground">
          {query.data.selection.time_semantics}
        </p>
      )}
      <label className="block text-sm">
        Human correction or review rationale
        <textarea
          value={reason}
          onChange={(e) => setReason(e.target.value)}
          className="block w-full rounded border bg-background p-2"
        />
      </label>
      <div className="overflow-auto">
        <table className="w-full text-left text-sm">
          <caption className="sr-only">
            Assertions, attribution, evidence and review state
          </caption>
          <thead>
            <tr>
              {[
                "Subject / speaker",
                "Relation / object",
                "Qualifications",
                "Evidence",
                "Review",
              ].map((h) => (
                <th className="p-2" key={h}>
                  {h}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {query.data?.edges.map((e) => (
              <tr key={e.id} className="border-t">
                <td className="p-2">
                  {e.assertion.subject}
                  <br />
                  {e.assertion.speaker}
                </td>
                <td className="p-2">
                  {e.label}: {e.assertion.object ?? e.assertion.proposition}
                </td>
                <td className="p-2">
                  {e.assertion.epistemic_type} · {e.assertion.modality}
                  <br />
                  {e.assertion.conditions.join("; ")}
                  <br />
                  {e.assertion.exceptions.join("; ")}
                </td>
                <td>
                  {e.assertion.evidence.map((s, i) => (
                    <button
                      key={i}
                      className="rounded border p-2"
                      onClick={() => setSpan(s)}
                    >
                      Inspect v{s.source.split("@").pop()}
                    </button>
                  ))}
                </td>
                <td className="p-2">
                  {e.review_state}
                  <div className="flex gap-2">
                    <button
                      disabled={
                        !reason.trim() ||
                        pendingReview ||
                        e.review_state === "review_required"
                      }
                      className="rounded border p-2 disabled:opacity-50"
                      onClick={() => void review(e.id, e.fingerprint, "accept")}
                    >
                      Accept
                    </button>
                    <button
                      disabled={
                        !reason.trim() ||
                        pendingReview ||
                        e.review_state === "review_required"
                      }
                      className="rounded border p-2 disabled:opacity-50"
                      onClick={() => void review(e.id, e.fingerprint, "reject")}
                    >
                      Reject
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {query.data && !query.data.edges.length && (
        <p>
          No authorized assertions match these filters. Adjust the selection, or
          generate an analysis and inspect its proposed claims.
        </p>
      )}
      {span && <SourceInspector span={span} onClose={() => setSpan(null)} />}
    </section>
  );
}
