import type { ReactNode } from "react";

export function Modal(props: { title: string; children: ReactNode; footer: ReactNode }) {
  return (
    <div className="modalOverlay" role="dialog" aria-modal="true">
      <div className="modal">
        <div className="modalHeader">
          <div className="modalTitle">{props.title}</div>
        </div>
        <div className="modalBody">{props.children}</div>
        <div className="modalFooter">{props.footer}</div>
      </div>
    </div>
  );
}

