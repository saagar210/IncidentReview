export function ReportSection(props: { reportMd: string }) {
  return (
    <section className="card" id="report">
      <h2>QIR Report (Markdown)</h2>
      <textarea className="md" value={props.reportMd} readOnly placeholder="Generate the report to view Markdown output." />
    </section>
  );
}

