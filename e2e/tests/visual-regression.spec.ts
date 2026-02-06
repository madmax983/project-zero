import { test, expect } from "@playwright/test";

test.describe("Visual Regression", () => {
  test("main menu screenshot", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("pre").first()).toBeVisible({ timeout: 30_000 });
    await expect(page.getByText("Start Game")).toBeVisible({ timeout: 5_000 });

    await expect(page).toHaveScreenshot("main-menu.png", {
      maxDiffPixelRatio: 0.05,
    });
  });

  test("game view screenshot", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("pre").first()).toBeVisible({ timeout: 30_000 });

    // Start the game
    await page.keyboard.press("Enter");
    await expect(page.getByText("Start Game")).not.toBeVisible({
      timeout: 5_000,
    });

    // Wait a moment for the game to render
    await page.waitForTimeout(500);

    await expect(page).toHaveScreenshot("game-view.png", {
      maxDiffPixelRatio: 0.1, // Game has animated elements, allow more variance
    });
  });
});
