// Authenticated CLI for the same persistent API as the browser. No local kernel or database.
import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
let sessionTokenSupplier = null;
export function setSessionTokenSupplier(supplier) {
  if (supplier !== null && typeof supplier !== "function")
    throw new Error("Token supplier must be a function or null");
  sessionTokenSupplier = supplier;
}
export async function request(
  path,
  body,
  method = body === undefined ? "GET" : "POST",
) {
  const url = process.env.TACITUS_API_URL ?? "http://127.0.0.1:8080";
  if (!["http://127.0.0.1:8080", "http://localhost:8080"].includes(url))
    throw new Error("Use the documented loopback SSH tunnel on port 8080");
  const token = sessionTokenSupplier
    ? await sessionTokenSupplier()
    : process.env.TACITUS_TOKEN;
  const tenant = process.env.TACITUS_TENANT;
  const workspace = process.env.TACITUS_WORKSPACE;
  if (
    !token ||
    ![tenant, workspace].every(
      (v) => typeof v === "string" && /^[a-f0-9-]{36}$/i.test(v),
    )
  )
    throw new Error(
      "Set TACITUS_TOKEN, TACITUS_TENANT and TACITUS_WORKSPACE privately",
    );
  const response = await fetch(`${url}/api/v1${path}`, {
    method,
    redirect: "error",
    signal: AbortSignal.timeout(650000),
    headers: {
      Authorization: `Bearer ${token}`,
      "X-Tenant-ID": tenant,
      "X-Workspace-ID": workspace,
      "Content-Type": "application/json",
    },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  if (!response.ok)
    throw new Error(
      `API ${response.status}: ${(await response.text()).slice(0, 1000)}`,
    );
  return response.json();
}
export async function waitAnalysis(id) {
  const deadline = Date.now() + 650000;
  while (Date.now() < deadline) {
    const job = await request(`/tacitus/analyses/${encodeURIComponent(id)}`);
    if (job.status === "completed") return job;
    if (["failed", "cancelled", "interrupted"].includes(job.status))
      throw new Error(`Analysis ${job.status}: ${job.error ?? "no result"}`);
    await new Promise((r) => setTimeout(r, 1500));
  }
  throw new Error("Analysis timeout; inspect or cancel its persisted job");
}
async function main([command, ...args]) {
  if (command === "upload") {
    const [file] = args;
    if (!file) throw new Error("upload FILE.txt|FILE.md");
    const content = await readFile(file, "utf8");
    if (!content.trim() || Buffer.byteLength(content) > 2 * 1024 * 1024)
      throw new Error("Nonempty text under 2 MiB required");
    return request("/documents", {
      title: file.split(/[\\/]/).pop(),
      content,
      async_processing: true,
    });
  }
  if (command === "ask") {
    const [file] = args;
    if (!file) throw new Error("ask REQUEST.json");
    const body = JSON.parse(await readFile(file, "utf8"));
    const job = await request("/tacitus/analyses", body);
    process.stderr.write(`Persisted analysis ${job.id}\n`);
    return waitAnalysis(job.id);
  }
  if (command === "inspect") {
    const [id, version, start, end] = args;
    if (!id || ![version, start, end].every((v) => /^\d+$/.test(v ?? "")))
      throw new Error("inspect SOURCE_UUID VERSION START_BYTE END_BYTE");
    return request(
      `/tacitus/sources/${encodeURIComponent(id)}?version=${version}&start=${start}&end=${end}`,
    );
  }
  if (
    ["sources", "methods", "graph", "impact", "status", "analyses"].includes(
      command,
    )
  )
    return request(`/tacitus/${command}`);
  if (command === "cancel")
    return request(
      `/tacitus/analyses/${encodeURIComponent(args[0])}/cancel`,
      {},
    );
  if (
    [
      "propose",
      "suggest",
      "review",
      "metadata",
      "replace",
      "sync",
      "preview",
      "export",
    ].includes(command)
  ) {
    const [file, out] = args;
    if (!file) throw new Error(`${command} REQUEST.json [OUTPUT.capsule]`);
    const body = JSON.parse(await readFile(file, "utf8"));
    const path = {
      propose: "/tacitus/methods",
      suggest: "/tacitus/methods/suggest",
      review: "/tacitus/reviews",
      sync: "/tacitus/sources",
      preview: "/tacitus/capsules/preview",
      export: "/tacitus/capsules/export",
      metadata: `/tacitus/sources/${encodeURIComponent(body.source_id)}`,
      replace: `/tacitus/sources/${encodeURIComponent(body.source_id)}/replacement`,
    }[command];
    if (["metadata", "replace"].includes(command)) delete body.source_id;
    if (command === "export" && !out)
      throw new Error(
        "Export requires a new output file path and expected_preview_hash from exact preview",
      );
    const result = await request(
      path,
      body,
      command === "metadata" ? "PATCH" : "POST",
    );
    if (command === "export") {
      await writeFile(out, Buffer.from(result.capsule_base64, "base64"), {
        flag: "wx",
        mode: 0o600,
      });
      return { filename: out, preview_hash: result.preview_hash };
    }
    return result;
  }
  if (command === "import") {
    const bytes = await readFile(args[0]);
    if (bytes.length > 8 * 1024 * 1024) throw new Error("8 MiB capsule limit");
    return request("/tacitus/capsules/import", {
      capsule_base64: bytes.toString("base64"),
    });
  }
  throw new Error(
    "Commands: upload, ask, inspect, sources, methods, graph, impact, status, analyses, cancel, propose, review, metadata, sync, preview, export, import. See docs/tacitus/policy-mvp.md",
  );
}
if (
  process.argv[1] &&
  resolve(process.argv[1]) === fileURLToPath(import.meta.url)
)
  main(process.argv.slice(2))
    .then((result) =>
      process.stdout.write(JSON.stringify(result, null, 2) + "\n"),
    )
    .catch((e) => {
      process.stderr.write(e.message + "\n");
      process.exitCode = 1;
    });
