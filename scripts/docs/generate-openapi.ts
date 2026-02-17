import { mkdirSync, writeFileSync } from "node:fs";

import { OpenAPIRegistry, OpenApiGeneratorV31, extendZodWithOpenApi } from "@asteasolutions/zod-to-openapi";
import { z } from "zod";

import {
  AppErrorSchema,
  DashboardPayloadV2Schema,
  InitDbResponseSchema,
  ValidationReportSchema,
  WorkspaceMigrationStatusSchema,
} from "../../src/lib/schemas";

extendZodWithOpenApi(z);

const registry = new OpenAPIRegistry();

registry.registerPath({
  method: "post",
  path: "/rpc/init_db",
  description: "Initialize or open a workspace database through the Tauri command layer.",
  request: {
    body: {
      content: {
        "application/json": {
          schema: z.object({ dbPath: z.string().min(1).nullable().optional() }),
        },
      },
    },
  },
  responses: {
    200: {
      description: "Workspace initialized.",
      content: {
        "application/json": {
          schema: InitDbResponseSchema,
        },
      },
    },
    400: {
      description: "AppError response.",
      content: {
        "application/json": {
          schema: AppErrorSchema,
        },
      },
    },
  },
});

registry.registerPath({
  method: "post",
  path: "/rpc/load_dashboard_v2",
  description: "Load deterministic dashboard analytics payload.",
  responses: {
    200: {
      description: "Dashboard payload V2.",
      content: {
        "application/json": {
          schema: DashboardPayloadV2Schema,
        },
      },
    },
    400: {
      description: "AppError response.",
      content: {
        "application/json": {
          schema: AppErrorSchema,
        },
      },
    },
  },
});

registry.registerPath({
  method: "post",
  path: "/rpc/run_validators",
  description: "Run validator engine and return anomaly report.",
  responses: {
    200: {
      description: "Validation report.",
      content: {
        "application/json": {
          schema: ValidationReportSchema,
        },
      },
    },
    400: {
      description: "AppError response.",
      content: {
        "application/json": {
          schema: AppErrorSchema,
        },
      },
    },
  },
});

registry.registerPath({
  method: "post",
  path: "/rpc/check_workspace_migration_status",
  description: "Return migration drift status for selected workspace DB.",
  request: {
    body: {
      content: {
        "application/json": {
          schema: z.object({ dbPath: z.string().min(1) }),
        },
      },
    },
  },
  responses: {
    200: {
      description: "Workspace migration status.",
      content: {
        "application/json": {
          schema: WorkspaceMigrationStatusSchema,
        },
      },
    },
    400: {
      description: "AppError response.",
      content: {
        "application/json": {
          schema: AppErrorSchema,
        },
      },
    },
  },
});

const generator = new OpenApiGeneratorV31(registry.definitions);
const document = generator.generateDocument({
  openapi: "3.1.0",
  info: {
    title: "IncidentReview Command Contract",
    version: "1.0.0",
    description: "Generated command contract docs for Tauri RPC handlers.",
  },
});

mkdirSync("openapi", { recursive: true });
writeFileSync("openapi/openapi.generated.json", JSON.stringify(document, null, 2));
