// @ts-check

import { includeIgnoreFile } from "@eslint/compat";
import js from "@eslint/js";
import prettier from "eslint-config-prettier";
import svelte from "eslint-plugin-svelte";
import globals from "globals";
import { fileURLToPath } from "node:url";
import tseslint from "typescript-eslint";
import svelteConfig from "./svelte.config.js";
import simpleImportSort from "eslint-plugin-simple-import-sort";

const gitignorePath = fileURLToPath(new URL("./.gitignore", import.meta.url));

export default tseslint.config(
  includeIgnoreFile(gitignorePath),
  {
    ignores: ["*.js"],
  },
  js.configs.recommended,
  tseslint.configs.recommendedTypeChecked,
  svelte.configs["flat/recommended"],
  prettier,
  svelte.configs["flat/prettier"],
  {
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
      globals: globals.browser,
    },
  },
  {
    files: ["**/*.svelte", "**/*.svelte.ts"],
    languageOptions: {
      parserOptions: {
        extraFileExtensions: [".svelte"],
        parser: tseslint.parser,
        svelteFeatures: {
          experimentalGenerics: true,
        },
        svelteConfig,
      },
    },
  },
  {
    plugins: { "simple-import-sort": simpleImportSort },
    rules: {
      "@typescript-eslint/no-unused-vars": [
        "warn",
        {
          argsIgnorePattern: "^_",
        },
      ],
      "@typescript-eslint/switch-exhaustiveness-check": [
        "error",
        {
          requireDefaultForNonUnion: true,
          considerDefaultExhaustiveForUnions: true,
        },
      ],
      "@typescript-eslint/strict-boolean-expressions": [
        "error",
        { allowNullableBoolean: true },
      ],
      "simple-import-sort/imports": "warn",
      "simple-import-sort/exports": "warn",
      // these are falsely erroring when using generic components
      "@typescript-eslint/no-unsafe-return": "off",
      "@typescript-eslint/no-unsafe-member-access": "off",
      "@typescript-eslint/no-unsafe-argument": "off",
      "@typescript-eslint/no-unsafe-assignment": "off",
    },
  },
);
