import eslintPluginSvelte from "eslint-plugin-svelte";
import tseslint from "typescript-eslint";
import globals from "globals";
import svelteParser from "svelte-eslint-parser";
import tsParser from "@typescript-eslint/parser";

export default tseslint.config(
  {
    ignores: [
      "build/",
      ".svelte-kit/",
      "crates/",
      "node_modules/",
      "static/",
      "target/",
      "src-tauri/",
    ],
  },
  ...tseslint.configs.recommended,
  ...eslintPluginSvelte.configs.recommended,
  {
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node,
      },
    },
  },
  {
    files: ["**/*.svelte", "**/*.svelte.ts", "**/*.svelte.js"],
    languageOptions: {
      parser: svelteParser,
      parserOptions: {
        parser: tsParser,
      },
    },
  },
  {
    rules: {
      "@typescript-eslint/no-unused-vars": [
        "warn",
        { argsIgnorePattern: "^_" },
      ],
    },
  },
);
