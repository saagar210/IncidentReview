import { describe, expect, it } from "vitest";

import { workspaceOpenAndLoadAll } from "./workspace_flow";

describe("workspaceOpenAndLoadAll", () => {
  it("opens workspace and reloads all view data from backend commands", async () => {
    const calls: Array<{ cmd: string; args?: Record<string, unknown> }> = [];

    const invoker = async (command: string, args: Record<string, unknown> | undefined) => {
      calls.push({ cmd: command, args });
      switch (command) {
        case "workspace_open":
          return { db_path: "/tmp/x.sqlite", is_empty: true };
        case "incidents_list":
          return [];
        case "get_dashboard_v2":
          return { version: 2, incident_count: 0, severity_counts: [], incidents: [], detection_story: { detection_source_mix: [], it_awareness_lag_buckets: [] }, vendor_service_story: { top_vendors_by_count: [], top_services_by_count: [], top_vendors_by_pain: [], top_services_by_pain: [] }, response_story: { time_to_mitigation_buckets: [], time_to_resolve_buckets: [] } };
        case "validation_report":
          return [];
        case "generate_report_md":
          return "# Quarterly Incident Review (QIR)\n";
        default:
          throw new Error(`unexpected command ${command}`);
      }
    };

    const snap = await workspaceOpenAndLoadAll("/tmp/x.sqlite", invoker as never);
    expect(snap.report_md).toContain("# Quarterly Incident Review");

    const cmds = calls.map((c) => c.cmd);
    expect(cmds).toEqual([
      "workspace_open",
      "incidents_list",
      "get_dashboard_v2",
      "validation_report",
      "generate_report_md",
    ]);
  });
});
