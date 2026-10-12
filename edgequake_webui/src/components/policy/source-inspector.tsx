"use client";
import { useEffect, useState } from "react";
import { policy, type Span, type SourceDetail } from "@/lib/api/policy";

export function SourceInspector({
  span,
  onClose,
}: {
  span: Span;
  onClose: () => void;
}) {
  const [data, setData] = useState<{
    source: SourceDetail;
    passage: string;
  } | null>(null);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [author, setAuthor] = useState("");
  const [family, setFamily] = useState("");
  const [language, setLanguage] = useState("und");
  const [rights, setRights] = useState(false);
  const [license, setLicense] = useState("");
  const [replacement, setReplacement] = useState("");
  const [published, setPublished] = useState("");
  const [validFrom, setValidFrom] = useState("");
  const [validUntil, setValidUntil] = useState("");
  useEffect(() => {
    setPublished(
      data?.source.published_at == null ? "" : String(data.source.published_at),
    );
    setValidFrom(
      data?.source.valid_from == null ? "" : String(data.source.valid_from),
    );
    setValidUntil(
      data?.source.valid_until == null ? "" : String(data.source.valid_until),
    );
  }, [data]);
  useEffect(() => {
    let alive = true;
    setData(null);
    setError("");
    policy
      .inspect(span)
      .then((value) => {
        if (alive) {
          setData(value);
          setAuthor(value.source.author ?? "");
          setFamily(value.source.source_family);
          setLanguage(value.source.language);
          setRights(value.source.redistributable);
          setLicense(value.source.license ?? "");
        }
      })
      .catch((e) => {
        if (alive) setError(e.message);
      });
    return () => {
      alive = false;
    };
  }, [span]);
  async function save() {
    if (!data) return;
    setBusy(true);
    setError("");
    try {
      await policy.metadata(data.source.id, {
        expected_version: data.source.version,
        author: author || null,
        language,
        source_family: family,
        published_at: published ? Number(published) : null,
        valid_from: validFrom ? Number(validFrom) : null,
        valid_until: validUntil ? Number(validUntil) : null,
        redistributable: rights,
        license: license || null,
      });
      setError(
        "Saved a new source version. Refresh the analysis; dependent reviews need reconsideration.",
      );
    } catch (e) {
      setError(e instanceof Error ? e.message : "Metadata update failed");
    } finally {
      setBusy(false);
    }
  }
  return (
    <aside
      aria-label="Exact source passage"
      className="min-w-0 rounded-md border bg-background p-4"
    >
      <div className="flex justify-between gap-2">
        <h2 className="font-semibold">Exact source passage</h2>
        <button
          onClick={onClose}
          className="rounded border px-3 py-1 focus-visible:ring-2"
        >
          Close
        </button>
      </div>
      {error && (
        <p role="status" className="my-3 text-sm">
          {error}
        </p>
      )}
      {!data && !error && <p role="status">Loading the pinned version…</p>}
      {data && (
        <>
          <p className="mt-3 font-medium">
            {data.source.title} · v{data.source.version}
          </p>
          <p className="text-sm">
            {data.source.author ?? "Author unknown"} · {span.original_language}{" "}
            · bytes {span.start}–{span.end}
          </p>
          <p className="mt-2 break-all text-xs">SHA-256: {span.text_hash}</p>
          <blockquote className="my-4 whitespace-pre-wrap border-l-2 pl-3 text-sm">
            {data.passage}
          </blockquote>
          <p className="text-sm text-muted-foreground">
            Exact extracted text; citation integrity does not verify the
            underlying statement.
          </p>
          <details className="mt-4">
            <summary className="cursor-pointer">
              Source metadata and export rights
            </summary>
            <div className="mt-3 space-y-2 text-sm">
              <label className="block">
                Author
                <input
                  className="block w-full rounded border bg-background p-2"
                  value={author}
                  onChange={(e) => setAuthor(e.target.value)}
                />
              </label>
              <label className="block">
                Original language
                <input
                  className="block w-full rounded border bg-background p-2"
                  value={language}
                  onChange={(e) => setLanguage(e.target.value)}
                />
              </label>
              <label className="block">
                Common source origin
                <input
                  className="block w-full rounded border bg-background p-2"
                  value={family}
                  onChange={(e) => setFamily(e.target.value)}
                />
              </label>
              <label className="block">
                Publication time (UTC Unix seconds; empty means unknown)
                <input
                  type="number"
                  className="block w-full rounded border bg-background p-2"
                  value={published}
                  onChange={(e) => setPublished(e.target.value)}
                />
              </label>
              <label className="block">
                World-valid from (UTC Unix seconds; empty means unknown)
                <input
                  type="number"
                  className="block w-full rounded border bg-background p-2"
                  value={validFrom}
                  onChange={(e) => setValidFrom(e.target.value)}
                />
              </label>
              <label className="block">
                World-valid until (exclusive UTC Unix seconds)
                <input
                  type="number"
                  className="block w-full rounded border bg-background p-2"
                  value={validUntil}
                  onChange={(e) => setValidUntil(e.target.value)}
                />
              </label>
              <label className="flex gap-2">
                <input
                  type="checkbox"
                  checked={rights}
                  onChange={(e) => setRights(e.target.checked)}
                />
                I have rights to include this evidence in an export
              </label>
              <label className="block">
                License or rights basis
                <input
                  className="block w-full rounded border bg-background p-2"
                  value={license}
                  onChange={(e) => setLicense(e.target.value)}
                />
              </label>
              <button
                disabled={busy}
                onClick={save}
                className="rounded border px-3 py-2 disabled:opacity-50"
              >
                Save new metadata version
              </button>
              <label className="block">
                Replace with a ready document ID
                <input
                  value={replacement}
                  onChange={(e) => setReplacement(e.target.value)}
                  className="block w-full rounded border bg-background p-2"
                />
              </label>
              <p>
                Upload changed evidence in Documents first, then copy its ID.
                Earlier citations stay pinned; dependent records require review.
              </p>
              <button
                disabled={busy || !replacement.trim()}
                className="rounded border px-3 py-2 disabled:opacity-50"
                onClick={async () => {
                  if (!data) return;
                  setBusy(true);
                  try {
                    await policy.replace(data.source.id, {
                      expected_version: data.source.version,
                      document_id: replacement,
                    });
                    setError(
                      "Replacement recorded. Reopen the latest source before further changes; earlier assessments require review.",
                    );
                  } catch (e) {
                    setError(
                      e instanceof Error ? e.message : "Replacement failed",
                    );
                  } finally {
                    setBusy(false);
                  }
                }}
              >
                Record changed evidence
              </button>
            </div>
          </details>
        </>
      )}
    </aside>
  );
}
