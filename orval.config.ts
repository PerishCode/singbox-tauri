import { defineConfig } from "orval";

export default defineConfig({
  local: {
    input: {
      target: "http://127.0.0.1:18427/api/openapi.json",
    },
    output: {
      target: "./src/lib/api/generated.ts",
      client: "axios",
      mode: "single",
      prettier: false,
      override: {
        mutator: {
          path: "./src/lib/api/http.ts",
          name: "localApiRequest",
        },
      },
    },
  },
});
