"use client";
import { useEffect, useState } from "react";
import {
  policy,
  type Analysis,
  type MethodEntry,
  type Source,
} from "@/lib/api/policy";

function encode(bytes: Uint8Array): string {
  let raw = "";
  for (const byte of bytes) raw += String.fromCharCode(byte);
  return btoa(raw);
}
export function CapsulePanel({
  analysis,
  methods,
  onImported,
}: {
  analysis: Analysis | null;
  methods: MethodEntry[];
  onImported: () => void;
}) {
  const [sources, setSources] = useState<Source[]>([]);
  const [chosen, setChosen] = useState<string[]>([]);
  const [methodIds, setMethodIds] = useState<string[]>([]);
  const [title, setTitle] = useState("Policy context and method");
  const [include, setInclude] = useState(false);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState("");
  const [preview, setPreview] = useState<{
    preview_hash: string;
    files: Record<string, string>;
    warning: string;
  } | null>(null);
  useEffect(() => {
    let alive = true;
    policy
      .sources()
      .then((r) => {
        if (alive) setSources(r.sources);
      })
      .catch((e) => {
        if (alive) setMessage(e.message);
      });
    return () => {
      alive = false;
    };
  }, [analysis, methods]);
  const selection = {
    title,
    source_keys: chosen,
    method_ids: methodIds,
    assertion_ids:
      analysis?.result?.assertions
        .filter((a) => a.evidence.every((s) => chosen.includes(s.source)))
        .map((a) => a.id) ?? [],
    profile: methodIds.length ? "cognitive_method" : "context",
    evidence_mode: include ? "evidence_included" : "references_only",
  };
  const key = JSON.stringify(selection);
  useEffect(() => setPreview(null), [key]);
  async function inspect() {
    setBusy(true);
    setMessage("");
    try {
      setPreview(await policy.preview(selection));
    } catch (e) {
      setMessage(e instanceof Error ? e.message : "Preview failed");
    } finally {
      setBusy(false);
    }
  }
  async function download() {
    if (!preview) return;
    setBusy(true);
    try {
      const file = await policy.export({
        ...selection,
        expected_preview_hash: preview.preview_hash,
      });
      const bytes = Uint8Array.from(atob(file.capsule_base64), (c) =>
        c.charCodeAt(0),
      );
      const url = URL.createObjectURL(
        new Blob([bytes], { type: "application/zip" }),
      );
      const link = document.createElement("a");
      link.href = url;
      link.download = file.filename;
      link.click();
      setTimeout(() => URL.revokeObjectURL(url), 1000);
      setMessage(
        "Native capsule exported. Imported reviews will be untrusted.",
      );
    } catch (e) {
      setMessage(e instanceof Error ? e.message : "Export failed");
    } finally {
      setBusy(false);
    }
  }
  async function upload(file: File) {
    if (file.size > 8 * 1024 * 1024) {
      setMessage("Capsules are limited to 8 MiB.");
      return;
    }
    setBusy(true);
    try {
      const result = await policy.import(
        encode(new Uint8Array(await file.arrayBuffer())),
      );
      setMessage(
        `Imported ${result.documents.length} documents as ${result.review_state}. Wait for document processing and review methods before reuse.`,
      );
      onImported();
    } catch (e) {
      setMessage(e instanceof Error ? e.message : "Import failed");
    } finally {
      setBusy(false);
    }
  }
  return (
    <section
      className="space-y-3 rounded-md border p-4"
      aria-label="Native cognitive capsules"
    >
      <h2 className="font-semibold">Native cognitive capsules</h2>
      <p className="text-sm text-muted-foreground">
        Export selected evidence and methods. References-only packages can still
        contain sensitive assertions.
      </p>
      <label className="block text-sm">
        Title
        <input
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          className="mt-1 block w-full rounded border bg-background p-2"
        />
      </label>
      <fieldset>
        <legend className="text-sm font-medium">Evidence versions</legend>
        {sources.map((s) => (
          <label key={s.key} className="my-1 flex gap-2 text-sm">
            <input
              type="checkbox"
              checked={chosen.includes(s.key)}
              onChange={(e) =>
                setChosen(
                  e.target.checked
                    ? [...chosen, s.key]
                    : chosen.filter((k) => k !== s.key),
                )
              }
            />
            {s.title} · v{s.version}
            {!s.redistributable ? " · export rights not declared" : ""}
          </label>
        ))}
      </fieldset>
      <fieldset>
        <legend className="text-sm font-medium">Scoped methods</legend>
        {methods.map(({ method: m }) => (
          <label key={m.id} className="my-1 flex gap-2 text-sm">
            <input
              type="checkbox"
              checked={methodIds.includes(m.id)}
              onChange={(e) =>
                setMethodIds(
                  e.target.checked
                    ? [...methodIds, m.id]
                    : methodIds.filter((id) => id !== m.id),
                )
              }
            />
            {m.description}
          </label>
        ))}
      </fieldset>
      <label className="flex gap-2 text-sm">
        <input
          type="checkbox"
          checked={include}
          onChange={(e) => setInclude(e.target.checked)}
        />
        Include evidence text with declared export rights; required for
        fresh-workspace reuse
      </label>
      <div className="flex flex-wrap gap-2">
        <button
          disabled={busy || !chosen.length}
          onClick={inspect}
          className="rounded border px-3 py-2 disabled:opacity-50"
        >
          Inspect exact export
        </button>
        <button
          disabled={busy || !preview}
          onClick={download}
          className="rounded border px-3 py-2 disabled:opacity-50"
        >
          Export native capsule
        </button>
      </div>
      {preview && (
        <details open>
          <summary className="cursor-pointer">Exact fields and files</summary>
          <p className="my-2 text-sm">{preview.warning}</p>
          {Object.entries(preview.files).map(([name, content]) => (
            <details key={name}>
              <summary className="cursor-pointer break-all text-sm">
                {name}
              </summary>
              <pre className="max-h-64 overflow-auto whitespace-pre-wrap break-words rounded bg-muted p-2 text-xs">
                {content}
              </pre>
            </details>
          ))}
        </details>
      )}
      <label className="block text-sm">
        Import into this workspace
        <input
          type="file"
          accept=".capsule"
          disabled={busy}
          className="mt-2 block max-w-full"
          onChange={(e) => {
            const file = e.target.files?.[0];
            if (file) void upload(file);
            e.target.value = "";
          }}
        />
      </label>
      {message && (
        <p role="status" className="text-sm">
          {message}
        </p>
      )}
    </section>
  );
}
