import { test, expect } from "@playwright/test";
import { startGame } from "./helpers";

test.describe("Simulation", () => {
  test("day counter advances", async ({ page }) => {
    await startGame(page);

    // Extract initial day number
    const dayText = page.getByText(/Day \d+/);
    await expect(dayText).toBeVisible({ timeout: 5_000 });
    const initialText = await dayText.textContent();
    const initialDay = parseInt(initialText!.match(/Day (\d+)/)![1], 10);

    // Wait for simulation to advance
    await page.waitForTimeout(3_000);

    const laterText = await dayText.textContent();
    const laterDay = parseInt(laterText!.match(/Day (\d+)/)![1], 10);
    expect(laterDay).toBeGreaterThan(initialDay);
  });

  test("map renders terrain", async ({ page }) => {
    await startGame(page);

    const content = await page.locator("pre").first().textContent();
    // Terrain uses period/dot chars for grass which are abundant
    expect(content).toContain(".");
  });

  test("map renders colony border", async ({ page }) => {
    await startGame(page);

    await expect(page.getByText("Colony")).toBeVisible({ timeout: 5_000 });
  });
});
