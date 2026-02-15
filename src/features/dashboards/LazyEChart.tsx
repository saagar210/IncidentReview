import { lazy, Suspense, type ComponentProps } from "react";

// Lazy load ECharts component to reduce initial bundle
const ReactEChartsLazy = lazy(() => import("echarts-for-react"));

/**
 * Lazy-loaded wrapper for ECharts-for-React
 * Defers chart rendering and loading until component is mounted
 * Returns null while loading
 */
type LazyEChartProps = ComponentProps<typeof ReactEChartsLazy>;

export function LazyEChart(props: LazyEChartProps) {
  return (
    <Suspense fallback={<div className="chart-loading" />}>
      <ReactEChartsLazy {...props} />
    </Suspense>
  );
}
