# Gemini Apps connection

Gemini Apps (`geminiapps`) reads the **Current usage** and **Weekly limit** shown in Google's Windows/web app, including each reset time. Gemini CLI (`gemini`) and Antigravity remain separate providers. These are compute-based quota percentages, not token counts or model-specific allowances.

## Connect on Windows

The local v0.71.1 release candidate adds a **dedicated persistent Gemini browser**. This is being validated; it is not yet part of the published v0.71.0 EXE. Imported desktop cookies can expire even when Google's renewal endpoint returns HTTP 200. See the [candidate's checks and remaining live verification](releases/v0.71.1.md).

### Development branch: connect from your browser

The development branch adds an explicit browser/profile connection in **Settings → Providers → Gemini Apps**. This option is not installed on the currently monitored PC yet. Choose your signed-in browser (including Vivaldi), choose its profile, and click **Connect from this browser**. Only that profile's root Google/Gemini cookies are read; profiles and browsers are never merged or substituted. The usage API must authenticate before the saved connection is replaced. A failed import keeps the previous selection.

If the browser exclusively locks its cookie database, close that browser once and retry. App-bound encrypted cookies report unsupported instead of being bypassed. This is a one-time copy: Google may expire it later, so it does not establish long-duration stability on its own.

### Connect in the dedicated browser

1. In **Settings → Providers → Gemini Apps**, enable the provider and choose **Open dedicated Google sign-in**.
2. Sign in yourself in the dedicated **Gemini — Win-CodexBar** window. Google may request two-factor authentication.
3. Back in CodexBar settings, choose **Finish connecting**. The old connection remains selected until an authenticated quota response has been verified and the replacement saved. Opening the login window alone never marks the provider connected.
4. The login window is hidden after connection. Closing its window also hides it; keep CodexBar running for background refreshes. After restarting CodexBar, the same browser profile is opened again.
5. Use **Disconnect Gemini Apps** to revoke the selection and clear this app's Gemini browser profiles. If Windows is still releasing profile files, an error requests cleanup again; the selected connection remains revoked.

After completing two-step verification, return to CodexBar and choose **Finish connecting**. The development branch additionally tries returning a stuck Google page to Gemini in the same profile before verification, and retains unfinished sign-in across app restarts. Completed, canceled or deleted profiles cannot resume as new accounts under their former history scope. Rejected navigation is diagnosed by host only. These recovery changes are not installed during the current live monitor. The original candidate eventually connected without them, so the original stall's cause remains unproven.

Google cookies, device-bound session keys, CSRF tokens and page HTML stay inside the isolated WebView2 profile. The native collector receives only a bounded quota response from a request inside the Gemini origin. Google pages cannot call the desktop application's custom commands or plugins. Profiles are separate from your normal browsers and the Gemini desktop app; there is no automatic account import or app restart. A global `WEBVIEW2_USER_DATA_FOLDER` override must be removed because it defeats profile isolation.

Only an opaque connection identifier is stored in CodexBar's existing encrypted credential-selection file. Browser-driven cookie renewal does not change this identifier or split mobile history into a new connection. The quota API does not establish a verified Google account ID. To change accounts, explicitly reconnect in a new dedicated profile.

### Legacy desktop import

**Import from desktop app (legacy)** retains the old explicit desktop-cookie import. Close Google's Gemini app once before importing because its database is locked while it runs. This path can lose access after a few hours and is not the recommended persistent connection. Only matching, unexpired root-path Google/Gemini cookies are read. Their values are encrypted for your Windows user by the existing DPAPI credential store, never displayed, and never sent to Android.

Renewed cookies use a separate DPAPI-protected `geminiapps_session.json`, bound to the exact connection you selected. Background renewal keeps the account-selection file unchanged, so successful refreshes stay available to Android. Disconnect and credential revocation remove the renewal cache; an older request cannot restore it after a reconnect. Only the two Google authentication cookies needed for renewal are sent to the fixed Google Accounts endpoint. Redirects are never followed.

Renewed cookies are checkpointed before subsequent requests, so a failed quota request cannot discard an already renewed session. Collection is serialized and accepted writes advance the session generation, preventing a late response from overwriting newer authentication. HTTP 200 alone does not mean the session authenticated: the collector requires the authenticated current-usage payload. A changed bootstrap schema reports a fetch error separately from an explicit sign-in requirement. Google can still revoke any login or require sign-in again; the app cannot guarantee indefinite access after a server-side revocation.

This version supports the default `/app` Google account route. Selecting among multiple Google accounts is not supported. Indexed/other-origin redirects fail instead of silently changing accounts. App-bound encrypted sessions cannot be imported.

## Phone and tablet

Use Android **v0.9.8 or later**, pair Windows in **Connections → Acquisition**, and choose Windows mode. `Gemini Apps` appears separately from `Gemini CLI`, with current/weekly percentages and resets. Phone-only acquisition does not fetch Gemini Apps and never substitutes CLI data.

Pair your phone and tablet using the same Windows pairing code. Each device keeps its own mode and display settings. Tailscale works without an exit node when both devices can reach the PC's selected Tailscale address and TCP port 43825. Revoking Windows pairing invalidates both devices' shared code; pair them again afterward.

## Data and verification limits

The undocumented first-party quota RPC can change. Missing/invalid windows or timestamps report an error; unknown data does not become 0%. The API supplies no verified stable Google account identity, token totals, model-level breakdown or established plan-name mapping. Quota history is scoped to the explicitly selected connection, without presenting it as a verified account identity. Reconnecting starts a separate connection scope; browser cookie rotation does not.

The legacy import previously passed approximately 50 minutes of live collection on 2026-10-15, but subsequent real use expired; that result does not prove multi-hour stability. On 2026-10-17, both minimal and full authentication-cookie renewal probes returned HTTP 200 without issuing a fresh `__Secure-1PSIDTS`; the following bootstrap was unauthenticated. On 2026-10-18 the user completed the dedicated-browser connection in the original v0.71.1 candidate, and real quota plus successive collection timestamps were verified while the official Gemini app was not running. A bounded 24-hour local monitor now checks fresh quota, the same connection and actual serving executable, process restarts and observation gaps. The running app is being preserved; new source changes are not installed. The 24-hour result, controlled authenticated restart and sleep/resume acceptance remain pending. [Current investigation and evidence](gemini-apps-investigation.md).

[Acquisition evidence](gemini-apps-investigation.md) · [Mobile sharing](android-sharing.md) · [Google's Usage Limits help](https://support.google.com/gemini/answer/16275805?hl=en)
