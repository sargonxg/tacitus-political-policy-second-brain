"use client";
import { useEffect, useState } from "react";
import { policy, type OntologyRegistry } from "@/lib/api/policy";

export function DomainProfile({
  data,
  selected,
  onSelect,
  onApplied,
}: {
  data: OntologyRegistry;
  selected: string;
  onSelect: (id: string) => void;
  onApplied: () => Promise<unknown>;
}) {
  const [confirmed, setConfirmed] = useState(false);
  const [pending, setPending] = useState(false);
  const [message, setMessage] = useState("");
  const pack = data.registry.packs.find((p) => p.id === selected);
  useEffect(() => {
    setConfirmed(false);
    setMessage("");
  }, [selected]);
  async function apply() {
    if (!pack || !confirmed || pending) return;
    setPending(true);
    setMessage("");
    try {
      await policy.applyOntology({
        id: pack.id,
        version: pack.version,
        registry_digest: data.registry_digest,
        replace_extraction_configuration: true,
      });
      setConfirmed(false);
      await onApplied();
      setMessage(
        "Profile saved for future ingestion. Existing documents and assertions were not reprocessed.",
      );
    } catch (e) {
      setMessage(
        e instanceof Error ? e.message : "Profile could not be applied",
      );
    } finally {
      setPending(false);
    }
  }
  return (
    <div className="space-y-3 rounded border p-3">
      <label className="block text-sm">
        Domain vocabulary
        <select
          aria-label="Domain vocabulary"
          className="ml-2 rounded border bg-background p-2"
          value={selected}
          disabled={pending}
          onChange={(e) => onSelect(e.target.value)}
        >
          <option value="">
            All graph tools · keep current extraction settings
          </option>
          {data.registry.packs.map((p) => (
            <option key={p.id} value={p.id}>
              {p.name} v{p.version}
            </option>
          ))}
        </select>
      </label>
      <p className="text-sm">
        Recorded profile: {data.recorded_profile?.profile ?? "none"}
        {data.recorded_profile &&
          (data.configuration_matches_recorded_profile
            ? " · settings match"
            : " · extraction settings have changed")}
        . Registry {data.registry.version}.
      </p>
      {pack && (
        <>
          <p className="text-sm">{pack.description}</p>
          {pack.safeguards.map((s) => (
            <p className="text-sm text-muted-foreground" key={s}>
              {s}
            </p>
          ))}
          <details>
            <summary className="cursor-pointer text-sm">
              Inspect entity and relation definitions
            </summary>
            <dl className="space-y-2 p-2 text-sm">
              {pack.entities.map((e) => (
                <div key={e.id}>
                  <dt className="font-medium">{e.extraction_type}</dt>
                  <dd>{e.definition}</dd>
                </div>
              ))}
              {data.registry.relations
                .filter((r) => pack.predicates.includes(r.predicate))
                .map((r) => (
                  <div key={r.id}>
                    <dt className="font-medium">{r.predicate}</dt>
                    <dd>
                      {r.definition} {r.safeguard}
                    </dd>
                  </div>
                ))}
            </dl>
            <p className="text-sm">
              Entity/relation types guide extraction. The suggested
              subject/object types are descriptive guidance, not enforced legal
              or causal rules. Unknown types are retained.
            </p>
          </details>
          <label className="flex items-start gap-2 text-sm">
            <input
              type="checkbox"
              checked={confirmed}
              disabled={pending}
              onChange={(e) => setConfirmed(e.target.checked)}
            />
            Replace this workspace&apos;s entity/relation extraction settings
            for future uploads. Requires owner/admin membership.
          </label>
          <button
            className="rounded border p-2 text-sm disabled:opacity-50"
            disabled={!confirmed || pending}
            onClick={() => void apply()}
          >
            {pending ? "Saving profile…" : "Apply extraction profile"}
          </button>
        </>
      )}
      {message && (
        <p role="status" className="text-sm">
          {message}
        </p>
      )}
    </div>
  );
}
