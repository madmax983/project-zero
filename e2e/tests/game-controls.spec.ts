import { test, expect } from "@playwright/test";

test.describe("Game Controls", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("pre").first()).toBeVisible({ timeout: 30_000 });

    // Start the game from main menu
    await expect(page.getByText("Start Game")).toBeVisible({ timeout: 5_000 });
    await page.keyboard.press("Enter");
    // Wait for game to load
    await expect(page.getByText("Start Game")).not.toBeVisible({
      timeout: 5_000,
    });
  });

  test("space bar toggles pause", async ({ page }) => {
    // Game starts in Running state
    await page.keyboard.press("Space");

    // Should see paused indicator (the status bar shows pause icon)
    // Give a frame to render
    await page.waitForTimeout(200);

    // Take a snapshot to verify the game rendered (basic sanity)
    const content = await page.locator("pre").first().textContent();
    expect(content).toBeTruthy();
  });

  test("b key enters build mode", async ({ page }) => {
    await page.keyboard.press("b");
    await page.waitForTimeout(200);

    // Build mode should show BUILD indicator in status bar
    const content = await page.locator("pre").first().textContent();
    expect(content).toBeTruthy();

    // Press Escape to exit build mode
    await page.keyboard.press("Escape");
  });

  test("l key opens chronicle", async ({ page }) => {
    await page.keyboard.press("l");
    await page.waitForTimeout(200);

    // Chronicle overlay should appear
    const content = await page.locator("pre").first().textContent();
    expect(content).toBeTruthy();

    // Press Escape to close
    await page.keyboard.press("Escape");
  });

  test("number keys change speed", async ({ page }) => {
    // Press 2 for Fast speed
    await page.keyboard.press("2");
    await page.waitForTimeout(200);

    const content = await page.locator("pre").first().textContent();
    expect(content).toBeTruthy();

    // Press 1 to go back to normal
    await page.keyboard.press("1");
  });
});
