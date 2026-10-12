// Log in privately using the generated bootstrap config, then exercise the real API.
import { readFile } from "node:fs/promises";
import { runAcceptance } from "./tacitus-cloud-acceptance.mjs";
import { setSessionTokenSupplier } from "./tacitus-policy.mjs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
async function main() {
  if (process.env.TACITUS_CLOUD_EXECUTE !== "1")
    throw new Error(
      "Set TACITUS_CLOUD_EXECUTE=1 on the chosen cloud host; this creates synthetic test workspaces/documents.",
    );
  const phase = process.argv[2];
  if (phase && !["after-restart"].includes(phase))
    throw new Error("Usage: tacitus-cloud-test.mjs [after-restart]");
  const config = JSON.parse(
    await readFile(resolve(root, ".tacitus/stack/config.json"), "utf8"),
  );
  if (config.apiPort !== 8080)
    throw new Error(
      "Acceptance CLI currently requires API port 8080. Keep the documented default port.",
    );
  const base = "http://127.0.0.1:8080/api/v1";
  async function call(path, token, body) {
    const response = await fetch(base + path, {
      method: body === undefined ? "GET" : "POST",
      redirect: "error",
      signal: AbortSignal.timeout(30000),
      headers: {
        "Content-Type": "application/json",
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
      },
      body: body === undefined ? undefined : JSON.stringify(body),
    });
    if (!response.ok)
      throw new Error(
        `Cloud test setup API ${response.status} at ${path}; inspect private server diagnostics`,
      );
    return response.json();
  }
  let token;
  let expiresAt = 0;
  async function session() {
    if (Date.now() + 60000 >= expiresAt) {
      const login = await call("/auth/login", null, {
        username: process.env.E2E_USERNAME ?? "admin",
        password: process.env.E2E_PASSWORD ?? config.adminPassword,
      });
      if (typeof login.access_token !== "string" || !login.access_token)
        throw new Error("Authenticated access token missing");
      token = login.access_token;
      expiresAt = Date.now() + (Number(login.expires_in) || 900) * 1000;
    }
    return token;
  }
  const receipt = phase
    ? JSON.parse(
        await readFile(resolve(root, ".tacitus/cloud-acceptance.json"), "utf8"),
      )
    : null;
  const requestedTenant = receipt?.tenant ?? process.env.TACITUS_TENANT;
  const tenants = await call("/tenants", await session());
  const tenant =
    tenants.items?.find((t) => t.id === requestedTenant) ??
    (!requestedTenant ? tenants.items?.[0] : null);
  if (!tenant)
    throw new Error(
      "No accessible tenant found. Create one in the browser, or set TACITUS_TENANT to its ID.",
    );
  const workspaces = await call(
    `/tenants/${tenant.id}/workspaces`,
    await session(),
  );
  const workspace = workspaces.items?.[0];
  if (!workspace)
    throw new Error("Create an accessible workspace in the browser first.");
  process.env.TACITUS_TENANT = tenant.id;
  process.env.TACITUS_WORKSPACE = workspace.id;
  process.chdir(root);
  setSessionTokenSupplier(session);
  try {
    await runAcceptance(phase);
  } finally {
    setSessionTokenSupplier(null);
  }
  console.log(
    "Real API acceptance completed. Browser, offline-network and audit verification remain separate gates.",
  );
}
main().catch((e) => {
  console.error(e.message);
  process.exitCode = 1;
});
