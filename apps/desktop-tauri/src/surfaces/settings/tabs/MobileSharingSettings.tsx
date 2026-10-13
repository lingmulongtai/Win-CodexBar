import { useEffect, useState } from "react";
import { Field, SettingsGroup, Toggle } from "../../../components/FormControls";
import { useLocale } from "../../../hooks/useLocale";
import { configureMobileSharing, getMobilePairingCode, getMobileSharingStatus, revokeMobilePairing } from "../../../lib/tauri";
import type { MobileSharingStatus } from "../../../types/bridge";

export default function MobileSharingSettings() {
  const { t } = useLocale();
  const [status, setStatus] = useState<MobileSharingStatus | null>(null);
  const [address, setAddress] = useState("");
  const [port, setPort] = useState("43825");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  useEffect(() => {
    let cancelled = false;
    getMobileSharingStatus().then((next) => {
      if (cancelled) return;
      setStatus(next); setAddress(next.address); setPort(String(next.port));
    }).catch(() => { if (!cancelled) setMessage(t("MobileSharingFailed")); });
    return () => { cancelled = true; };
  }, [t]);
  async function run(action: () => Promise<void>) {
    if (busy) return;
    setBusy(true); setMessage(null);
    try { await action(); } catch (error) { setMessage(`${t("MobileSharingFailed")} ${String(error)}`); }
    finally { setBusy(false); }
  }
  async function save(enabled: boolean) {
    const parsedPort = Number(port);
    if (!Number.isInteger(parsedPort) || parsedPort < 1024 || parsedPort > 65535) throw new Error(t("MobileSharingPort"));
    setStatus(await configureMobileSharing(enabled, address, parsedPort));
  }
  return <SettingsGroup title={t("MobileSharingTitle")} footer={<p>{t("MobileSharingHelper")}</p>}>
    <Field label={t("MobileSharingEnable")}>
      <Toggle variant="switch" checked={status?.enabled ?? false} ariaLabel={t("MobileSharingEnable")} disabled={busy || !status} onChange={(enabled) => void run(async () => {
        if (!enabled && status) setStatus(await configureMobileSharing(false, status.address, status.port));
        else await save(enabled);
      })} />
    </Field>
    <Field label={t("MobileSharingAddress")}>
      <input className="text-input" aria-label={t("MobileSharingAddress")} value={address} disabled={busy || !status} onChange={(event) => setAddress(event.target.value)} autoComplete="off" spellCheck={false} placeholder="192.168.1.10" />
    </Field>
    <Field label={t("MobileSharingPort")}>
      <input className="text-input" aria-label={t("MobileSharingPort")} type="number" min={1024} max={65535} value={port} disabled={busy || !status} onChange={(event) => setPort(event.target.value)} />
    </Field>
    <div className="settings-group__row">
      <button className="credential-btn" disabled={busy || !status} onClick={() => void run(() => save(status?.enabled ?? false))}>{t("MobileSharingSave")}</button>
      <button className="credential-btn" disabled={busy || !status?.running} onClick={() => void run(async () => {
        const code = await getMobilePairingCode();
        await navigator.clipboard.writeText(code); setMessage(t("MobileSharingCopied"));
      })}>{t("MobileSharingCopy")}</button>
      <button className="credential-btn" disabled={busy || !status} onClick={() => void run(async () => {
        setStatus(await revokeMobilePairing()); setMessage(t("MobileSharingRevoked"));
      })}>{t("MobileSharingRevoke")}</button>
    </div>
    <p role="status">{message ?? status?.error ?? (status?.running ? t("MobileSharingRunning") : t("MobileSharingStopped"))}</p>
  </SettingsGroup>;
}
