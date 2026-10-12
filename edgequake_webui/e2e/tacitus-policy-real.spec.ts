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
  await page.goto("/");
  await expect(
    page.getByRole("heading", { name: "Ask Tacitus" }),
  ).toBeVisible();
  const marker = `BROWSER_${randomUUID().replaceAll("-", "")}`;
  await page
    .getByLabel("Upload authorized evidence")
    .setInputFiles({
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
  expect(failures).toEqual([]);
});
