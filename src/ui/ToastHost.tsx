import type { Toast } from "./useToasts";

function toastClass(kind: Toast["kind"]) {
  switch (kind) {
    case "success":
      return "toast--success";
    case "warning":
      return "toast--warning";
    case "error":
      return "toast--error";
  }
}

export function ToastHost({
  toasts,
  onDismiss,
}: {
  toasts: Toast[];
  onDismiss: (id: string) => void;
}) {
  if (toasts.length === 0) return null;
  return (
    <div className="toastHost" role="status" aria-live="polite">
      {toasts.map((t) => (
        <div key={t.id} className={`toast ${toastClass(t.kind)}`}>
          <div className="toast__body">
            <div className="toast__title">{t.title}</div>
            <div className="toast__msg">{t.message}</div>
          </div>
          <button className="toast__x" type="button" onClick={() => onDismiss(t.id)} aria-label="Dismiss">
            x
          </button>
        </div>
      ))}
    </div>
  );
}

