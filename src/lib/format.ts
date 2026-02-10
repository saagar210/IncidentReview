export function formatSeconds(secs: number | null | undefined): string {
  if (secs == null) return "UNKNOWN";
  const minutes = Math.floor(secs / 60);
  const rem = secs % 60;
  if (minutes >= 60) {
    const hours = Math.floor(minutes / 60);
    const m = minutes % 60;
    return `${hours}h ${m}m`;
  }
  if (minutes > 0) return `${minutes}m ${rem}s`;
  return `${rem}s`;
}

