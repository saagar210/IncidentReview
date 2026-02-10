import { describe, expect, it } from "vitest";
import { renderToString } from "react-dom/server";

import { BackupRestoreSection } from "./BackupRestoreSection";

describe("BackupRestoreSection", () => {
  it("renders minimal backup/restore UI", () => {
    const html = renderToString(
      <BackupRestoreSection
        backupResult={null}
        restoreBackupDir=""
        restoreManifest={null}
        restoreAllowOverwrite={false}
        setRestoreAllowOverwrite={() => {}}
        restoreResult={null}
        onBackupCreate={() => {}}
        onPickBackupForRestore={() => {}}
        onRestoreFromBackup={() => {}}
      />
    );

    expect(html).toContain("Backup / Restore");
    expect(html).toContain("Create Backup Folder");
  });
});

