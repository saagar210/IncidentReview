export type ValidationReportItem = {
  id: number;
  external_id: string | null;
  title: string;
  warnings: Array<{ code: string; message: string; details?: string | null }>;
};

export function ValidationSection(props: {
  validationReport: ValidationReportItem[] | null;
  dashboardLoaded: boolean;
  hasIncidentFilter: boolean;
  onRefreshValidation: () => void | Promise<void>;
  onRefreshIncidents: () => void | Promise<void>;
  onClearIncidentFilter: () => void;
  onFilterIncidentFromValidation: (incidentId: number, label: string) => void;
}) {
  return (
    <section className="card" id="validation">
      <h2>Validation / Anomalies</h2>
      <div className="actions">
        <button className="btn" type="button" onClick={() => void props.onRefreshValidation()}>
          Refresh validation
        </button>
        <button className="btn" type="button" onClick={() => void props.onRefreshIncidents()}>
          Refresh incidents
        </button>
        <button
          className="btn"
          type="button"
          onClick={props.onClearIncidentFilter}
          disabled={!props.hasIncidentFilter}
        >
          Clear incident filter
        </button>
      </div>

      {!props.validationReport ? (
        <p className="muted">Load validation to see incidents with warnings/errors.</p>
      ) : (
        <>
          <p className="hint">
            Showing incidents with warnings. Validators run in <code>crates/qir_core</code>; the UI only renders the payload.
          </p>
          <ul className="list">
            {props.validationReport
              .filter((i) => i.warnings.length > 0)
              .map((i) => (
                <li key={i.id}>
                  <div className="actions">
                    <span className="mono">{i.external_id ?? `id=${i.id}`}</span>
                    <span>{i.title}</span>
                    <span className="mono">warnings={i.warnings.length}</span>
                    <button
                      className="linkBtn"
                      type="button"
                      onClick={() => {
                        props.onFilterIncidentFromValidation(i.id, `validation:${i.external_id ?? `id=${i.id}`}`);
                      }}
                    >
                      Filter incidents table
                    </button>
                  </div>
                  {!props.dashboardLoaded ? <p className="hint">(Load the dashboard to view the incidents table.)</p> : null}
                  <ul className="list">
                    {i.warnings.map((w, idx) => (
                      <li key={idx}>
                        <span className="mono">{w.code}</span>: {w.message}{" "}
                        {w.details ? <span className="mono">({w.details})</span> : null}
                      </li>
                    ))}
                  </ul>
                </li>
              ))}
          </ul>
          {props.validationReport.filter((i) => i.warnings.length > 0).length === 0 && <p className="muted">No validation warnings found.</p>}
        </>
      )}
    </section>
  );
}

