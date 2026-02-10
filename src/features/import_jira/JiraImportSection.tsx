export type JiraProfile = {
  id: number;
  name: string;
  mapping: {
    external_id?: string | null;
    title: string;
    description?: string | null;
    severity?: string | null;
    detection_source?: string | null;
    vendor?: string | null;
    service?: string | null;
    impact_pct?: string | null;
    service_health_pct?: string | null;
    start_ts?: string | null;
    first_observed_ts?: string | null;
    it_awareness_ts?: string | null;
    ack_ts?: string | null;
    mitigate_ts?: string | null;
    resolve_ts?: string | null;
  };
};

export type CsvPreview = { headers: string[]; rows: string[][] };

export type JiraMappingState = {
  external_id: string | null;
  title: string;
  description: string | null;
  severity: string | null;
  detection_source: string | null;
  vendor: string | null;
  service: string | null;
  impact_pct: string | null;
  service_health_pct: string | null;
  start_ts: string | null;
  first_observed_ts: string | null;
  it_awareness_ts: string | null;
  ack_ts: string | null;
  mitigate_ts: string | null;
  resolve_ts: string | null;
};

export type JiraImportSummary = {
  inserted: number;
  updated: number;
  skipped: number;
  conflicts: Array<{ row: number; reason: string; external_id?: string | null; fingerprint?: string | null }>;
  warnings: Array<{ code: string; message: string; details?: string | null }>;
};

export function JiraImportSection(props: {
  jiraProfiles: JiraProfile[];
  selectedProfileId: number | null;
  setSelectedProfileId: (id: number | null) => void;
  profileName: string;
  setProfileName: (name: string) => void;
  csvFileName: string;
  csvPreview: CsvPreview | null;
  mapping: JiraMappingState;
  setMapping: (next: JiraMappingState) => void;
  importSummary: JiraImportSummary | null;
  onRefreshProfiles: () => void | Promise<void>;
  onPickCsvFile: (file: File | null) => void | Promise<void>;
  applyCommonJiraDefaults: () => void;
  onImportCsv: () => void | Promise<void>;
  onSaveProfile: () => void | Promise<void>;
  onDeleteProfile: () => void | Promise<void>;
}) {
  const csvPreview = props.csvPreview;

  return (
    <section className="card" id="jira">
      <h2>Jira CSV Import (Mapping Profiles)</h2>
      <div className="actions">
        <button className="btn" type="button" onClick={() => void props.onRefreshProfiles()}>
          Refresh profiles
        </button>
        <label className="btn">
          Choose CSV
          <input
            className="fileInput"
            type="file"
            accept=".csv,text/csv"
            onChange={(e) => void props.onPickCsvFile(e.currentTarget.files?.[0] ?? null)}
          />
        </label>
        <button className="btn" type="button" onClick={props.applyCommonJiraDefaults} disabled={!csvPreview}>
          Apply common Jira defaults
        </button>
        <button className="btn btn--accent" type="button" onClick={() => void props.onImportCsv()}>
          Import CSV
        </button>
      </div>
      <p className="hint">
        Selected file: <span className="mono">{props.csvFileName || "none"}</span>
      </p>

      <div className="twoCol">
        <section className="card">
          <h2>Profiles</h2>
          <div className="actions">
            <select
              className="select"
              value={props.selectedProfileId ?? ""}
              onChange={(e) => {
                const v = e.currentTarget.value;
                const id = v ? Number(v) : null;
                props.setSelectedProfileId(id);
                const prof = props.jiraProfiles.find((p) => p.id === id);
                if (prof) {
                  props.setProfileName(prof.name);
                  props.setMapping({
                    external_id: prof.mapping.external_id ?? null,
                    title: prof.mapping.title ?? "",
                    description: prof.mapping.description ?? null,
                    severity: prof.mapping.severity ?? null,
                    detection_source: prof.mapping.detection_source ?? null,
                    vendor: prof.mapping.vendor ?? null,
                    service: prof.mapping.service ?? null,
                    impact_pct: prof.mapping.impact_pct ?? null,
                    service_health_pct: prof.mapping.service_health_pct ?? null,
                    start_ts: prof.mapping.start_ts ?? null,
                    first_observed_ts: prof.mapping.first_observed_ts ?? null,
                    it_awareness_ts: prof.mapping.it_awareness_ts ?? null,
                    ack_ts: prof.mapping.ack_ts ?? null,
                    mitigate_ts: prof.mapping.mitigate_ts ?? null,
                    resolve_ts: prof.mapping.resolve_ts ?? null,
                  });
                }
              }}
            >
              <option value="">(no profile selected)</option>
              {props.jiraProfiles.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name} (id={p.id})
                </option>
              ))}
            </select>
            <input
              className="textInput"
              placeholder="Profile name"
              value={props.profileName}
              onChange={(e) => props.setProfileName(e.currentTarget.value)}
            />
            <button className="btn" type="button" onClick={() => void props.onSaveProfile()}>
              Save profile
            </button>
            <button className="btn" type="button" onClick={() => void props.onDeleteProfile()} disabled={!props.selectedProfileId}>
              Delete
            </button>
          </div>
          <p className="hint">Profiles are stored locally in SQLite via qir_core.</p>
        </section>

        <section className="card">
          <h2>CSV Preview</h2>
          {!csvPreview ? (
            <p className="muted">Choose a CSV to preview headers and sample rows.</p>
          ) : (
            <>
              <p className="hint">
                Columns: <span className="mono">{csvPreview.headers.join(", ")}</span>
              </p>
              <div className="tableWrap">
                <table className="table">
                  <thead>
                    <tr>
                      {csvPreview.headers.map((h) => (
                        <th key={h}>{h}</th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {csvPreview.rows.map((r, idx) => (
                      <tr key={idx}>
                        {r.map((c, i) => (
                          <td key={i} className="mono">
                            {c}
                          </td>
                        ))}
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </>
          )}
        </section>
      </div>

      <section className="card">
        <h2>Mapping</h2>
        {!csvPreview ? (
          <p className="muted">Load a CSV first.</p>
        ) : (
          <div className="mapping">
            {(
              [
                { key: "external_id", label: "External ID (optional)", required: false },
                { key: "title", label: "Title (required)", required: true },
                { key: "description", label: "Description (optional)", required: false },
                { key: "severity", label: "Severity (optional)", required: false },
                { key: "detection_source", label: "Detection Source (optional)", required: false },
                { key: "vendor", label: "Vendor (optional)", required: false },
                { key: "service", label: "Service (optional)", required: false },
                { key: "impact_pct", label: "Impact % (optional)", required: false },
                { key: "service_health_pct", label: "Service Health % (optional)", required: false },
                { key: "start_ts", label: "Start TS (optional)", required: false },
                { key: "first_observed_ts", label: "First Observed TS (optional)", required: false },
                { key: "it_awareness_ts", label: "IT Awareness TS (optional)", required: false },
                { key: "ack_ts", label: "Ack TS (optional)", required: false },
                { key: "mitigate_ts", label: "Mitigate TS (optional)", required: false },
                { key: "resolve_ts", label: "Resolve TS (optional)", required: false },
              ] as const
            ).map((f) => (
              <div key={f.key} className="mappingRow">
                <div className="mappingRow__label">{f.label}</div>
                <select
                  className="select"
                  value={(props.mapping as never)[f.key] ?? (f.required ? "" : "")}
                  onChange={(e) => {
                    const v = e.currentTarget.value;
                    props.setMapping({
                      ...props.mapping,
                      [f.key]: v === "" ? (f.required ? "" : null) : v,
                    } as never);
                  }}
                >
                  <option value="">{f.required ? "(select a column)" : "(none)"}</option>
                  {csvPreview.headers.map((h) => (
                    <option key={h} value={h}>
                      {h}
                    </option>
                  ))}
                </select>
              </div>
            ))}
          </div>
        )}
        <p className="hint">
          Timestamp normalization is deterministic: canonical incident timestamps are stored as RFC3339 UTC; non-RFC3339 inputs are preserved as raw strings and surfaced as warnings (no fuzzy parsing, no guessing).
        </p>
      </section>

      {props.importSummary && (
        <section className="card">
          <h2>Import Result</h2>
          <div className="kpiRow">
            <div className="kpi">
              <div className="kpi__label">Inserted</div>
              <div className="kpi__value">{props.importSummary.inserted}</div>
            </div>
            <div className="kpi">
              <div className="kpi__label">Updated</div>
              <div className="kpi__value">{props.importSummary.updated}</div>
            </div>
            <div className="kpi">
              <div className="kpi__label">Skipped</div>
              <div className="kpi__value">{props.importSummary.skipped}</div>
            </div>
            <div className="kpi">
              <div className="kpi__label">Conflicts</div>
              <div className="kpi__value">{props.importSummary.conflicts.length}</div>
            </div>
            <div className="kpi">
              <div className="kpi__label">Warnings</div>
              <div className="kpi__value">{props.importSummary.warnings.length}</div>
            </div>
          </div>

          {props.importSummary.conflicts.length > 0 && (
            <>
              <h3 className="subhead">Conflicts</h3>
              <ul className="list">
                {props.importSummary.conflicts.map((c, idx) => (
                  <li key={idx} className="mono">
                    row={c.row}: {c.reason}
                  </li>
                ))}
              </ul>
            </>
          )}

          {props.importSummary.warnings.length > 0 && (
            <>
              <h3 className="subhead">Warnings</h3>
              <ul className="list">
                {props.importSummary.warnings.map((w, idx) => (
                  <li key={idx}>
                    <span className="mono">{w.code}</span>: {w.message}{" "}
                    {w.details ? <span className="mono">({w.details})</span> : null}
                  </li>
                ))}
              </ul>
            </>
          )}
        </section>
      )}
    </section>
  );
}
