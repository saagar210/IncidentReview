// @vitest-environment jsdom
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { BackupRestoreSection } from "./BackupRestoreSection";

function baseProps() {
  return {
    backupResult: null,
    restoreBackupDir: "",
    restoreManifest: null,
    restoreAllowOverwrite: false,
    setRestoreAllowOverwrite: vi.fn(),
    restoreResult: null,
    onBackupCreate: vi.fn(),
    onPickBackupForRestore: vi.fn(),
    onRestoreFromBackup: vi.fn(),
  } as const;
}

describe("BackupRestoreSection", () => {
  it("guards restore until manifest exists and invokes actions", () => {
    const props = baseProps();

    const { rerender } = render(<BackupRestoreSection {...props} />);

    expect(screen.getByRole("heading", { name: "Backup / Restore (Local-Only)" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Restore (Overwrite)" })).toBeDisabled();

    fireEvent.click(screen.getByRole("button", { name: "Create Backup Folder..." }));
    fireEvent.click(screen.getByRole("button", { name: "Pick Backup For Restore..." }));
    expect(props.onBackupCreate).toHaveBeenCalledTimes(1);
    expect(props.onPickBackupForRestore).toHaveBeenCalledTimes(1);

    rerender(
      <BackupRestoreSection
        {...props}
        backupResult={{
          backup_dir: "/tmp/backup-1",
          manifest: {
            manifest_version: 1,
            app_version: "0.1.0",
            export_time: "2026-02-17T00:00:00Z",
            schema_migrations: ["0001"],
            counts: { incidents: 3, timeline_events: 11, artifacts_rows: 2 },
            db: { filename: "incidentreview.sqlite", sha256: "abc", bytes: 10 },
            artifacts: { included: true, files: [] },
          },
        }}
        restoreBackupDir="/tmp/backup-1"
        restoreManifest={{
          manifest_version: 1,
          app_version: "0.1.0",
          export_time: "2026-02-17T00:00:00Z",
          schema_migrations: ["0001"],
          counts: { incidents: 3, timeline_events: 11, artifacts_rows: 2 },
          db: { filename: "incidentreview.sqlite", sha256: "abc", bytes: 10 },
          artifacts: { included: true, files: [] },
        }}
      />
    );

    expect(screen.getAllByText("/tmp/backup-1")).toHaveLength(2);
    expect(screen.getByRole("button", { name: "Restore (Overwrite)" })).toBeEnabled();

    fireEvent.click(screen.getByRole("checkbox"));
    expect(props.setRestoreAllowOverwrite).toHaveBeenCalledWith(true);

    fireEvent.click(screen.getByRole("button", { name: "Restore (Overwrite)" }));
    expect(props.onRestoreFromBackup).toHaveBeenCalledTimes(1);
  });
});
