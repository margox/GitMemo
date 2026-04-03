import { GitBranch } from "lucide-react";
import { useI18n } from "../hooks/useI18n";

export function NotInitialized() {
  const { t } = useI18n();
  return (
    <div style={{ display: "flex", alignItems: "center", justifyContent: "center", height: "100%" }}>
      <div style={{ textAlign: "center", padding: "0 32px" }}>
        <GitBranch size={48} style={{ color: "#555", margin: "0 auto 16px" }} />
        <p style={{ fontSize: 15, fontWeight: 600, marginBottom: 8, color: "var(--text)" }}>
          GitMemo not initialized
        </p>
        <p style={{ fontSize: 13, color: "var(--text-secondary)" }}>
          {t("dashboard.initHint")}
        </p>
      </div>
    </div>
  );
}
