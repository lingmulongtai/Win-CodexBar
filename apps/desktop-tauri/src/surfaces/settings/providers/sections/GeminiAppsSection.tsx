import { useEffect, useRef, useState } from "react";
import type { LocaleKey } from "../../../../i18n/keys";
import { connectGeminiApps, disconnectGeminiApps, getManualCookies } from "../../../../lib/tauri";

export function GeminiAppsSection({ disabled, t, onChanged }: {
  disabled: boolean; t: (key: LocaleKey) => string; onChanged: () => void;
}) {
  const [savedAt, setSavedAt] = useState<string | null>(null);
  const [busy, setBusy] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const mounted = useRef(false);
  useEffect(() => {
    mounted.current = true;
    void getManualCookies().then((cookies) => {
      if (mounted.current) setSavedAt(cookies.find(c => c.providerId === "geminiapps")?.savedAt ?? null);
    }).catch((reason: unknown) => { if (mounted.current) setError(String(reason)); })
      .finally(() => { if (mounted.current) setBusy(false); });
    return () => { mounted.current = false; };
  }, []);
  const change = async (connect: boolean) => {
    if (busy || disabled) return;
    setBusy(true); setError(null);
    try {
      await (connect ? connectGeminiApps() : disconnectGeminiApps());
      const cookies = await getManualCookies();
      if (mounted.current) {
        setSavedAt(cookies.find(c => c.providerId === "geminiapps")?.savedAt ?? null);
        onChanged();
      }
    } catch (reason: unknown) { if (mounted.current) setError(String(reason)); }
    finally { if (mounted.current) setBusy(false); }
  };
  return <section className="provider-detail-section">
    <h3>{t("GeminiAppsTitle")}</h3>
    <p className="provider-detail-helper">{t("GeminiAppsHelp")}</p>
    <p>{savedAt ? `${t("GeminiAppsSaved")} ${savedAt}` : t("GeminiAppsNotConnected")}</p>
    <button className="credential-btn credential-btn--primary" disabled={disabled || busy} onClick={() => void change(true)}>
      {t("GeminiAppsConnect")}
    </button>{" "}
    {savedAt && <button className="credential-btn credential-btn--danger" disabled={disabled || busy} onClick={() => void change(false)}>
      {t("GeminiAppsDisconnect")}
    </button>}
    {error && <p role="alert" className="provider-detail-error">{error}</p>}
  </section>;
}
