type NavItem = {
  label: string;
  href: string;
  kind?: "accent";
};

export function AppNav(props: { items: NavItem[] }) {
  return (
    <nav className="card" aria-label="Navigation">
      <h2>Navigate</h2>
      <div className="actions">
        {props.items.map((i) => (
          <a key={i.href} className={i.kind === "accent" ? "btn btn--accent" : "btn"} href={i.href}>
            {i.label}
          </a>
        ))}
      </div>
      <p className="hint">All metrics are computed deterministically in Rust; the UI renders only.</p>
    </nav>
  );
}

