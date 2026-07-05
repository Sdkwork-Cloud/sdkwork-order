#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";

const root = process.argv[2] ?? process.cwd();
const targets = [
  "apis/app-api/order/order-app-api.openapi.json",
  "sdks/sdkwork-order-app-sdk/openapi/sdkwork-order-app-api.openapi.json",
  "sdks/sdkwork-order-app-sdk/openapi/sdkwork-order-app-api.sdkgen.json",
];

const ORDER_OWNED_TAG_PREFIXES = [
  "orders",
  "afterSales",
  "shipments",
  "fulfillments",
  "checkout",
  "recharges",
  "payments",
];

function isOrderOwnedTag(name) {
  return ORDER_OWNED_TAG_PREFIXES.some(
    (prefix) => name === prefix || name.startsWith(`${prefix}.`),
  );
}

for (const relativePath of targets) {
  const filePath = path.join(root, relativePath);
  if (!fs.existsSync(filePath)) {
    continue;
  }

  const doc = JSON.parse(fs.readFileSync(filePath, "utf8"));
  const schemas = doc.components?.schemas ?? {};
  const serialized = JSON.stringify(doc);

  const unreferenced = Object.keys(schemas).filter(
    (name) => !serialized.includes(`#/components/schemas/${name}`),
  );
  for (const name of unreferenced) {
    delete schemas[name];
  }

  if (Array.isArray(doc.tags)) {
    doc.tags = doc.tags.filter((tag) => isOrderOwnedTag(tag.name));
  }

  fs.writeFileSync(filePath, `${JSON.stringify(doc, null, 2)}\n`);
  console.log(
    `${relativePath}: removed ${unreferenced.length} unreferenced schemas`,
  );
}
