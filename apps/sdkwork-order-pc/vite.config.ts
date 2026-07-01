import react from "@vitejs/plugin-react";
import path from "node:path";
import { defineConfig, loadEnv } from "vite";

const repoRoot = path.resolve(import.meta.dirname, "../..");
const orderAppSdkEntry = path.resolve(
  repoRoot,
  "sdks/sdkwork-order-app-sdk/sdkwork-order-app-sdk-typescript/generated/server-openapi/src/index.ts",
);

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, __dirname, "");

  return {
    define: {
      "process.env.SDKWORK_ACCESS_TOKEN": JSON.stringify(env.SDKWORK_ACCESS_TOKEN ?? ""),
    },
    plugins: [react()],
    resolve: {
      alias: [
        { find: "@sdkwork/order-app-sdk", replacement: orderAppSdkEntry },
        {
          find: "@sdkwork/order-contracts",
          replacement: path.resolve(
            repoRoot,
            "apps/sdkwork-order-common/packages/sdkwork-order-contracts/src/index.ts",
          ),
        },
        {
          find: "@sdkwork/order-sdk-ports",
          replacement: path.resolve(
            repoRoot,
            "apps/sdkwork-order-common/packages/sdkwork-order-sdk-ports/src/index.ts",
          ),
        },
        {
          find: "@sdkwork/order-service",
          replacement: path.resolve(
            repoRoot,
            "apps/sdkwork-order-common/packages/sdkwork-order-service/src/index.ts",
          ),
        },
      ],
    },
    server: {
      port: 5181,
      host: "127.0.0.1",
      fs: {
        allow: [repoRoot],
      },
    },
  };
});
