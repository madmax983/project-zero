import { test, expect } from "@playwright/test";
import { startGame } from "./helpers";

test.describe("Game Controls", () => {
  test.beforeEach(async ({ page }) => {
    await startGame(page);
  });

  test("space bar toggles pause", async ({ page }) => {
    await page.keyboard.press("Space");
    await page.waitForTimeout(200);

    // When paused via Space, the play/pause indicator changes to ⏸
    await expect(page.getByText("⏸")).toBeVisible({ timeout: 5_000 });

    // Unpause — ⏸ should disappear from the play indicator
    await page.keyboard.press("Space");
    await expect(page.getByText("⏸")).not.toBeVisible({ timeout: 5_000 });
  });

  test("b key enters build mode", async ({ page }) => {
    await page.keyboard.press("b");

    // Build mode should show BUILD indicator in status bar
    await expect(page.getByText("BUILD:")).toBeVisible({ timeout: 5_000 });

    await page.keyboard.press("Escape");
  });

  test("l key opens chronicle", async ({ page }) => {
    await page.keyboard.press("l");

    // Chronicle overlay should appear with its title
    await expect(page.getByText("Chronicle")).toBeVisible({ timeout: 5_000 });

    await page.keyboard.press("Escape");
  });

  test("number keys change speed", async ({ page }) => {
    // Press 2 for Fast speed (3x)
    await page.keyboard.press("2");
    await page.waitForTimeout(200);

    await expect(page.getByText("3x")).toBeVisible({ timeout: 5_000 });

    // Press 1 to go back to normal (1x)
    await page.keyboard.press("1");
    await page.waitForTimeout(200);

    await expect(page.getByText("1x")).toBeVisible({ timeout: 5_000 });
  });
});
