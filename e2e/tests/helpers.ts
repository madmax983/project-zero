import { Page, expect } from "@playwright/test";

export async function loadApp(page: Page) {
  await page.goto("/");
  await expect(page.locator("pre").first()).toBeVisible({ timeout: 30_000 });
}

export async function startGame(page: Page) {
  await loadApp(page);
  await expect(page.getByText("Start Game")).toBeVisible({ timeout: 5_000 });
  await page.keyboard.press("Enter");
  await expect(page.getByText("Start Game")).not.toBeVisible({ timeout: 5_000 });
}
