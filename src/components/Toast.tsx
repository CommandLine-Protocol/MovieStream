import React from "react";
import { useToast, ToastType } from "../state/ToastContext";
import { CheckCircle2, AlertCircle, Info, AlertTriangle, X } from "lucide-react";

export const ToastContainer: React.FC = () => {
  const { toasts, removeToast } = useToast();

  if (toasts.length === 0) return null;

  const getIcon = (type: ToastType) => {
    switch (type) {
      case "success":
        return <CheckCircle2 size={18} color="#10b981" />;
      case "error":
        return <AlertCircle size={18} color="#ef4444" />;
      case "warning":
        return <AlertTriangle size={18} color="#f59e0b" />;
      default:
        return <Info size={18} color="#06b6d4" />;
    }
  };

  return (
    <div
      style={{
        position: "fixed",
        bottom: 24,
        right: 24,
        zIndex: 2000,
        display: "flex",
        flexDirection: "column",
        gap: 12,
        maxWidth: 380,
      }}
    >
      {toasts.map((t) => (
        <div
          key={t.id}
          className="glass-panel"
          style={{
            display: "flex",
            alignItems: "flex-start",
            gap: 12,
            padding: "12px 16px",
            background: "rgba(18, 19, 28, 0.95)",
            boxShadow: "0 10px 30px rgba(0, 0, 0, 0.6)",
            borderLeft: `4px solid ${
              t.type === "success"
                ? "#10b981"
                : t.type === "error"
                ? "#ef4444"
                : t.type === "warning"
                ? "#f59e0b"
                : "#06b6d4"
            }`,
            animation: "slideIn 0.25s ease forwards",
          }}
        >
          <div style={{ marginTop: 2 }}>{getIcon(t.type)}</div>
          <div style={{ flex: 1 }}>
            <div style={{ fontWeight: 600, fontSize: "0.9rem" }}>{t.title}</div>
            {t.message && (
              <div style={{ fontSize: "0.8rem", color: "var(--text-secondary)", marginTop: 2 }}>
                {t.message}
              </div>
            )}
          </div>
          <button
            onClick={() => removeToast(t.id)}
            style={{
              background: "transparent",
              border: "none",
              color: "var(--text-muted)",
              cursor: "pointer",
              padding: 2,
            }}
          >
            <X size={14} />
          </button>
        </div>
      ))}
    </div>
  );
};
