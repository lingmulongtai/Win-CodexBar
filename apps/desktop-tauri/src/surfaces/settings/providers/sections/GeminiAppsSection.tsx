import { useEffect, useRef, useState } from "react";
import type { LocaleKey } from "../../../../i18n/keys";
import { beginGeminiBrowser, finishGeminiBrowser, cancelGeminiBrowser, connectGeminiApps, disconnectGeminiApps, getManualCookies } from "../../../../lib/tauri";

export function GeminiAppsSection({ disabled, t, onChanged }: {
  disabled: boolean; t: (key: LocaleKey) => string; onChanged: () => void;
}) {
  const [savedAt, setSavedAt] = useState<string | null>(null);
  const [busy, setBusy] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const mounted = useRef(false);
  useEffect(() => {
    mounted.current = true;
    void getManualCookies().then((cookies) => {
      if (mounted.current) setSavedAt(cookies.find(c => c.providerId === "geminiapps")?.savedAt ?? null);
    }).catch((reason: unknown) => { if (mounted.current) setError(String(reason)); })
      .finally(() => { if (mounted.current) setBusy(false); });
    return () => { mounted.current = false; };
  }, []);
  const change = async (action: "browser" | "finish" | "cancel" | "import" | "disconnect") => {
    if (busy || disabled) return;
    setBusy(true); setError(null);
    try {
      if (action === "browser") {
        await beginGeminiBrowser();
        if (mounted.current) setPending(true);
        return;
      }
      if (action === "cancel") {
        await cancelGeminiBrowser();
        if (mounted.current) setPending(false);
        return;
      }
      await (action === "finish" ? finishGeminiBrowser() : action === "import" ? connectGeminiApps() : disconnectGeminiApps());
      const cookies = await getManualCookies();
      if (mounted.current) {
        setSavedAt(cookies.find(c => c.providerId === "geminiapps")?.savedAt ?? null);
        setPending(false);
        onChanged();
      }
    } catch (reason: unknown) {
      if (action === "disconnect") {
        // Revocation can succeed even when Windows is still releasing the profile.
        const cookies = await getManualCookies().catch(() => null);
        if (cookies && mounted.current) {
          setSavedAt(cookies.find(c => c.providerId === "geminiapps")?.savedAt ?? null);
          setPending(false);
          onChanged();
        }
      }
      if (mounted.current) setError(String(reason));
    }
    finally { if (mounted.current) setBusy(false); }
  };
  return <section className="provider-detail-section">
    <h3>{t("GeminiAppsTitle")}</h3>
    <p className="provider-detail-helper">{t("GeminiAppsBrowserHelp")}</p>
    <p>{savedAt ? `${t("GeminiAppsSaved")} ${savedAt}` : t("GeminiAppsNotConnected")}</p>
    <button className="credential-btn credential-btn--primary" disabled={disabled || busy} onClick={() => void change("browser")}>
      {t("GeminiAppsBrowserOpen")}
    </button>{" "}
    {pending && <>
      <p role="status" className="provider-detail-helper">{t("GeminiAppsBrowserPending")}</p>
      <button className="credential-btn credential-btn--primary" disabled={disabled || busy} onClick={() => void change("finish")}>{t("GeminiAppsBrowserFinish")}</button>{" "}
      <button className="credential-btn" disabled={disabled || busy} onClick={() => void change("cancel")}>{t("GeminiAppsBrowserCancel")}</button>{" "}
    </>}
    {savedAt && <button className="credential-btn credential-btn--danger" disabled={disabled || busy} onClick={() => void change("disconnect")}>
      {t("GeminiAppsDisconnect")}
    </button>}
    <details>
      <summary>{t("GeminiAppsLegacyImport")}</summary>
      <p className="provider-detail-helper">{t("GeminiAppsHelp")}</p>
      <button className="credential-btn" disabled={disabled || busy} onClick={() => void change("import")}>{t("GeminiAppsConnect")}</button>
    </details>
    {error && <p role="alert" className="provider-detail-error">{error}</p>}
  </section>;
}
