module.exports = {
  root: true,
  env: { browser: true, es2022: true },
  parser: "@typescript-eslint/parser",
  plugins: [
    "@typescript-eslint",
    "react-hooks",
    "react-refresh",
    "jsx-a11y",
    "tailwindcss"
  ],
  extends: [
    "eslint:recommended",
    "plugin:@typescript-eslint/recommended",
    "plugin:react-hooks/recommended",
    "plugin:jsx-a11y/recommended"
  ],
  ignorePatterns: ["dist", "node_modules", "src-tauri", "target"],
  rules: {
    "react-refresh/only-export-components": ["warn", { allowConstantExport: true }],
    "tailwindcss/classnames-order": "warn",
    "tailwindcss/no-custom-classname": "off"
  }
};
