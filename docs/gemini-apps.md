# Gemini Apps connection

Gemini Apps (`geminiapps`) reads the **Current usage** and **Weekly limit** shown in Google's Windows/web app, including each reset time. Gemini CLI (`gemini`) and Antigravity remain separate providers. These are compute-based quota percentages, not token counts or model-specific allowances.

## Connect on Windows

Use Windows **v0.70.6 or later**. Version 0.70.5 fixed a bootstrap transport error, but did not retain Google's session renewals and could lose access after a successful connection.

1. Install the official Google Gemini desktop app and sign in with your primary Google account.
2. Close Gemini once. It locks its session database while running.
3. In **Settings → Providers → Gemini Apps**, enable the provider and choose **Connect using Gemini desktop app**.
4. Wait for successful verification, then reopen Gemini. Later refreshes retain new cookies from Google's bootstrap/quota responses and attempt Google session renewal at startup and at most once per ten minutes, while Gemini remains open.
5. To remove this connection, use **Disconnect Gemini Apps**. If Google expires the session, sign in again and repeat the import.

The connection checks the usage API before replacing a saved session. Only matching, unexpired root-path Google/Gemini cookies are read. Their values are encrypted for your Windows user by the existing DPAPI credential store, are never displayed in the settings UI, and are never sent to Android. No process debugging, encryption bypass, Gemini modification or automatic app restart is used.

Renewed cookies use a separate DPAPI-protected `geminiapps_session.json`, bound to the exact connection you selected. Background renewal keeps the account-selection file unchanged, so successful refreshes stay available to Android. Disconnect and credential revocation remove the renewal cache; an older request cannot restore it after a reconnect. Only the two Google authentication cookies needed for renewal are sent to the fixed Google Accounts endpoint. Redirects are never followed.

If the connection was already expired in v0.70.5, update and connect once more. HTTP 200 alone does not mean the session authenticated: the collector requires the authenticated current-usage payload. A changed bootstrap schema reports a fetch error separately from an explicit sign-in requirement. Google can still revoke a login or require sign-in again.

This version supports the default `/app` Google account route. Selecting among multiple Google accounts is not supported. Indexed/other-origin redirects fail instead of silently changing accounts. App-bound encrypted sessions cannot be imported.

## Phone and tablet

Use Android **v0.9.8 or later**, pair Windows in **Connections → Acquisition**, and choose Windows mode. `Gemini Apps` appears separately from `Gemini CLI`, with current/weekly percentages and resets. Phone-only acquisition does not fetch Gemini Apps and never substitutes CLI data.

Pair your phone and tablet using the same Windows pairing code. Each device keeps its own mode and display settings. Tailscale works without an exit node when both devices can reach the PC's selected Tailscale address and TCP port 43825. Revoking Windows pairing invalidates both devices' shared code; pair them again afterward.

## Data and verification limits

The undocumented first-party quota RPC can change. Missing/invalid windows or timestamps report an error; unknown data does not become 0%. The API supplies no verified stable Google account identity, token totals, model-level breakdown or established plan-name mapping. Account-dependent history/reset tracking is therefore omitted from the mobile Gemini Apps observation.

The native app session and quota RPC were verified on 2026-10-15. After the user approved encrypted storage, the Rust collector successfully reused the saved Windows-user DPAPI session while Gemini was running. Gemini's bootstrap rejected the client's HTTP/2 stream; the Gemini Apps collector now uses HTTP/1.1 with normal TLS verification. Deterministic tests cover framing, changed schemas, fractions, zero usage, window ordering, timestamps, scoped cookie extraction and separate CLI/Apps IDs. Native GUI screenshots use an isolated synthetic profile; they do not expose Google account information or chats. Published-executable verification is recorded in the release notes.

[Acquisition evidence](gemini-apps-investigation.md) · [Mobile sharing](android-sharing.md) · [Google's Usage Limits help](https://support.google.com/gemini/answer/16275805?hl=en)
