#!/usr/bin/env node

import { spawnSync } from "node:child_process";

const postgresUrl = process.env.ORDER_TEST_POSTGRES_URL?.trim();
const requireDatabase = process.argv.includes("--require-database");

if (!postgresUrl) {
  const message =
    "ORDER_TEST_POSTGRES_URL is unset; skipping postgres repository parity tests";
  if (requireDatabase) {
    console.error(message);
    process.exit(1);
  }
  console.log(message);
  process.exit(0);
}

const result = spawnSync(
  "cargo",
  ["test", "-p", "sdkwork-order-repository-sqlx", "postgres_", "--", "--nocapture"],
  {
    stdio: "inherit",
    shell: process.platform === "win32",
    env: { ...process.env, ORDER_TEST_POSTGRES_URL: postgresUrl },
  },
);

process.exit(result.status ?? 1);
