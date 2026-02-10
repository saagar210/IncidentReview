export type SanitizedExportResult = { export_dir: string; incident_count: number };

export type SanitizedManifest = {
  manifest_version: number;
  app_version: string;
  export_time: string;
  incident_count: number;
  files: Array<{ filename: string; bytes: number; sha256: string }>;
};

export type SanitizedImportSummary = {
  inserted_incidents: number;
  inserted_timeline_events: number;
  import_warnings: Array<{ code: string; message: string; details?: string | null }>;
};

export function SanitizedImportSection(props: {
  sanitizedExport: SanitizedExportResult | null;
  sanitizedImportDir: string;
  sanitizedImportManifest: SanitizedManifest | null;
  sanitizedImportSummary: SanitizedImportSummary | null;
  onExportSanitizedDataset: () => void | Promise<void>;
  onPickSanitizedDatasetForImport: () => void | Promise<void>;
  onImportSanitizedDataset: () => void | Promise<void>;
}) {
  return (
    <section className="card" id="sanitized">
      <h2>Sanitized Dataset Import/Export (Deterministic)</h2>
      <div className="actions">
        <button className="btn" type="button" onClick={() => void props.onExportSanitizedDataset()}>
          Export Sanitized Dataset...
        </button>
        <button className="btn" type="button" onClick={() => void props.onPickSanitizedDatasetForImport()}>
          Pick Sanitized Dataset...
        </button>
        <button
          className="btn btn--accent"
          type="button"
          onClick={() => void props.onImportSanitizedDataset()}
          disabled={!props.sanitizedImportManifest}
        >
          Import Sanitized Dataset
        </button>
      </div>
      <p className="hint">
        Sanitized export/redaction is deterministic. Import refuses to run on a non-empty DB (create/open a fresh workspace first).
      </p>

      {props.sanitizedExport && (
        <section className="card">
          <h2>Last Sanitized Export</h2>
          <p className="hint">
            Folder: <span className="mono">{props.sanitizedExport.export_dir}</span>
          </p>
          <p className="hint">
            Incidents: <span className="mono">{props.sanitizedExport.incident_count}</span>
          </p>
          <p className="hint">
            Free-text fields (Slack text, notes) are redacted; categories are pseudonymized deterministically for sharing.
          </p>
        </section>
      )}

      {props.sanitizedImportManifest && (
        <section className="card">
          <h2>Sanitized Import Preview</h2>
          <p className="hint">
            Selected: <span className="mono">{props.sanitizedImportDir}</span>
          </p>
          <ul className="list">
            <li>
              Export time: <span className="mono">{props.sanitizedImportManifest.export_time}</span>
            </li>
            <li>
              App version: <span className="mono">{props.sanitizedImportManifest.app_version}</span>
            </li>
            <li>
              Incidents: <span className="mono">{props.sanitizedImportManifest.incident_count}</span>
            </li>
            <li>
              Files:{" "}
              <span className="mono">
                {props.sanitizedImportManifest.files.map((f) => f.filename).sort().join(", ")}
              </span>
            </li>
          </ul>
          <p className="hint">
            On import, incident titles become <span className="mono">Incident INC_###</span> and timeline text becomes{" "}
            <span className="mono">[REDACTED]</span>.
          </p>
        </section>
      )}

      {props.sanitizedImportSummary && (
        <section className="card">
          <h2>Sanitized Import Result</h2>
          <ul className="list">
            <li>
              Inserted incidents: <span className="mono">{props.sanitizedImportSummary.inserted_incidents}</span>
            </li>
            <li>
              Inserted events: <span className="mono">{props.sanitizedImportSummary.inserted_timeline_events}</span>
            </li>
            <li>
              Import warnings: <span className="mono">{props.sanitizedImportSummary.import_warnings.length}</span>
            </li>
          </ul>
          {props.sanitizedImportSummary.import_warnings.length > 0 ? (
            <>
              <h3>Warnings</h3>
              <ul className="list">
                {props.sanitizedImportSummary.import_warnings.map((w, idx) => (
                  <li key={idx}>
                    <span className="mono">{w.code}</span>: {w.message}{" "}
                    {w.details ? <span className="mono">({w.details})</span> : null}
                  </li>
                ))}
              </ul>
            </>
          ) : (
            <p className="muted">No import warnings.</p>
          )}
        </section>
      )}
    </section>
  );
}

