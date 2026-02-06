import { test, expect } from "@playwright/test";

test.describe("Smoke Tests", () => {
  test("WASM app loads and renders", async ({ page }) => {
    await page.goto("/");

    // Wait for WASM to initialize — DomBackend renders into <pre> elements
    const pre = page.locator("pre");
    await expect(pre.first()).toBeVisible({ timeout: 30_000 });
  });

  test("page title is SCALE", async ({ page }) => {
    await page.goto("/");
    await expect(page).toHaveTitle("SCALE");
  });

  test("no console errors during load", async ({ page }) => {
    const errors: string[] = [];
    page.on("console", (msg) => {
      if (msg.type() === "error") {
        errors.push(msg.text());
      }
    });

    await page.goto("/");
    await page.waitForTimeout(3_000);

    // Filter out expected WASM-related warnings
    const realErrors = errors.filter(
      (e) => !e.includes("wasm") && !e.includes("SharedArrayBuffer"),
    );
    expect(realErrors).toHaveLength(0);
  });
});
