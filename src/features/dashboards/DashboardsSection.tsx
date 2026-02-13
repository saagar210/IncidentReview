import { useMemo } from "react";
import { LazyEChart } from "./LazyEChart";

import { formatSeconds } from "../../lib/format";

type Bucket = { key: string; label: string; count: number; incident_ids: number[] };

export type DashboardIncidentRow = {
  id: number;
  external_id: string | null;
  title: string;
  severity: string | null;
  detection_source: string | null;
  vendor: string | null;
  service: string | null;
  it_awareness_lag_seconds: number | null;
  time_to_mitigation_seconds: number | null;
  mttr_seconds: number | null;
  warning_count: number;
};

export type DashboardPayloadV2 = {
  version: number;
  incident_count: number;
  severity_counts: Array<{ severity: string; count: number; incident_ids: number[] }>;
  incidents: DashboardIncidentRow[];
  detection_story: {
    detection_source_mix: Bucket[];
    it_awareness_lag_buckets: Bucket[];
  };
  vendor_service_story: {
    top_vendors_by_count: Bucket[];
    top_services_by_count: Bucket[];
    top_vendors_by_pain: Array<Bucket & { pain_sum: number; pain_known_count: number }>;
    top_services_by_pain: Array<Bucket & { pain_sum: number; pain_known_count: number }>;
  };
  response_story: {
    time_to_mitigation_buckets: Bucket[];
    time_to_resolve_buckets: Bucket[];
  };
};

export function DashboardsSection(props: {
  dashboard: DashboardPayloadV2 | null;
  selectedSeverity: string | null;
  setSelectedSeverity: (next: string | null) => void;
  incidentFilterIds: number[] | null;
  incidentFilterLabel: string;
  setIncidentFilterIds: (next: number[] | null) => void;
  setIncidentFilterLabel: (next: string) => void;
  onOpenIncidentDetail: (id: number) => void | Promise<void>;
}) {
  function applyIncidentFilter(ids: number[], label: string) {
    const sorted = [...ids].sort((a, b) => a - b);
    props.setIncidentFilterIds(sorted);
    props.setIncidentFilterLabel(label);
  }

  const filteredIncidents = useMemo(() => {
    if (!props.dashboard) return [];
    if (props.incidentFilterIds && props.incidentFilterIds.length > 0) {
      const set = new Set(props.incidentFilterIds);
      return props.dashboard.incidents.filter((i) => set.has(i.id));
    }
    if (!props.selectedSeverity) return props.dashboard.incidents;
    return props.dashboard.incidents.filter((i) => (i.severity ?? "UNKNOWN") === props.selectedSeverity);
  }, [props.dashboard, props.incidentFilterIds, props.selectedSeverity]);

  const severityChartOption = useMemo(() => {
    if (!props.dashboard) return null;
    return {
      tooltip: { trigger: "item" },
      series: [
        {
          name: "Severity",
          type: "pie",
          radius: ["45%", "70%"],
          label: { show: true, formatter: "{b}: {c}" },
          data: props.dashboard.severity_counts.map((s) => ({
            name: s.severity,
            value: s.count,
            incident_ids: s.incident_ids,
            key: `severity:${s.severity}`,
          })),
        },
      ],
    };
  }, [props.dashboard]);

  const detectionSourceChartOption = useMemo(() => {
    if (!props.dashboard) return null;
    return {
      tooltip: { trigger: "item" },
      series: [
        {
          name: "Detection Source",
          type: "pie",
          radius: ["35%", "70%"],
          label: { show: true, formatter: "{b}: {c}" },
          data: props.dashboard.detection_story.detection_source_mix.map((b) => ({
            name: b.label,
            value: b.count,
            incident_ids: b.incident_ids,
            key: b.key,
          })),
        },
      ],
    };
  }, [props.dashboard]);

  const itAwarenessLagOption = useMemo(() => {
    if (!props.dashboard) return null;
    const buckets = props.dashboard.detection_story.it_awareness_lag_buckets;
    return {
      tooltip: { trigger: "axis" },
      xAxis: { type: "category", data: buckets.map((b) => b.label), axisLabel: { interval: 0 } },
      yAxis: { type: "value" },
      series: [
        {
          name: "IT awareness lag",
          type: "bar",
          data: buckets.map((b) => ({
            value: b.count,
            incident_ids: b.incident_ids,
            key: b.key,
          })),
        },
      ],
    };
  }, [props.dashboard]);

  const vendorCountOption = useMemo(() => {
    if (!props.dashboard) return null;
    const buckets = props.dashboard.vendor_service_story.top_vendors_by_count;
    return {
      tooltip: { trigger: "axis" },
      xAxis: { type: "category", data: buckets.map((b) => b.label), axisLabel: { interval: 0, rotate: 25 } },
      yAxis: { type: "value" },
      series: [
        {
          name: "Incidents",
          type: "bar",
          data: buckets.map((b) => ({ value: b.count, incident_ids: b.incident_ids, key: b.key })),
        },
      ],
    };
  }, [props.dashboard]);

  const vendorPainOption = useMemo(() => {
    if (!props.dashboard) return null;
    const buckets = props.dashboard.vendor_service_story.top_vendors_by_pain;
    return {
      tooltip: { trigger: "axis" },
      xAxis: { type: "category", data: buckets.map((b) => b.label), axisLabel: { interval: 0, rotate: 25 } },
      yAxis: { type: "value" },
      series: [
        {
          name: "Pain (impact * degradation * duration)",
          type: "bar",
          data: buckets.map((b) => ({
            value: b.pain_sum,
            incident_ids: b.incident_ids,
            key: b.key,
            pain_known_count: b.pain_known_count,
          })),
        },
      ],
    };
  }, [props.dashboard]);

  const serviceCountOption = useMemo(() => {
    if (!props.dashboard) return null;
    const buckets = props.dashboard.vendor_service_story.top_services_by_count;
    return {
      tooltip: { trigger: "axis" },
      xAxis: { type: "category", data: buckets.map((b) => b.label), axisLabel: { interval: 0, rotate: 25 } },
      yAxis: { type: "value" },
      series: [
        {
          name: "Incidents",
          type: "bar",
          data: buckets.map((b) => ({ value: b.count, incident_ids: b.incident_ids, key: b.key })),
        },
      ],
    };
  }, [props.dashboard]);

  const servicePainOption = useMemo(() => {
    if (!props.dashboard) return null;
    const buckets = props.dashboard.vendor_service_story.top_services_by_pain;
    return {
      tooltip: { trigger: "axis" },
      xAxis: { type: "category", data: buckets.map((b) => b.label), axisLabel: { interval: 0, rotate: 25 } },
      yAxis: { type: "value" },
      series: [
        {
          name: "Pain (impact * degradation * duration)",
          type: "bar",
          data: buckets.map((b) => ({
            value: b.pain_sum,
            incident_ids: b.incident_ids,
            key: b.key,
            pain_known_count: b.pain_known_count,
          })),
        },
      ],
    };
  }, [props.dashboard]);

  const timeToMitigationOption = useMemo(() => {
    if (!props.dashboard) return null;
    const buckets = props.dashboard.response_story.time_to_mitigation_buckets;
    return {
      tooltip: { trigger: "axis" },
      xAxis: { type: "category", data: buckets.map((b) => b.label), axisLabel: { interval: 0 } },
      yAxis: { type: "value" },
      series: [
        {
          name: "Time to mitigate",
          type: "bar",
          data: buckets.map((b) => ({ value: b.count, incident_ids: b.incident_ids, key: b.key })),
        },
      ],
    };
  }, [props.dashboard]);

  const timeToResolveOption = useMemo(() => {
    if (!props.dashboard) return null;
    const buckets = props.dashboard.response_story.time_to_resolve_buckets;
    return {
      tooltip: { trigger: "axis" },
      xAxis: { type: "category", data: buckets.map((b) => b.label), axisLabel: { interval: 0 } },
      yAxis: { type: "value" },
      series: [
        {
          name: "Time to resolve",
          type: "bar",
          data: buckets.map((b) => ({ value: b.count, incident_ids: b.incident_ids, key: b.key })),
        },
      ],
    };
  }, [props.dashboard]);

  return (
    <>
      <section className="grid" id="dashboards">
        <section className="card">
          <h2>Quarter At A Glance</h2>
          {!props.dashboard ? (
            <p className="muted">Load the dashboard to view severity distribution and incidents.</p>
          ) : (
            <>
              <div className="kpiRow">
                <div className="kpi">
                  <div className="kpi__label">Incident Count</div>
                  <div className="kpi__value">{props.dashboard.incident_count}</div>
                </div>
                <div className="kpi">
                  <div className="kpi__label">Selected Severity</div>
                  <div className="kpi__value">{props.selectedSeverity ?? "ALL"}</div>
                </div>
                <div className="kpi">
                  <div className="kpi__label">Incident Filter</div>
                  <div className="kpi__value">
                    {!props.incidentFilterIds || props.incidentFilterIds.length === 0 ? "NONE" : `${props.incidentFilterIds.length} selected`}
                  </div>
                </div>
              </div>

              {severityChartOption && (
                <div className="chart">
                  <LazyEChart
                    option={severityChartOption}
                    style={{ height: 260 }}
                    onEvents={{
                      click: (params: { name?: string; data?: { incident_ids?: number[]; key?: string } }) => {
                        props.setSelectedSeverity(params?.name ?? null);
                        const ids = params?.data?.incident_ids;
                        const key = params?.data?.key ?? "severity";
                        if (ids && ids.length > 0) {
                          applyIncidentFilter(ids, key);
                        }
                      },
                    }}
                  />
                  <div className="chart__footer">
                    <button className="linkBtn" type="button" onClick={() => props.setSelectedSeverity(null)}>
                      Clear filter
                    </button>
                    <button
                      className="linkBtn"
                      type="button"
                      onClick={() => {
                        props.setIncidentFilterIds(null);
                        props.setIncidentFilterLabel("");
                      }}
                      disabled={!props.incidentFilterIds || props.incidentFilterIds.length === 0}
                    >
                      Clear incident filter
                    </button>
                  </div>
                </div>
              )}
            </>
          )}
        </section>

        <section className="card">
          <h2>Incidents (Drill-down)</h2>
          {!props.dashboard ? (
            <p className="muted">Load the dashboard first.</p>
          ) : (
            <>
              {props.incidentFilterIds && props.incidentFilterIds.length > 0 && (
                <p className="hint">
                  Filtering to <span className="mono">{props.incidentFilterLabel || `${props.incidentFilterIds.length} incidents`}</span>.{" "}
                  <button
                    className="linkBtn"
                    type="button"
                    onClick={() => {
                      props.setIncidentFilterIds(null);
                      props.setIncidentFilterLabel("");
                    }}
                  >
                    Clear
                  </button>
                </p>
              )}
              <div className="tableWrap">
                <table className="table">
                  <thead>
                    <tr>
                      <th>External ID</th>
                      <th>Title</th>
                      <th>Severity</th>
                      <th>Awareness lag</th>
                      <th>Time to mitigate</th>
                      <th>MTTR</th>
                      <th>Warnings</th>
                    </tr>
                  </thead>
                  <tbody>
                    {filteredIncidents.map((i) => (
                      <tr key={i.id}>
                        <td className="mono">{i.external_id ?? "NO_EXTERNAL_ID"}</td>
                        <td>
                          <button className="linkBtn" type="button" onClick={() => void props.onOpenIncidentDetail(i.id)}>
                            {i.title}
                          </button>
                        </td>
                        <td>{i.severity ?? "UNKNOWN"}</td>
                        <td className="mono">{formatSeconds(i.it_awareness_lag_seconds)}</td>
                        <td className="mono">{formatSeconds(i.time_to_mitigation_seconds)}</td>
                        <td className="mono">{formatSeconds(i.mttr_seconds)}</td>
                        <td className="mono">{i.warning_count}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
              <p className="hint">
                Reconciliation check: the severity counts sum to the incident total (enforced in <code>crates/qir_core</code> tests).
              </p>
            </>
          )}
        </section>
      </section>

      <section className="grid">
        <section className="card">
          <h2>Detection Story</h2>
          {!props.dashboard ? (
            <p className="muted">Load the dashboard first.</p>
          ) : (
            <>
              <p className="hint">All charts reconcile to total incidents via explicit UNKNOWN buckets.</p>
              <div className="chart">
                <h3 className="subhead">Detection Source Mix</h3>
                {detectionSourceChartOption && (
                  <LazyEChart
                    option={detectionSourceChartOption}
                    style={{ height: 260 }}
                    onEvents={{
                      click: (params: { data?: { incident_ids?: number[]; key?: string } }) => {
                        const ids = params?.data?.incident_ids;
                        const key = params?.data?.key ?? "detection_source";
                        if (ids && ids.length > 0) applyIncidentFilter(ids, key);
                      },
                    }}
                  />
                )}
              </div>
              <div className="chart">
                <h3 className="subhead">IT Awareness Lag Distribution</h3>
                {itAwarenessLagOption && (
                  <LazyEChart
                    option={itAwarenessLagOption}
                    style={{ height: 260 }}
                    onEvents={{
                      click: (params: { data?: { incident_ids?: number[]; key?: string } }) => {
                        const ids = params?.data?.incident_ids;
                        const key = params?.data?.key ?? "it_awareness_lag";
                        if (ids && ids.length > 0) applyIncidentFilter(ids, key);
                      },
                    }}
                  />
                )}
              </div>
            </>
          )}
        </section>

        <section className="card">
          <h2>Response Story</h2>
          {!props.dashboard ? (
            <p className="muted">Load the dashboard first.</p>
          ) : (
            <>
              <div className="chart">
                <h3 className="subhead">Time To Mitigate</h3>
                {timeToMitigationOption && (
                  <LazyEChart
                    option={timeToMitigationOption}
                    style={{ height: 260 }}
                    onEvents={{
                      click: (params: { data?: { incident_ids?: number[]; key?: string } }) => {
                        const ids = params?.data?.incident_ids;
                        const key = params?.data?.key ?? "time_to_mitigation";
                        if (ids && ids.length > 0) applyIncidentFilter(ids, key);
                      },
                    }}
                  />
                )}
              </div>
              <div className="chart">
                <h3 className="subhead">Time To Resolve</h3>
                {timeToResolveOption && (
                  <LazyEChart
                    option={timeToResolveOption}
                    style={{ height: 260 }}
                    onEvents={{
                      click: (params: { data?: { incident_ids?: number[]; key?: string } }) => {
                        const ids = params?.data?.incident_ids;
                        const key = params?.data?.key ?? "time_to_resolve";
                        if (ids && ids.length > 0) applyIncidentFilter(ids, key);
                      },
                    }}
                  />
                )}
              </div>
            </>
          )}
        </section>
      </section>

      <section className="grid">
        <section className="card">
          <h2>Vendor/Service Reliability</h2>
          {!props.dashboard ? (
            <p className="muted">Load the dashboard first.</p>
          ) : (
            <>
              <div className="chart">
                <h3 className="subhead">Top Vendors By Incident Count</h3>
                {vendorCountOption && (
                  <LazyEChart
                    option={vendorCountOption}
                    style={{ height: 260 }}
                    onEvents={{
                      click: (params: { data?: { incident_ids?: number[]; key?: string } }) => {
                        const ids = params?.data?.incident_ids;
                        const key = params?.data?.key ?? "vendor_count";
                        if (ids && ids.length > 0) applyIncidentFilter(ids, key);
                      },
                    }}
                  />
                )}
              </div>
              <div className="chart">
                <h3 className="subhead">Top Vendors By Weighted Pain</h3>
                {vendorPainOption && (
                  <LazyEChart
                    option={vendorPainOption}
                    style={{ height: 260 }}
                    onEvents={{
                      click: (params: { data?: { incident_ids?: number[]; key?: string } }) => {
                        const ids = params?.data?.incident_ids;
                        const key = params?.data?.key ?? "vendor_pain";
                        if (ids && ids.length > 0) applyIncidentFilter(ids, key);
                      },
                    }}
                  />
                )}
                <p className="hint">
                  Pain is computed deterministically as <span className="mono">impact * degradation * duration_seconds</span> when inputs are present; otherwise it contributes 0 to pain_sum but incidents still drill down.
                </p>
              </div>
            </>
          )}
        </section>

        <section className="card">
          <h2>Service Reliability</h2>
          {!props.dashboard ? (
            <p className="muted">Load the dashboard first.</p>
          ) : (
            <>
              <div className="chart">
                <h3 className="subhead">Top Services By Incident Count</h3>
                {serviceCountOption && (
                  <LazyEChart
                    option={serviceCountOption}
                    style={{ height: 260 }}
                    onEvents={{
                      click: (params: { data?: { incident_ids?: number[]; key?: string } }) => {
                        const ids = params?.data?.incident_ids;
                        const key = params?.data?.key ?? "service_count";
                        if (ids && ids.length > 0) applyIncidentFilter(ids, key);
                      },
                    }}
                  />
                )}
              </div>
              <div className="chart">
                <h3 className="subhead">Top Services By Weighted Pain</h3>
                {servicePainOption && (
                  <LazyEChart
                    option={servicePainOption}
                    style={{ height: 260 }}
                    onEvents={{
                      click: (params: { data?: { incident_ids?: number[]; key?: string } }) => {
                        const ids = params?.data?.incident_ids;
                        const key = params?.data?.key ?? "service_pain";
                        if (ids && ids.length > 0) applyIncidentFilter(ids, key);
                      },
                    }}
                  />
                )}
              </div>
            </>
          )}
        </section>
      </section>
    </>
  );
}

