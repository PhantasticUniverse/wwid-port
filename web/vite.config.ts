import { defineConfig } from "vite";
import solid from "vite-plugin-solid";
import tailwindcss from "@tailwindcss/vite";
import { resolve } from "path";

export default defineConfig({
  plugins: [solid(), tailwindcss()],
  resolve: {
    alias: {
      "@wasm": resolve(__dirname, "wasm"),
    },
  },
  build: {
    target: "es2022",
  },
  worker: {
    format: "es",
  },
  server: {
    fs: {
      // Allow serving files from the wasm symlink target
      allow: [".."],
    },
  },
});
