export type IncidentOption = { id: number; external_id: string | null; title: string };

export type SlackPreview = {
  detected_format: string;
  line_count: number;
  message_count: number;
  warnings: Array<{ code: string; message: string; details?: string | null }>;
};

export type SlackIngestSummary = {
  incident_id: number;
  incident_created: boolean;
  detected_format: string;
  inserted_events: number;
  warnings: Array<{ code: string; message: string; details?: string | null }>;
};

export function SlackImportSection(props: {
  incidentOptions: IncidentOption[];
  slackTargetMode: "existing" | "new";
  setSlackTargetMode: (mode: "existing" | "new") => void;
  slackExistingIncidentId: number | null;
  setSlackExistingIncidentId: (id: number | null) => void;
  slackNewIncidentTitle: string;
  setSlackNewIncidentTitle: (title: string) => void;
  slackFileName: string;
  slackText: string;
  setSlackText: (text: string) => void;
  setSlackPreview: (prev: SlackPreview | null) => void;
  setSlackSummary: (sum: SlackIngestSummary | null) => void;
  slackPreview: SlackPreview | null;
  slackSummary: SlackIngestSummary | null;
  onRefreshIncidentsList: () => void | Promise<void>;
  onSlackPickFile: (file: File | null) => void | Promise<void>;
  onSlackPreview: () => void | Promise<void>;
  onSlackIngest: () => void | Promise<void>;
}) {
  return (
    <section className="card" id="slack">
      <h2>Slack Import (Transcript)</h2>
      <div className="actions">
        <button className="btn" type="button" onClick={() => void props.onRefreshIncidentsList()}>
          Refresh incidents
        </button>
        <label className="btn">
          Choose Slack file
          <input
            className="fileInput"
            type="file"
            accept=".txt,.json,application/json,text/plain"
            onChange={(e) => void props.onSlackPickFile(e.currentTarget.files?.[0] ?? null)}
          />
        </label>
        <button className="btn" type="button" onClick={() => void props.onSlackPreview()}>
          Preview
        </button>
        <button className="btn btn--accent" type="button" onClick={() => void props.onSlackIngest()}>
          Ingest
        </button>
      </div>

      <p className="hint">
        Selected file: <span className="mono">{props.slackFileName || "none"}</span>
      </p>

      <div className="twoCol">
        <section className="card">
          <h2>Attach To</h2>
          <div className="actions">
            <label className="radio">
              <input
                type="radio"
                name="slackTarget"
                checked={props.slackTargetMode === "existing"}
                onChange={() => props.setSlackTargetMode("existing")}
              />
              Existing incident
            </label>
            <label className="radio">
              <input
                type="radio"
                name="slackTarget"
                checked={props.slackTargetMode === "new"}
                onChange={() => props.setSlackTargetMode("new")}
              />
              New Slack-only incident shell
            </label>
          </div>

          {props.slackTargetMode === "existing" ? (
            <>
              <p className="hint">Choose an incident to attach events to.</p>
              <select
                className="select"
                value={props.slackExistingIncidentId ?? ""}
                onChange={(e) => props.setSlackExistingIncidentId(e.currentTarget.value ? Number(e.currentTarget.value) : null)}
              >
                <option value="">(select an incident)</option>
                {props.incidentOptions.map((i) => (
                  <option key={i.id} value={i.id}>
                    {(i.external_id ?? `id=${i.id}`) + " — " + i.title}
                  </option>
                ))}
              </select>
            </>
          ) : (
            <>
              <p className="hint">Title is required (no silent defaults).</p>
              <input
                className="textInput"
                placeholder="New incident title"
                value={props.slackNewIncidentTitle}
                onChange={(e) => props.setSlackNewIncidentTitle(e.currentTarget.value)}
              />
            </>
          )}
        </section>

        <section className="card">
          <h2>Transcript</h2>
          <textarea
            className="md"
            value={props.slackText}
            placeholder="Paste transcript text here, or choose a file above."
            onChange={(e) => {
              props.setSlackText(e.currentTarget.value);
              props.setSlackPreview(null);
              props.setSlackSummary(null);
            }}
          />
          {props.slackPreview ? (
            <>
              <h3 className="subhead">Preview</h3>
              <p className="hint">
                detected_format=<span className="mono">{props.slackPreview.detected_format}</span>, lines=
                <span className="mono">{props.slackPreview.line_count}</span>, messages=
                <span className="mono">{props.slackPreview.message_count}</span>
              </p>
              {props.slackPreview.warnings.length > 0 && (
                <ul className="list">
                  {props.slackPreview.warnings.map((w, idx) => (
                    <li key={idx}>
                      <span className="mono">{w.code}</span>: {w.message}{" "}
                      {w.details ? <span className="mono">({w.details})</span> : null}
                    </li>
                  ))}
                </ul>
              )}
            </>
          ) : (
            <p className="muted">Preview shows detected format and warnings (no timestamp guessing).</p>
          )}
        </section>
      </div>

      {props.slackSummary && (
        <section className="card">
          <h2>Slack Ingest Result</h2>
          <div className="kpiRow">
            <div className="kpi">
              <div className="kpi__label">Incident ID</div>
              <div className="kpi__value mono">{props.slackSummary.incident_id}</div>
            </div>
            <div className="kpi">
              <div className="kpi__label">Created</div>
              <div className="kpi__value">{props.slackSummary.incident_created ? "yes" : "no"}</div>
            </div>
            <div className="kpi">
              <div className="kpi__label">Format</div>
              <div className="kpi__value mono">{props.slackSummary.detected_format}</div>
            </div>
            <div className="kpi">
              <div className="kpi__label">Events</div>
              <div className="kpi__value">{props.slackSummary.inserted_events}</div>
            </div>
            <div className="kpi">
              <div className="kpi__label">Warnings</div>
              <div className="kpi__value">{props.slackSummary.warnings.length}</div>
            </div>
          </div>

          {props.slackSummary.warnings.length > 0 && (
            <>
              <h3 className="subhead">Warnings</h3>
              <ul className="list">
                {props.slackSummary.warnings.map((w, idx) => (
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

