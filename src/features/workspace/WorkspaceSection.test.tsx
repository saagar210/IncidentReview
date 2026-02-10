import { describe, expect, it } from "vitest";
import { renderToString } from "react-dom/server";

import { WorkspaceSection } from "./WorkspaceSection";

describe("WorkspaceSection", () => {
  it("renders minimal workspace controls", () => {
    const html = renderToString(
      <WorkspaceSection
        currentDbPathLabel="/tmp/demo.sqlite"
        workspaceInfo={{ current_db_path: "/tmp/demo.sqlite", recent_db_paths: ["/tmp/demo.sqlite"] }}
        workspaceMeta={{ db_path: "/tmp/demo.sqlite", is_empty: true }}
        workspaceNewFilename="incidentreview.sqlite"
        onWorkspaceNewFilenameChange={() => {}}
        workspaceRecentPick="/tmp/demo.sqlite"
        onWorkspaceRecentPickChange={() => {}}
        onOpenWorkspace={() => {}}
        onCreateWorkspace={() => {}}
        onSwitchToRecent={() => {}}
      />
    );

    expect(html).toContain("Workspace");
    expect(html).toContain("Open Workspace DB");
    expect(html).toContain("Create New Workspace");
  });
});

