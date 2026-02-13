import { lazy, Suspense } from "react";

// Lazy load ECharts component to reduce initial bundle
const ReactEChartsLazy = lazy(() => import("echarts-for-react"));

/**
 * Lazy-loaded wrapper for ECharts-for-React
 * Defers chart rendering and loading until component is mounted
 * Returns null while loading
 */
export function LazyEChart(props: any) {
  return (
    <Suspense fallback={<div className="chart-loading" />}>
      <ReactEChartsLazy {...props} />
    </Suspense>
  );
}
