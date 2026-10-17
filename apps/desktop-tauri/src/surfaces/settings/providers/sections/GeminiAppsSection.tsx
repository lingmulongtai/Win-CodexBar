import { useEffect, useRef, useState } from "react";
import type { LocaleKey } from "../../../../i18n/keys";
import { beginGeminiBrowser, finishGeminiBrowser, cancelGeminiBrowser, connectGeminiApps, connectGeminiAppsFromBrowser, listDetectedBrowsers, listGeminiBrowserProfiles, disconnectGeminiApps, getManualCookies } from "../../../../lib/tauri";
import type { DetectedBrowserBridge } from "../../../../types/bridge";
import { Select } from "../../../../components/FormControls";

export function GeminiAppsSection({ disabled, t, onChanged }: {
  disabled: boolean; t: (key: LocaleKey) => string; onChanged: () => void;
}) {
  const [savedAt, setSavedAt] = useState<string | null>(null);
  const [busy, setBusy] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const [browsers, setBrowsers] = useState<DetectedBrowserBridge[]>([]);
  const [browserType, setBrowserType] = useState("");
  const [profiles, setProfiles] = useState<string[]>([]);
  const [profileName, setProfileName] = useState("");
  const mounted = useRef(false);
  useEffect(() => {
    mounted.current = true;
    void listDetectedBrowsers().then((found) => {
      if (!mounted.current) return;
      const supported = found.filter(browser => browser.browserType !== "firefox");
      setBrowsers(supported);
      setBrowserType(supported.find(browser => browser.browserType === "vivaldi")?.browserType ?? supported[0]?.browserType ?? "");
    }).catch((reason: unknown) => { if (mounted.current) setError(String(reason)); });
    void getManualCookies().then((cookies) => {
      if (mounted.current) setSavedAt(cookies.find(c => c.providerId === "geminiapps")?.savedAt ?? null);
    }).catch((reason: unknown) => { if (mounted.current) setError(String(reason)); })
      .finally(() => { if (mounted.current) setBusy(false); });
    return () => { mounted.current = false; };
  }, []);
  useEffect(() => {
    let current = true;
    setProfiles([]); setProfileName("");
    if (browserType) void listGeminiBrowserProfiles(browserType).then((found) => {
      if (current) { setProfiles(found); setProfileName(found[0] ?? ""); }
    }).catch((reason: unknown) => { if (current) setError(String(reason)); });
    return () => { current = false; };
  }, [browserType]);
  const change = async (action: "browser" | "finish" | "cancel" | "import" | "browser-import" | "disconnect") => {
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
      await (action === "finish" ? finishGeminiBrowser() : action === "import" ? connectGeminiApps() : action === "browser-import" ? connectGeminiAppsFromBrowser(browserType, profileName) : disconnectGeminiApps());
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
    <p>{savedAt ? `${t("GeminiAppsSaved")} ${savedAt}` : t("GeminiAppsNotConnected")}</p>
    <h4>{t("BrowserCookieImportFromBrowser")}</h4>
    <p className="provider-detail-helper">{t("GeminiAppsCookieImportHelp")}</p>
    {browsers.length > 0 && <div className="credential-add-form">
      <Select ariaLabel={t("SelectBrowser")} value={browserType} options={browsers.map(browser => ({ value: browser.browserType, label: browser.displayName }))} disabled={disabled || busy} onChange={value => { setProfiles([]); setProfileName(""); setBrowserType(value); }} />
      <Select ariaLabel={t("GeminiAppsProfile")} value={profileName} options={profiles.map(profile => ({ value: profile, label: profile }))} disabled={disabled || busy || profiles.length === 0} onChange={setProfileName} />
      <button className="credential-btn credential-btn--primary" disabled={disabled || busy || !profileName} onClick={() => void change("browser-import")}>{t("GeminiAppsBrowserImport")}</button>
    </div>}
    <h4>{t("GeminiAppsDedicatedLogin")}</h4>
    <p className="provider-detail-helper">{t("GeminiAppsBrowserHelp")}</p>
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
