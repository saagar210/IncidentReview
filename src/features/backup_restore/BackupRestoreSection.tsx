export type BackupManifest = {
  manifest_version: number;
  app_version: string;
  export_time: string;
  schema_migrations: string[];
  counts: { incidents: number; timeline_events: number; artifacts_rows: number };
  db: { filename: string; sha256: string; bytes: number };
  artifacts: { included: boolean; files: Array<{ rel_path: string; sha256: string; bytes: number }> };
};

export type BackupCreateResult = {
  backup_dir: string;
  manifest: BackupManifest;
};

export type RestoreResult = { ok: boolean; restored_db_path: string; restored_artifacts: boolean };

export function BackupRestoreSection(props: {
  backupResult: BackupCreateResult | null;
  restoreBackupDir: string;
  restoreManifest: BackupManifest | null;
  restoreAllowOverwrite: boolean;
  setRestoreAllowOverwrite: (next: boolean) => void;
  restoreResult: RestoreResult | null;
  onBackupCreate: () => void | Promise<void>;
  onPickBackupForRestore: () => void | Promise<void>;
  onRestoreFromBackup: () => void | Promise<void>;
}) {
  return (
    <section className="card" id="data">
      <h2>Backup / Restore (Local-Only)</h2>
      <div className="actions">
        <button className="btn" type="button" onClick={() => void props.onBackupCreate()}>
          Create Backup Folder...
        </button>
        <button className="btn" type="button" onClick={() => void props.onPickBackupForRestore()}>
          Pick Backup For Restore...
        </button>
        <button className="btn btn--accent" type="button" onClick={() => void props.onRestoreFromBackup()} disabled={!props.restoreManifest}>
          Restore (Overwrite)
        </button>
      </div>
      <p className="hint">
        Backups are exported as folders containing <span className="mono">incidentreview.sqlite</span> and{" "}
        <span className="mono">manifest.json</span> (no zip by default). Restore requires explicit overwrite confirmation and validates DB hashes from the manifest.
      </p>

      {props.backupResult && (
        <section className="card">
          <h2>Last Backup</h2>
          <p className="hint">
            Folder: <span className="mono">{props.backupResult.backup_dir}</span>
          </p>
          <div className="kpiRow">
            <div className="kpi">
              <div className="kpi__label">Incidents</div>
              <div className="kpi__value">{props.backupResult.manifest.counts.incidents}</div>
            </div>
            <div className="kpi">
              <div className="kpi__label">Timeline events</div>
              <div className="kpi__value">{props.backupResult.manifest.counts.timeline_events}</div>
            </div>
            <div className="kpi">
              <div className="kpi__label">Artifacts rows</div>
              <div className="kpi__value">{props.backupResult.manifest.counts.artifacts_rows}</div>
            </div>
          </div>
        </section>
      )}

      {props.restoreManifest && (
        <section className="card">
          <h2>Restore Preview</h2>
          <p className="hint">
            Selected: <span className="mono">{props.restoreBackupDir}</span>
          </p>
          <ul className="list">
            <li>
              Export time: <span className="mono">{props.restoreManifest.export_time}</span>
            </li>
            <li>
              App version: <span className="mono">{props.restoreManifest.app_version}</span>
            </li>
            <li>
              Incidents: <span className="mono">{props.restoreManifest.counts.incidents}</span>
            </li>
            <li>
              Timeline events: <span className="mono">{props.restoreManifest.counts.timeline_events}</span>
            </li>
            <li>
              Artifacts included: <span className="mono">{props.restoreManifest.artifacts.included ? "yes" : "no"}</span>
            </li>
          </ul>

          <label className="checkbox">
            <input
              type="checkbox"
              checked={props.restoreAllowOverwrite}
              onChange={(e) => props.setRestoreAllowOverwrite(e.currentTarget.checked)}
            />
            I understand this will overwrite my local database.
          </label>

          {props.restoreResult && (
            <p className="hint">
              Restore result: <span className="mono">{props.restoreResult.ok ? "ok" : "failed"}</span> (db:{" "}
              <span className="mono">{props.restoreResult.restored_db_path}</span>)
            </p>
          )}
        </section>
      )}
    </section>
  );
}

