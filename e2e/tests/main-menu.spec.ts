import { test, expect } from "@playwright/test";

test.describe("Main Menu", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    // Wait for render
    await expect(page.locator("pre").first()).toBeVisible({ timeout: 30_000 });
  });

  test("displays SCALE title text", async ({ page }) => {
    // The main menu should render "SCALE" somewhere in the TUI
    await expect(page.getByText("SCALE")).toBeVisible({ timeout: 5_000 });
  });

  test("shows Start Game option", async ({ page }) => {
    await expect(page.getByText("Start Game")).toBeVisible({ timeout: 5_000 });
  });

  test("shows Quit option", async ({ page }) => {
    await expect(page.getByText("Quit")).toBeVisible({ timeout: 5_000 });
  });

  test("Enter starts the game", async ({ page }) => {
    // Wait for menu to be visible
    await expect(page.getByText("Start Game")).toBeVisible({ timeout: 5_000 });

    // Press Enter to start
    await page.keyboard.press("Enter");

    // After starting, the menu text should disappear and game UI should appear
    // The status bar contains speed/tick info that only shows in-game
    await expect(page.getByText("Start Game")).not.toBeVisible({
      timeout: 5_000,
    });
  });
});
