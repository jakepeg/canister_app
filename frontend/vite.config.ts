import { sveltekit } from "@sveltejs/kit/vite";
import type { UserConfig } from "vite";
import { defineConfig, loadEnv } from "vite";
import {
  host,
  iiUrl,
  network,
  readCanisterIds,
} from "./scripts/readCanisterIds";
import path from "path";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";
// Import the new polyfill plugin
import { nodePolyfills } from 'vite-plugin-node-polyfills';


const config: UserConfig = {
  // Add nodePolyfills plugin here
  plugins: [wasm(), topLevelAwait(), sveltekit(), nodePolyfills()], // Moved wasm() and topLevelAwait() first
  resolve: {
    alias: {
      $lib: path.resolve("./src/frontend/src/lib"),
      // Removed alias for ic-vetkd-utils to let Vite handle it
    },
  },
  // Remove optimizeDeps polyfill config, let the plugin handle it
  optimizeDeps: {
    esbuildOptions: {
      // Node.js global to browser globalThis - Still might be needed depending on dependencies
      define: {
        global: "globalThis",
      },
    },
  },
  // Keep single build config
  build: {
    target: "es2020",
    rollupOptions: {}, // Remove the polyfill plugin from here
  },
  // Add SSR configuration to process ic-vetkd-utils
  ssr: {
    noExternal: ['ic-vetkd-utils'],
  },
};

export default defineConfig(({ mode }: UserConfig): UserConfig => {
  // Expand environment - .env files - with canister IDs
  process.env = {
    ...process.env,
    ...loadEnv(mode ?? "development", process.cwd()),
    ...readCanisterIds({ prefix: "VITE_" }),
    VITE_DFX_NETWORK: network,
    VITE_HOST: host,
    VITE_II_URL: iiUrl,
  };

  return {
    ...config,
    // Backwards compatibility for auto generated types of dfx that are meant for webpack and process.env
    define: {
      "process.env": {
        ...readCanisterIds({}),
        DFX_NETWORK: network,
      },
    },
  };
});
