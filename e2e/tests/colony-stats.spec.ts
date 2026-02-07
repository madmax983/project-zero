import { test, expect } from "@playwright/test";
import { startGame } from "./helpers";

test.describe("Colony Stats Panel", () => {
  test.beforeEach(async ({ page }) => {
    await startGame(page);
  });

  test("shows demographics", async ({ page }) => {
    await expect(page.getByText("Population")).toBeVisible({ timeout: 5_000 });
  });

  test("shows food resource", async ({ page }) => {
    await expect(page.getByText(/Food/)).toBeVisible({ timeout: 5_000 });
  });

  test("shows wood resource", async ({ page }) => {
    await expect(page.getByText(/Wood/)).toBeVisible({ timeout: 5_000 });
  });

  test("shows stone resource", async ({ page }) => {
    await expect(page.getByText(/Stone/)).toBeVisible({ timeout: 5_000 });
  });

  test("shows tools resource", async ({ page }) => {
    await expect(page.getByText(/Tools/)).toBeVisible({ timeout: 5_000 });
  });

  test("shows fiber resource", async ({ page }) => {
    await expect(page.getByText(/Fiber/)).toBeVisible({ timeout: 5_000 });
  });

  test("shows cloth resource", async ({ page }) => {
    // "Cl Cloth" label — match Cloth but not Clothing
    await expect(page.getByText(/\bCloth\b/)).toBeVisible({ timeout: 5_000 });
  });

  test("shows clothing resource", async ({ page }) => {
    await expect(page.getByText(/Clothing/)).toBeVisible({ timeout: 5_000 });
  });
});
