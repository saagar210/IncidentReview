import { useCallback, useMemo, useState } from "react";

export type Toast = {
  id: string;
  kind: "success" | "warning" | "error";
  title: string;
  message: string;
};

export function useToasts() {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const pushToast = useCallback((t: Omit<Toast, "id">) => {
    const id = `${Date.now()}-${Math.random().toString(16).slice(2)}`;
    setToasts((prev) => [{ id, ...t }, ...prev].slice(0, 4));
  }, []);

  const dismissToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  return useMemo(() => ({ toasts, pushToast, dismissToast }), [toasts, pushToast, dismissToast]);
}

