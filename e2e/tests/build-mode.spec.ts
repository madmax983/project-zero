import { test, expect } from "@playwright/test";
import { startGame } from "./helpers";

test.describe("Build Mode", () => {
  test.beforeEach(async ({ page }) => {
    await startGame(page);
  });

  test("entering build mode shows BUILD indicator", async ({ page }) => {
    await page.keyboard.press("b");
    await expect(page.getByText("BUILD:")).toBeVisible({ timeout: 5_000 });
  });

  test("Tab cycles building types", async ({ page }) => {
    await page.keyboard.press("b");
    await expect(page.getByText("BUILD:")).toBeVisible({ timeout: 5_000 });

    const pre = page.locator("pre").first();
    const beforeTab = await pre.textContent();

    await page.keyboard.press("Tab");
    await page.waitForTimeout(200);

    const afterTab = await pre.textContent();
    // The building type name should change after pressing Tab
    expect(afterTab).not.toBe(beforeTab);
  });

  test("Escape exits build mode", async ({ page }) => {
    await page.keyboard.press("b");
    await expect(page.getByText("BUILD:")).toBeVisible({ timeout: 5_000 });

    await page.keyboard.press("Escape");
    await expect(page.getByText("BUILD:")).not.toBeVisible({ timeout: 5_000 });
  });

  test("shows build mode hotkeys", async ({ page }) => {
    await page.keyboard.press("b");
    await expect(page.getByText("Tab:switch")).toBeVisible({ timeout: 5_000 });
    await expect(page.getByText("Esc:exit")).toBeVisible({ timeout: 5_000 });
  });
});
