import { test, expect } from "@playwright/test";
import { randomUUID } from "node:crypto";
import { performDevLogin } from "./helpers/qc-auth";

test("real policy evidence upload, answer, exact passage and reload", async ({
  page,
}) => {
  test.setTimeout(900000);
  if (
    process.env.TACITUS_CLOUD_EXECUTE !== "1" ||
    !process.env.E2E_PASSWORD ||
    !process.env.TACITUS_TENANT ||
    !process.env.TACITUS_WORKSPACE
  )
    throw new Error(
      "Real-cloud test requires explicit execution flag, credentials and synthetic workspace IDs",
    );
  const failures: string[] = [];
  page.on("pageerror", (error) => failures.push(error.message));
  const tenant = process.env.TACITUS_TENANT;
  const workspace = process.env.TACITUS_WORKSPACE;
  await page.addInitScript(
    ({ tenant, workspace }) => {
      localStorage.setItem("tenantId", tenant);
      localStorage.setItem("workspaceId", workspace);
    },
    { tenant, workspace },
  );
  const login = await performDevLogin(page, {
    username: process.env.E2E_USERNAME ?? "admin",
    password: process.env.E2E_PASSWORD,
  });
  expect(login.loggedIn).toBe(true);
  await page.goto("/graph");
  const policyGraph = page
    .locator("section")
    .filter({
      has: page.getByRole("heading", { name: "Attributed policy graph" }),
    });
  await expect(policyGraph.getByLabel("Domain vocabulary")).toBeVisible();
  await policyGraph
    .getByLabel("Domain vocabulary")
    .selectOption("public-policy");
  await policyGraph
    .getByText("Inspect entity and relation definitions", { exact: true })
    .click();
  await expect(
    policyGraph.getByText("FUNDING_STAGE", { exact: true }),
  ).toBeVisible();
  await expect(
    policyGraph.getByRole("button", { name: "Apply extraction profile" }),
  ).toBeDisabled();
  await policyGraph
    .getByRole("checkbox", { name: /Replace this workspace/ })
    .check();
  const applyResponse = page.waitForResponse(
    (r) =>
      r.url().endsWith("/tacitus/ontologies/apply") &&
      r.request().method() === "POST",
  );
  await policyGraph
    .getByRole("button", { name: "Apply extraction profile" })
    .click();
  expect((await applyResponse).ok()).toBe(true);
  await expect(
    policyGraph.getByText(/Profile saved for future ingestion/),
  ).toBeVisible();
  await page.goto("/");
  await expect(
    page.getByRole("heading", { name: "Ask Tacitus" }),
  ).toBeVisible();
  const marker = `BROWSER_${randomUUID().replaceAll("-", "")}`;
  await page.getByLabel("Upload authorized evidence").setInputFiles({
    name: `${marker}.txt`,
    mimeType: "text/plain",
    buffer: Buffer.from(
      `Public synthetic test, CC0.\n\nThe Council authorized ${marker}. However, expenditure requires a separate appropriation. The Assembly has not approved the appropriation.`,
    ),
  });
  await page.getByText("Evidence selection and time", { exact: true }).click();
  const row = page.locator("label").filter({ hasText: `${marker}.txt` });
  await expect(row.getByRole("checkbox")).toBeEnabled({ timeout: 650000 });
  await row.getByRole("checkbox").check();
  await page
    .getByLabel("Question", { exact: true })
    .fill(
      `What authorization and expenditure distinction applies to ${marker}?`,
    );
  await page.getByRole("button", { name: "Generate cited analysis" }).click();
  const answer = page.getByRole("region", { name: "Policy analysis" });
  await expect(
    answer.getByText("completed", { exact: false }).first(),
  ).toBeVisible({ timeout: 650000 });
  await answer
    .getByRole("button", { name: /E\d+: inspect passage/ })
    .first()
    .click();
  const inspector = page.getByLabel("Exact source passage");
  await expect(inspector.locator("blockquote")).toContainText(marker);
  await expect(inspector.locator("blockquote")).toContainText(
    "separate appropriation",
  );
  await page.reload();
  await page.getByRole("button").filter({ hasText: marker }).last().click();
  await expect(
    page
      .getByRole("region", { name: "Policy analysis" })
      .getByText("completed", { exact: false })
      .first(),
  ).toBeVisible();
  await page.goto("/graph");
  await expect(
    policyGraph.getByText(/Recorded profile: public-policy/),
  ).toBeVisible();
  const fundingResponse = page.waitForResponse(
    (r) =>
      r.url().includes("/tacitus/graph?") &&
      new URL(r.url()).searchParams.get("tool") === "funding",
  );
  await policyGraph
    .getByLabel("Graph tool", { exact: true })
    .selectOption("funding");
  const funding = await (await fundingResponse).json();
  expect(funding.tool.id).toBe("funding");
  expect(
    funding.edges.every((e: { label: string }) =>
      funding.tool.predicates.includes(e.label),
    ),
  ).toBe(true);
  const cutoffResponse = page.waitForResponse(
    (r) =>
      r.url().includes("/tacitus/graph?") &&
      new URL(r.url()).searchParams.has("known_as_of"),
  );
  await policyGraph
    .getByLabel("Known as of (your local time)", { exact: true })
    .fill("1970-01-02T00:00");
  expect((await (await cutoffResponse).json()).edges).toEqual([]);
  await expect(
    policyGraph.getByText(/No authorized assertions match these filters/),
  ).toBeVisible();
  expect(failures).toEqual([]);
});
