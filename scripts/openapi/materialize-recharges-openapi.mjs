#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const orderRoot = path.resolve(__dirname, "../..");

const authorityPath = path.join(
  orderRoot,
  "apis/app-api/order/order-app-api.openapi.json",
);
const sdkOpenApiPath = path.join(
  orderRoot,
  "sdks/sdkwork-order-app-sdk/openapi/sdkwork-order-app-api.openapi.json",
);
const sdkGenPath = path.join(
  orderRoot,
  "sdks/sdkwork-order-app-sdk/openapi/sdkwork-order-app-api.sdkgen.json",
);

const problemResponses = {
  400: problemResponse("Bad request"),
  401: problemResponse("Unauthorized"),
  403: problemResponse("Forbidden"),
  404: problemResponse("Not found"),
  409: problemResponse("Conflict"),
  500: problemResponse("Internal server error"),
};

const security = [{ AuthToken: [], AccessToken: [] }];

const offsetPaginationParameters = [
  {
    name: "page",
    in: "query",
    required: false,
    schema: { type: "integer", minimum: 1, default: 1 },
  },
  {
    name: "page_size",
    in: "query",
    required: false,
    schema: { type: "integer", minimum: 1, maximum: 200, default: 20 },
  },
];

const extension = (resource) => ({
  "x-sdkwork-owner": "sdkwork-order",
  "x-sdkwork-api-authority": "sdkwork-order-app-api",
  "x-sdkwork-domain": "commerce",
  "x-sdkwork-resource": resource,
  "x-sdkwork-request-context": "WebRequestContext",
  "x-sdkwork-api-surface": "app-api",
});

function problemResponse(description) {
  return {
    description,
    content: {
      "application/problem+json": {
        schema: { $ref: "#/components/schemas/ProblemDetail" },
      },
    },
  };
}

function envelopeResponse(schemaRef) {
  return {
    200: {
      description: "Success",
      content: {
        "application/json": {
          schema: { $ref: schemaRef },
        },
      },
    },
    ...problemResponses,
  };
}

function listResponses() {
  return envelopeResponse("#/components/schemas/SdkWorkListResponse");
}

function resourceResponses() {
  return envelopeResponse("#/components/schemas/SdkWorkResourceResponse");
}

function commandResponses() {
  return envelopeResponse("#/components/schemas/SdkWorkCommandResponse");
}

function getOperation(summary, operationId, resource, extra = {}, envelope = "resource") {
  const responses =
    envelope === "list"
      ? listResponses()
      : envelope === "command"
        ? commandResponses()
        : resourceResponses();

  return {
    tags: ["recharges"],
    summary,
    operationId,
    responses,
    security,
    ...extension(resource),
    ...extra,
  };
}

const rechargePaths = {
  "/app/v3/api/recharges/packages": {
    get: getOperation(
      "Recharges packages list.",
      "recharges.packages.list",
      "recharges.packages",
      { parameters: offsetPaginationParameters },
      "list",
    ),
  },
  "/app/v3/api/recharges/settings": {
    get: getOperation(
      "Recharges settings retrieve.",
      "recharges.settings.retrieve",
      "recharges.settings",
      { parameters: [] },
      "resource",
    ),
  },
  "/app/v3/api/recharges/orders": {
    get: getOperation(
      "Recharges orders list.",
      "recharges.orders.list",
      "recharges.orders",
      {
        parameters: [
          {
            name: "status",
            in: "query",
            required: false,
            schema: { type: "string" },
          },
          ...offsetPaginationParameters,
        ],
      },
      "list",
    ),
    post: getOperation(
      "Recharges orders create.",
      "recharges.orders.create",
      "recharges.orders",
      {
        parameters: [],
        requestBody: {
          required: true,
          content: {
            "application/json": {
              schema: { $ref: "#/components/schemas/RechargeOrderCreateCommand" },
            },
          },
        },
      },
      "resource",
    ),
  },
  "/app/v3/api/recharges/orders/{orderId}": {
    get: getOperation(
      "Recharges orders retrieve.",
      "recharges.orders.retrieve",
      "recharges.orders",
      {
        parameters: [
          {
            name: "orderId",
            in: "path",
            required: true,
            schema: { type: "string" },
          },
        ],
      },
      "resource",
    ),
  },
  "/app/v3/api/recharges/orders/{orderId}/cancel": {
    post: getOperation(
      "Recharges orders cancel.",
      "recharges.orders.cancel",
      "recharges.orders",
      {
        parameters: [
          {
            name: "orderId",
            in: "path",
            required: true,
            schema: { type: "string" },
          },
        ],
        requestBody: {
          required: false,
          content: {
            "application/json": {
              schema: { $ref: "#/components/schemas/CommerceOperationCommand" },
            },
          },
        },
      },
      "command",
    ),
  },
};

function patchSpec(spec) {
  const next = structuredClone(spec);
  next.paths = {
    ...next.paths,
    ...rechargePaths,
  };

  next.components ??= {};
  next.components.schemas ??= {};
  next.components.schemas.RechargeOrderCreateCommand = {
    type: "object",
    additionalProperties: false,
    properties: {
      amount: {
        oneOf: [{ type: "string" }, { type: "number" }],
      },
      clientRequestNo: { type: "string" },
      currencyCode: { type: "string" },
      packageId: { type: "string" },
      source: { type: "string" },
      paymentMethod: { type: "string" },
      paymentPassword: { type: "string" },
    },
  };

  return next;
}

function writeJson(targetPath, value) {
  fs.writeFileSync(targetPath, `${JSON.stringify(value, null, 2)}\n`, "utf8");
}

const authority = JSON.parse(fs.readFileSync(authorityPath, "utf8"));
const patched = patchSpec(authority);

writeJson(authorityPath, patched);
writeJson(sdkOpenApiPath, patched);
writeJson(sdkGenPath, patched);

console.log(`Materialized ${Object.keys(rechargePaths).length} recharge paths into order app OpenAPI.`);
