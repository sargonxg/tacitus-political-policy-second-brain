"use client";
import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { policy, type Span } from "@/lib/api/policy";
import { useTenantStore } from "@/stores/use-tenant-store";
import { SourceInspector } from "./source-inspector";
export function PolicyGraphTable() {
  const { selectedTenantId, selectedWorkspaceId } = useTenantStore();
  const [span, setSpan] = useState<Span | null>(null);
  const [reason, setReason] = useState("");
  const [error, setError] = useState("");
  useEffect(() => {
    setSpan(null);
    setReason("");
    setError("");
  }, [selectedTenantId, selectedWorkspaceId]);
  const query = useQuery({
    queryKey: ["policy-graph", selectedTenantId, selectedWorkspaceId],
    queryFn: policy.graph,
    enabled: !!selectedTenantId && !!selectedWorkspaceId,
  });
  async function review(id: string, fingerprint: string, decision: string) {
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
    }
  }
  return (
    <section className="space-y-3 p-4">
      <h2 className="font-semibold">Attributed policy graph</h2>
      <p className="text-sm">
        Edges project source-bound assertions. Matching names are not merged;
        source changes require review.
      </p>
      {(error || query.error) && (
        <p role="alert">{error || query.error?.message}</p>
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
                      disabled={!reason.trim()}
                      className="rounded border p-2 disabled:opacity-50"
                      onClick={() => void review(e.id, e.fingerprint, "accept")}
                    >
                      Accept
                    </button>
                    <button
                      disabled={!reason.trim()}
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
          No authorized policy assertions yet. Generate an analysis and inspect
          its proposed claims.
        </p>
      )}
      {span && <SourceInspector span={span} onClose={() => setSpan(null)} />}
    </section>
  );
}
