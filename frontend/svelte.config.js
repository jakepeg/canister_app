import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/kit/vite";
import { readFileSync } from "fs";
import { fileURLToPath } from "url";

const file = fileURLToPath(new URL("package.json", import.meta.url));
const json = readFileSync(file, "utf8");
const { version } = JSON.parse(json);

const filesPath = (path) => `src/frontend/${path}`;

/** @type {import('@sveltejs/kit').Config} */
const config = {
  // Consult https://github.com/sveltejs/svelte-preprocess
  // for more information about preprocessors
  preprocess: vitePreprocess(),
  kit: {
    // Configure adapter-static to create a fallback page for SPA routing
    adapter: adapter({
      // Pages can be dynamically generated (e.g., /users/123)
      // fallback: '404.html' // You can also use 404.html
      fallback: 'index.html' // Creates index.html as the fallback
    }),
    files: {
      assets: filesPath("static"),
      hooks: {
        client: filesPath("src/hooks.client"),
        server: filesPath("src/hooks.server"),
      },
      lib: filesPath("src/lib"),
      params: filesPath("src/params"),
      routes: filesPath("src/routes"),
      serviceWorker: filesPath("src/service-worker"),
      appTemplate: filesPath("src/app.html"),
      errorTemplate: filesPath("src/error.html"),
    },
    alias: {
      "@/*": "./src/frontend/src/lib/*",
    },
  },
  serviceWorker: {
    register: false,
  },
  version: {
    name: version,
  },
  trailingSlash: "always",
};
export default config;
