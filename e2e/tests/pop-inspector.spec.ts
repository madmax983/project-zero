import { test, expect } from "@playwright/test";
import { startGame } from "./helpers";

test.describe("Pop Inspector", () => {
  test.beforeEach(async ({ page }) => {
    await startGame(page);
  });

  test("clicking a pop shows colonist inspector with morale", async ({ page }) => {
    // Pops render as ☺ (happy), ☻ (moderate), or ☹ (starving).
    // At game start they are well-fed so ☺ is expected.
    // Ratzilla renders each cell as a <span>, so we can locate and click it.
    const popSpan = page.locator("span", { hasText: "☺" }).first();
    await expect(popSpan).toBeVisible({ timeout: 5_000 });
    await popSpan.click();

    // Inspector should NOT show "Terrain:" (confirms entity mode, not tile mode)
    await expect(page.getByText("Terrain:")).not.toBeVisible({ timeout: 2_000 });

    // Inspector should show Morale
    await expect(page.getByText("Morale")).toBeVisible({ timeout: 5_000 });
  });

  test("selected pop shows needs gauges", async ({ page }) => {
    const popSpan = page.locator("span", { hasText: "☺" }).first();
    await expect(popSpan).toBeVisible({ timeout: 5_000 });
    await popSpan.click();

    await expect(page.getByText("Hunger")).toBeVisible({ timeout: 5_000 });
    await expect(page.getByText("Rest")).toBeVisible({ timeout: 5_000 });
  });

  test("selected pop shows current action", async ({ page }) => {
    const popSpan = page.locator("span", { hasText: "☺" }).first();
    await expect(popSpan).toBeVisible({ timeout: 5_000 });
    await popSpan.click();

    await expect(page.getByText("Hunger")).toBeVisible({ timeout: 5_000 });

    // Pop should be doing one of the possible actions
    const actions = [
      "Eating",
      "Sleeping",
      "Socializing",
      "Exploring",
      "Working",
      "Repairing",
      "Researching",
      "Hauling",
      "Healing",
      "Idle",
    ];
    const actionRegex = new RegExp(actions.join("|"));
    await expect(page.getByText(actionRegex)).toBeVisible({ timeout: 5_000 });
  });

  test("selected pop shows biography", async ({ page }) => {
    const popSpan = page.locator("span", { hasText: "☺" }).first();
    await expect(popSpan).toBeVisible({ timeout: 5_000 });
    await popSpan.click();

    await expect(page.getByText("Hunger")).toBeVisible({ timeout: 5_000 });
    await expect(page.getByText("Biography")).toBeVisible({ timeout: 5_000 });
  });

  test("Escape clears pop selection", async ({ page }) => {
    const popSpan = page.locator("span", { hasText: "☺" }).first();
    await expect(popSpan).toBeVisible({ timeout: 5_000 });
    await popSpan.click();

    await expect(page.getByText("Hunger")).toBeVisible({ timeout: 5_000 });

    await page.keyboard.press("Escape");
    await expect(page.getByText("Hunger")).not.toBeVisible({ timeout: 5_000 });
  });
});
