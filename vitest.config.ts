import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    environment: "node",
    setupFiles: ["tests/setup/vitest.setup.ts"],
    include: ["src/**/*.test.ts", "src/**/*.test.tsx", "tests/**/*.test.ts", "tests/**/*.test.tsx"],
    coverage: {
      provider: "v8",
      reporter: ["text", "json-summary", "lcov"],
      exclude: [
        "**/tests/**",
        "**/scripts/**",
        "**/dist/**",
        "**/playwright-report/**",
        "**/test-results/**",
        "**/vite.config.ts",
        "**/playwright.config.ts",
      ],
      thresholds: {
        lines: 20,
        branches: 30,
        functions: 20,
        statements: 20,
      },
    },
  },
});
