export type WorkspaceInfo = {
  current_db_path: string;
  recent_db_paths: string[];
  load_error?: { code: string; message: string; details?: string | null; retryable: boolean } | null;
};

export type WorkspaceMetadata = { db_path: string; is_empty: boolean };

export function WorkspaceSection(props: {
  currentDbPathLabel: string;
  workspaceInfo: WorkspaceInfo | null;
  workspaceMeta: WorkspaceMetadata | null;
  workspaceNewFilename: string;
  onWorkspaceNewFilenameChange: (next: string) => void;
  workspaceRecentPick: string;
  onWorkspaceRecentPickChange: (next: string) => void;
  onOpenWorkspace: () => void | Promise<void>;
  onCreateWorkspace: () => void | Promise<void>;
  onSwitchToRecent: () => void | Promise<void>;
}) {
  return (
    <section className="card" id="workspace">
      <h2>Workspace (Create / Open / Switch)</h2>
      <p className="hint">
        A workspace is a local SQLite DB file. Switching workspaces reloads all data (incidents, dashboards, validation, report) from the selected DB.
      </p>
      <p className="hint">
        Current: <span className="mono">{props.currentDbPathLabel}</span>
      </p>
      {props.workspaceMeta ? (
        <p className="hint">
          Status: <span className="mono">{props.workspaceMeta.is_empty ? "empty" : "non-empty"}</span>
        </p>
      ) : null}

      <div className="actions">
        <button className="btn" type="button" onClick={() => void props.onOpenWorkspace()}>
          Open Workspace DB...
        </button>
        <input
          className="textInput"
          value={props.workspaceNewFilename}
          onChange={(e) => props.onWorkspaceNewFilenameChange(e.currentTarget.value)}
          placeholder="New DB filename (e.g. incidentreview.sqlite)"
        />
        <button className="btn btn--accent" type="button" onClick={() => void props.onCreateWorkspace()}>
          Create New Workspace...
        </button>
      </div>

      <div className="actions">
        <select
          className="select"
          value={props.workspaceRecentPick}
          onChange={(e) => props.onWorkspaceRecentPickChange(e.currentTarget.value)}
        >
          <option value="">(no recent workspaces)</option>
          {(props.workspaceInfo?.recent_db_paths ?? []).map((p) => (
            <option key={p} value={p}>
              {p}
            </option>
          ))}
        </select>
        <button
          className="btn"
          type="button"
          onClick={() => void props.onSwitchToRecent()}
          disabled={!props.workspaceRecentPick}
        >
          Switch To Selected
        </button>
      </div>
    </section>
  );
}

