# Android sharing (Mobile Preview 1)

This fork is based on nesszer/Win-CodexBar 0.70.0 and adds one optional Windows-to-Android usage feed. Provider sign-in and collection stay in the existing desktop app. The compatible receiver is [CodexBar Android 0.9.6 or newer](https://github.com/lingmulongtai/CodexBar-android/releases/latest).

## Setup

1. Start **Win-CodexBar Mobile** and connect the providers you want to monitor using its existing settings.
2. In **Settings → Advanced → Android sharing**, enter the Windows PC's numeric LAN or Tailscale IPv4 address. Enable sharing. The default TCP port is **43825**.
3. If Windows Firewall blocks it, allow this app on the trusted private network you use. Keep the selected address reachable from the phone. Do not expose the port through an internet router.
4. Use **Copy pairing code**, transfer that private code to your phone, and paste it into **CodexBar → Connections → Win-CodexBar**. Connect, then choose the services to receive from Windows.

日本語UIでは、Windowsの「設定 → 詳細設定 → Androidへの共有」を有効にし、Androidの「接続 → Win-CodexBar」にコピーしたコードを貼り付けます。PCとスマホが同じLANまたはTailscaleで接続されている必要があります。

The phone pulls snapshots during its normal dashboard and background refreshes; delivery is not an instant push service. Windows must be running and reachable. The Android refresh interval and Android's background scheduling affect notification timing.

Turning a service off on Android restores its saved phone connection. Disconnecting Windows preserves those native credentials. Changing the desktop's pairing key with **Revoke pairing** invalidates the old code; copy the new code and pair again. Disabling sharing stops the listener and closes active clients. Codes are passwords: do not publish them or include them in screenshots or issue reports.

## What is shared

Only a sanitized snapshot of supported, enabled providers is sent: quota windows, reset times, observation time, optional monthly budgets/balances, and known Codex reset credits. Unknown values remain unknown. Failed desktop fetches never masquerade as fresh zero usage. Observations older than 35 minutes are marked stale on Android; older than two hours are unavailable. History and reset tracking are scoped to an opaque account subject and cleared when the selected source changes.

The supported IDs are Codex, Claude, Gemini CLI, Copilot, Cursor, z.ai, ZenMux, Kimi Coding, ElevenLabs, OpenRouter, Synthetic, Chutes, DeepSeek, Venice, Moonshot API, ClinePass, IBM Bob, Fireworks, Devin and Antigravity. Only services with data collected by Windows can initially be enabled on Android. This does not add provider collectors.

**Gemini means Gemini CLI quotas. Google Gemini Apps / the Gemini Windows app's usage display is not collected by this integration.** Desktop and Android views do not have identical layouts or every desktop-specific field.

The feed does not contain provider credentials, email addresses, account names, local paths, prompts, session transcripts or raw errors. Pairing secrets are protected using Windows DPAPI and Android Keystore. Requests use HMAC-SHA256 authentication, timestamp checks and single-use nonces; responses use AES-256-GCM with the request identity as authenticated data. The local transport is TCP with newline-delimited JSON, bounded requests/responses, timeouts and a concurrent-client limit.

## Preview distribution and coexistence

The owned fork is [lingmulongtai/Win-CodexBar](https://github.com/lingmulongtai/Win-CodexBar). Its **Mobile Preview 1 portable executable is unsigned**. It does not use the original project's SignPath certificate or installer. Download it only from the owned fork's release and compare its SHA-256 with the published checksum. Preview updates are manual; the inherited numeric updater does not order preview revisions.

The product identifier, settings directory, auto-start identity, toast identity, updater cache and support links are separate from the upstream app. Existing CodexBar Android companion pairings and Windows residents are preserved. After verifying the new app with your actual accounts and phone, you can decide which older residents to remove. Two apps collecting the same account concurrently may do redundant provider requests.

## Validation and limits

Native Windows proof used an isolated configuration with synthetic quota data, the actual Tauri app, and Android's actual Kotlin receiver. It is evidence for transport interoperability, not a real-account or physical-phone acceptance test. Unit tests cover authentication vectors, malformed messages, replay rejection, listener shutdown, canonical provider IDs, unknown values, source changes, stale observations and account-scoped history. See the PR's validation report for exact results.

The preview has no cloud relay, phone discovery, QR scanner, automatic firewall changes or Google Gemini Apps collector. The pairing code already contains the endpoint; no provider token has to be entered again on the phone.
