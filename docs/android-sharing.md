# Android sharing — Win-CodexBar + Mobile Connect

This independent fork adds an optional encrypted Windows-to-Android feed to the existing Win-CodexBar collectors. Windows v0.70.2 and [CodexBar Android v0.9.7](https://github.com/lingmulongtai/CodexBar-android/releases/latest) add explicit acquisition modes and PC log totals. Android v0.9.6 can still read basic quota snapshots; update to v0.9.7 for the new features.

## Setup

1. [Download the EXE](https://github.com/lingmulongtai/Win-CodexBar-Mobile-Connect/releases/latest/download/Win-CodexBar-Mobile-Connect.exe), start **Win-CodexBar + Mobile Connect** and connect/enable your providers.
2. In **Settings → Advanced → Android sharing**, select **Tailscale (no exit node)** or **LAN** to use the detected PC address. You can also enter a numeric private IPv4 address. Enable sharing. Default TCP port: **43825**.
3. If Windows Firewall blocks it, allow this app on the trusted private network you use. Keep the selected address reachable from the phone. Do not expose the port through an internet router.
4. Use **Copy pairing code**, transfer that private code to the phone, and paste it into **CodexBar → Connections → Usage source & Windows connection**. Connecting selects Windows mode.
5. Choose **Collect on this phone** or **Collect on Windows**. Advanced phone exceptions select individual saved phone connections. Choices survive mode changes; **Use Windows for all services** explicitly clears exceptions.

日本語UIではWindowsの「詳細設定 → Androidへの共有」で接続先を選び、Androidの「接続 → 取得方法とWindows接続」にコードを貼り付けます。「スマホで取得」「Windowsでまとめて取得」を選択できます。旧版の選択はスマホ例外として引き継ぎ、全取得をPCへ移す場合は「すべてWindowsから取得する」を選びます。

## Tailscale without an exit node

Connect both devices to the same tailnet, select the PC's Tailscale `100.x.x.x` address in Windows sharing, and copy the updated endpoint code into Android. The phone's exit node can stay **None**. No subnet router or internet router port forwarding is needed for direct tailnet-device access. Existing tailnet access rules and Windows Firewall must permit TCP 43825. See [Tailscale's connection guide](https://tailscale.com/docs/how-to/connect-to-devices) and [exit-node explanation](https://tailscale.com/docs/features/exit-nodes).

If detection is unavailable, confirm the Tailscale app is installed/connected or enter the numeric address shown in Tailscale manually. An enabled exit node is not required for discovery; the app only calls `tailscale ip -4` and does not change VPN routing. Changing the endpoint preserves the pairing key, but the phone needs the code with the new address. LAN mode requires a reachable LAN address instead.

The phone pulls snapshots during its normal dashboard and background refreshes; delivery is not an instant push service. Windows must be running and reachable. The Android refresh interval and Android's background scheduling affect notification timing.

Phone mode uses saved phone connections without automatic PC polling. Windows mode uses the PC, automatically discovers new known enabled services and never silently falls back to a different phone account when the PC is offline. Switching or disconnecting Windows preserves native credentials and archived source/account histories. Revoke pairing invalidates the key; copy a new code and pair again. Disabling sharing stops the listener and active clients. Codes grant usage access: do not publish them or include them in screenshots.

## What is shared

Sanitized supported-provider snapshots include quota windows, reset times, observation time, optional budgets/balances and known Codex reset credits. Unknown values remain unknown. Failed fetches never look like fresh zero usage. Quota observations over 35 minutes old are stale; over two hours old are unavailable. History/reset baselines are archived separately by opaque account/source and retained when switching, bounded by existing retention limits.

Optional **Local usage on this PC** totals come from completed cached Windows log scans: today/30-day/selected-period tokens and estimated USD costs, PC calendar date, aggregation time, a sanitized model label and incomplete-request count where known. They may include multiple accounts and are separate from the signed-in quota account and billed charges. Local aggregation has its own stale label. Sharing never starts a scan; enable local usage/cost features on Windows when needed. This does not reproduce every desktop-only field or layout.

Supported IDs are Codex, Claude, Gemini CLI, Copilot, Cursor, z.ai, ZenMux, Kimi Coding, ElevenLabs, OpenRouter, Synthetic, Chutes, DeepSeek, Venice, Moonshot API, ClinePass, IBM Bob, Fireworks, Devin and Antigravity. Authentication-required/unavailable PC providers remain PC-sourced with an actionable status. This does not add collectors.

**Gemini means Gemini CLI quotas. Google Gemini Apps / the Gemini Windows app's usage display is not collected by this integration.** Desktop and Android views do not have identical layouts or every desktop-specific field.

The feed does not contain provider credentials, email addresses, account names, local paths, prompts, session transcripts or raw errors. Pairing secrets are protected using Windows DPAPI and Android Keystore. Requests use HMAC-SHA256 authentication, timestamp checks and single-use nonces; responses use AES-256-GCM with the request identity as authenticated data. The local transport is TCP with newline-delimited JSON, bounded requests/responses, timeouts and a concurrent-client limit.

## Distribution and coexistence

The owned repository is [lingmulongtai/Win-CodexBar-Mobile-Connect](https://github.com/lingmulongtai/Win-CodexBar-Mobile-Connect). Stable releases offer a direct **unsigned EXE**, SHA256SUMS and build provenance. Windows 10/11 x64 and WebView2 Runtime are required; Node.js and ZIP extraction are unnecessary. Updates are manual: quit, replace the EXE, relaunch. Upstream signing/installer/Winget credentials are not used.

The product identifier, settings directory, auto-start identity, toast identity, updater cache and support links are separate from the upstream app. Existing CodexBar Android companion pairings and Windows residents are preserved. After verifying the new app with your actual accounts and phone, you can decide which older residents to remove. Two apps collecting the same account concurrently may do redundant provider requests.

## Validation and limits

Native Windows proof used an isolated configuration with synthetic quota data, the actual Tauri app, and Android's actual Kotlin receiver. The downloaded, published v0.70.2 EXE also passed GUI checks for sharing on/off, Tailscale/LAN selection and persistence, and updates, with native screenshots. It is evidence for transport interoperability and the published GUI, not a real-account or physical-phone acceptance test. Unit tests cover authentication vectors, malformed messages, replay rejection, listener shutdown, canonical provider IDs, unknown values, source changes, stale observations and account-scoped history. See the [validation report](mobile-validation.md) for exact results.

There is no cloud relay, phone discovery, QR scanner, automatic firewall change or Google Gemini Apps collector. Pairing includes the endpoint; no provider token must be re-entered on the phone. Off-LAN physical-phone acceptance remains separate from local native/emulator checks.
