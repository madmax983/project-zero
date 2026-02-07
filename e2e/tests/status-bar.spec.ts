import { test, expect } from "@playwright/test";
import { startGame } from "./helpers";

test.describe("Status Bar", () => {
  test.beforeEach(async ({ page }) => {
    await startGame(page);
  });

  test("shows day counter", async ({ page }) => {
    await expect(page.getByText(/Day \d+/)).toBeVisible({ timeout: 5_000 });
  });

  test("shows season name", async ({ page }) => {
    await expect(
      page.getByText(/Spring|Summer|Autumn|Winter/),
    ).toBeVisible({ timeout: 5_000 });
  });

  test("shows population count", async ({ page }) => {
    await expect(page.getByText("Souls:")).toBeVisible({ timeout: 5_000 });
  });

  test("shows morale", async ({ page }) => {
    await expect(page.getByText("Morale:")).toBeVisible({ timeout: 5_000 });
  });

  test("shows yield", async ({ page }) => {
    await expect(page.getByText("Yield:")).toBeVisible({ timeout: 5_000 });
  });

  test("shows speed indicator", async ({ page }) => {
    await expect(page.getByText("1x")).toBeVisible({ timeout: 5_000 });
  });
});
