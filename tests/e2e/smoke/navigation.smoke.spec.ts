import { expect, test } from "@playwright/test";

test("@smoke home loads and dashboard navigation works", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByRole("heading", { name: "IncidentReview" })).toBeVisible();
  await expect(page.getByRole("navigation", { name: "Navigation" })).toBeVisible();

  await page.getByRole("link", { name: "Dashboards" }).click();
  await expect(page).toHaveURL(/#dashboards$/);
  await expect(page.getByText("All metrics are computed deterministically in Rust")).toBeVisible();
});
