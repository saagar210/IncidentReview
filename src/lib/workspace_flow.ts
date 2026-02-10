import type { ZodType } from "zod";

import { invokeValidated } from "./tauri";
import {
  DashboardPayloadV2Schema,
  IncidentListSchema,
  ValidationReportSchema,
  WorkspaceMetadataSchema,
} from "./schemas";

type Invoker = <T>(command: string, args: Record<string, unknown> | undefined, schema: ZodType<T> | null) => Promise<T>;

export type WorkspaceLoadSnapshot = {
  incidents: unknown;
  dashboard: unknown;
  validation: unknown;
  report_md: string;
};

export async function workspaceOpenAndLoadAll(dbPath: string, invoker: Invoker = invokeValidated): Promise<WorkspaceLoadSnapshot> {
  await invoker("workspace_open", { dbPath }, WorkspaceMetadataSchema);

  const incidents = await invoker("incidents_list", undefined, IncidentListSchema);
  const dashboard = await invoker("get_dashboard_v2", undefined, DashboardPayloadV2Schema);
  const validation = await invoker("validation_report", undefined, ValidationReportSchema);
  const report_md = await invoker<string>("generate_report_md", undefined, null);

  return { incidents, dashboard, validation, report_md };
}

