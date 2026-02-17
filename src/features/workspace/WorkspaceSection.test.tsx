// @vitest-environment jsdom
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { WorkspaceSection } from "./WorkspaceSection";

describe("WorkspaceSection", () => {
  it("enforces recent workspace switch state and wires actions", () => {
    const onWorkspaceNewFilenameChange = vi.fn();
    const onWorkspaceRecentPickChange = vi.fn();
    const onOpenWorkspace = vi.fn();
    const onCreateWorkspace = vi.fn();
    const onSwitchToRecent = vi.fn();

    const { rerender } = render(
      <WorkspaceSection
        currentDbPathLabel="/tmp/demo.sqlite"
        workspaceInfo={{ current_db_path: "/tmp/demo.sqlite", recent_db_paths: ["/tmp/demo.sqlite", "/tmp/other.sqlite"] }}
        workspaceMeta={{ db_path: "/tmp/demo.sqlite", is_empty: true }}
        workspaceNewFilename="incidentreview.sqlite"
        onWorkspaceNewFilenameChange={onWorkspaceNewFilenameChange}
        workspaceRecentPick=""
        onWorkspaceRecentPickChange={onWorkspaceRecentPickChange}
        onOpenWorkspace={onOpenWorkspace}
        onCreateWorkspace={onCreateWorkspace}
        onSwitchToRecent={onSwitchToRecent}
      />
    );

    expect(screen.getByText("Workspace (Create / Open / Switch)")).toBeInTheDocument();
    expect(screen.getByText("empty")).toBeInTheDocument();

    const switchButton = screen.getByRole("button", { name: "Switch To Selected" });
    expect(switchButton).toBeDisabled();

    fireEvent.change(screen.getByPlaceholderText("New DB filename (e.g. incidentreview.sqlite)"), {
      target: { value: "q2.sqlite" },
    });
    expect(onWorkspaceNewFilenameChange).toHaveBeenCalledWith("q2.sqlite");

    fireEvent.change(screen.getByLabelText("Recent workspaces"), { target: { value: "/tmp/other.sqlite" } });
    expect(onWorkspaceRecentPickChange).toHaveBeenCalledWith("/tmp/other.sqlite");

    fireEvent.click(screen.getByRole("button", { name: "Open Workspace DB..." }));
    fireEvent.click(screen.getByRole("button", { name: "Create New Workspace..." }));
    expect(onOpenWorkspace).toHaveBeenCalledTimes(1);
    expect(onCreateWorkspace).toHaveBeenCalledTimes(1);

    rerender(
      <WorkspaceSection
        currentDbPathLabel="/tmp/demo.sqlite"
        workspaceInfo={{ current_db_path: "/tmp/demo.sqlite", recent_db_paths: ["/tmp/demo.sqlite", "/tmp/other.sqlite"] }}
        workspaceMeta={{ db_path: "/tmp/demo.sqlite", is_empty: false }}
        workspaceNewFilename="incidentreview.sqlite"
        onWorkspaceNewFilenameChange={onWorkspaceNewFilenameChange}
        workspaceRecentPick="/tmp/demo.sqlite"
        onWorkspaceRecentPickChange={onWorkspaceRecentPickChange}
        onOpenWorkspace={onOpenWorkspace}
        onCreateWorkspace={onCreateWorkspace}
        onSwitchToRecent={onSwitchToRecent}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: "Switch To Selected" }));
    expect(onSwitchToRecent).toHaveBeenCalledTimes(1);
  });
});
