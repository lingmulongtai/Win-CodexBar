# Google Gemini Apps quota investigation

Status on 2026-10-15: authenticated Web and native-app sessions both retrieved Usage Limits data. The production integration uses the separate `geminiapps` provider and Android `GEMINI_APPS` service; `gemini` remains Gemini CLI and `antigravity` remains independent. See [connection instructions](gemini-apps.md).

## Session stability follow-up — 2026-10-17

### Two-step verification follow-up — 2026-10-18

The user reports that the dedicated window remains on Google's loading state after completing two-step verification. The original anonymous-window proof did not cover this path. The currently active profile's locally inspected history is still on an `accounts.google.com` sign-in challenge; no full authentication URL, query, account data or cookie value was output. The native window reader timed out after retry and reinitialization, so this is not a successful visual inspection of the stalled page.

The original navigation guard silently canceled every top-level host other than `accounts.google.com` and `gemini.google.com`, and replaced new windows with a parent navigation. These are hypotheses, not an established cause: the actual rejected host/popup flow has not yet been observed. The follow-up adds host-only rejection diagnostics, explicit Finish recovery to Gemini in the same browser profile, and a DPAPI-protected unfinished-connection reference for restart recovery. A reference is durably revoked before committing the selected connection, and missing/committed profiles cannot be restored into an old history scope. No speculative list of extra authentication hosts has been enabled.

An explicit Chromium browser/profile import is also added, including Vivaldi. It reuses the strict Google-root scope, expiry, host-digest, duplicate rejection and authenticated verification logic. No multi-profile session merging or alternative-browser fallback occurs. The running Vivaldi Default profile's cookie database rejects ordinary read-only access on this PC. Its Local State has a normal encrypted-key entry and no app-bound-key entry, but the locked database's cookie encryption version could not be inspected. No browser was closed, unlocked or decrypted during this metadata-only check.

Frontend focused checks cover selected-profile import, failure preserving the previous selection and blocking a stale profile during a browser switch. Live recovery/import and authenticated lifetime remain pending until the rebuilt application actually retrieves quota. The currently published stable release was read back as v0.71.0; the installed v0.71.1 is a local validation candidate.

The user subsequently completed the connection in the **original installed v0.71.1**, before any follow-up EXE was built or installed. Its SHA-256 remains `129db3d8e527ceb7cd80ca2591639d18bf03380d7020a28e7b13e0651c247915`. Both quota windows and multiple distinct successful collection timestamps were verified, with the dedicated WebView2 browser running and no official Gemini process. This does not prove that the development recovery changes fixed the original stall: they are not running on this PC.

At the user's request, the working app is kept intact while a bounded private 24-hour monitor samples the authenticated mobile cache once per minute. The earlier waiting process had stopped and had not measured authenticated uptime. The replacement runs through a user-level Windows Scheduled Task with a logon trigger, bounded expiry and automatic task cleanup. Its actual scheduled execution, current-user SID and DPAPI read were verified. It does not restart the app, force refreshes, operate Google sign-in, wake the PC or generate chats.

The monitor binds observations to the selected browser connection and the actual TCP-listener process image, checks the listener identity before and after a response, and records process starts/restarts, source changes, stale quota and missing observations separately. A surviving installed file alone is not accepted as evidence of the serving executable. Private state survives observer/PC restart; quota collection timestamps must advance and waiting/cached display never counts as a fresh collection. Fixture checks cover stale cache, different login, changed executable, process restart, persisted start/deadline, offline gaps, expiry and fingerprint redaction. No automatic stability PASS is inferred from elapsed wall time.

The read-only upstream was checked at `1e97e289696fa496a9bec0d7a9c962371511b717`. Compatible fixes `15d55e000` (inherited update execution policy, including its test isolation fix) and `54f60275f` (six corrupted Korean messages) were ported separately. The remaining bulk update spans 237 files and conflicts with this fork's provider registration, Gemini CLI changes, locale keys and mobile bridge types while moving/deleting their original files. Integrating those structural changes requires preserving and validating the fork's new provider/spec/type entries; it is deferred from this urgent authentication fix rather than resolving conflicts by dropping fork features.

### Initial candidate evidence — 2026-10-17

The previously reported 50-minute run did not establish multi-hour stability: the saved production login subsequently expired. Two independent, normal-TLS probes of the selected session returned HTTP 200 from `RotateCookies`, with only CC cookie updates and no fresh `__Secure-1PSIDTS`. Both following `/app` bootstraps lacked authenticated CSRF state. No valid quota payload was obtained and no credentials were replaced. The installed Gemini 1.15.0 profile also contains a device-bound-session database; that supports investigating browser-bound authentication, but does not by itself prove why Google expired this particular session.

The development implementation now keeps authentication in a separate persistent WebView2 profile. A fixed script fetches bootstrap and quota on the Gemini origin, retaining cookies, device keys, CSRF and page HTML inside that browser. Only a size-bounded quota response crosses a nonce-bound, profile-bound, one-shot callback; malformed, expired and replayed callbacks are canceled. The remote window cannot invoke any app custom command (the guard checks the actual native WebView label and URL); it has no remote plugin capability either. Navigation is limited to HTTPS Gemini/Google Accounts. No browser security feature, app restriction, TLS verification or device binding is disabled.

Connection verification precedes selection commit. Cancel keeps the previous connection. Saving a new selection retires the previous browser profile; disconnect revokes before cleanup and invalidates in-flight data. Profile removal verifies the canonical UUID child under this app's own Gemini directory and reports busy files instead of claiming successful erasure. Legacy cookie fetches are serialized, checkpoint renewed credentials before later failures, and reject stale session writes.

Validation status:

- Gemini core regression tests: 22 passed, including renewal followed by quota failure, stale writes, invalid authentication, and the browser/parser boundary.
- Frontend: 143 files / 1,341 tests passed; the seven connection-control cases include pending/failed verification, cancel, and a revoked selection whose browser cleanup needs retrying. TypeScript, locale-key parity and production frontend build passed. Existing unrelated lint warnings remain.
- Full core tests: 4,846 library tests plus one CLI test passed; two existing tests ignored. Full Windows shell tests: 780 passed after default-account route validation was added. Browser-script tests: eight passed, including account switches during bootstrap. Strict Clippy passed both manifests with all targets and `-D warnings`.
- Native Windows proof used the rebuilt EXE from `d538c98a0` (SHA-256 `96903eb6ad75530136d5f9b3c3f1904a7987683c4a69a9575cfcbfe29fa10df1`). Main and Google WebViews ran in different profiles. The real Google WebView was denied a custom settings read, denied the same read after spoofing both JavaScript window labels, and denied event-plugin access. Closing the Google window hid it while retaining its browser. Two unauthenticated verification attempts were correctly rejected in 304 ms and 281 ms; no selection was saved. Cancellation removed the complete profile directory and its browser process, including a synthetic cookie and local-storage entry. Production settings, manual credentials, renewal cache and mobile pairing were byte-for-byte unchanged. [Native screenshot](images/mobile-connect/windows-gemini-browser-development-ja.png).
- The production-shaped, static-runtime **v0.71.1 Release EXE** from `5b8ffbcbe` also passed the native checks: separate profiles, direct/spoofed custom-command denial, plugin denial, unauthenticated rejection (239 ms), and cancellation removing the profile plus synthetic cookie/storage. Its SHA-256 is `129db3d8e527ceb7cd80ca2591639d18bf03380d7020a28e7b13e0651c247915`. The PE verifier confirmed x64 GUI, version 0.71.1 and no external VC++ runtime or WebView2 loader DLL dependencies. The screenshot linked above was refreshed from this exact EXE.
- That exact candidate replaced this PC's resident executable at its existing installation path and was started with the real settings, no synthetic usage and no remote debugging. The old EXE was backed up privately. All four protected settings/credential/cache/pairing files remained byte-for-byte unchanged and the existing mobile listener resumed.
- At the October 17 handoff, authenticated multi-hour collection had **not started** and the old observer was waiting for sign-in. That waiting time never counts as authenticated uptime. The October 18 follow-up above records the first live verification and replacement monitor. Authenticated app-restart and sleep/resume acceptance remain pending. Elapsed wall time, cached snapshots and observer exit success alone are not a pass; unique collection timestamps, failures and gaps must be reviewed.

Upstream was fetched read-only before the work: `nesszer/Win-CodexBar/main` at `1e0a414cc10c6429a106f4674637427333518afa` was merged normally (off-machine proof guidance only). The Android upstream was checked at `1811d1fe032fd4a80240fd6bc5d04cee649a0afb`, with no new change to incorporate. Android code and its existing release are unchanged.

## Verified acquisition evidence

- The installed Google-signed Windows app is Gemini 1.14.0, packaged with Electron. Its host loads `https://gemini.google.com/app`.
- The packaged native bridge exposes app/window/device operations, but no quota method was found. Production startup explicitly rejects remote debugging and inspector arguments; these protections were preserved.
- One user-approved restart with Chromium's `--force-renderer-accessibility` flag completed. The available native reader still exposed no readable quota text.
- In a logged-in Vivaldi Web tab for the same Google account, Settings → Usage limits displayed current and weekly percentages and reset times. Opening that view requested the RPC `jSf9Qc`; its fraction values and Unix reset timestamps matched the visible values.
- The app's running `Network/Cookies` database rejects ordinary read access with a Windows sharing violation. A second user-approved close/read/restart succeeded. Gemini was restarted normally before HTTPS requests. Its DPAPI-protected Local State key decrypted the AES-GCM session cookies, and the app session returned the same current/weekly quota fields. No lock, process-debugging restriction or encryption boundary was bypassed; the experiment did not persist decrypted credentials.

## Observed Web protocol

The first-party HTTPS endpoint is:

```text
POST https://gemini.google.com/_/BardChatUi/data/batchexecute
rpcids=jSf9Qc
source-path=/app
rt=c
```

The form has `f.req` and `at` fields. The observed quota call argument is the empty JSON array `[]`; `at` is a session CSRF token and must remain private. The response is XSSI-prefixed, line-framed JSON. Select only a `wrb.fr` row whose RPC ID is `jSf9Qc`, then parse its JSON payload string.

```text
payload = [planCode, windows, flag]
window = [unclassifiedNumber, usedFraction, windowType, [[resetUnixSeconds, nanos]]]
windowType 1 = Current usage
windowType 2 = Weekly limit
```

Map `usedFraction * 100` to used percentage. Preserve the exact reset instant. The leading number is not established as a duration, prompt count, token count or quota capacity; do not display it as any of those. A missing or changed schema must report unavailable rather than invent a zero quota. Reject invalid fractions, duplicate known window types and invalid timestamps. Permit a known current window without assuming that every plan supplies a weekly window.

No raw Cookie header, CSRF token, request/session identifier, HTML, account identifier, chat text or network dump is included in this document or repository.

## Integration boundary

Use a new `geminiapps` provider and an explicit Android `GEMINI_APPS` service. Keep CLI and Antigravity observations separate. The Windows-to-Android channel must carry only the existing sanitized quota fields and collection timestamp, never Google credentials.

Google authentication includes parent-domain cookies. A `gemini.google.com` child-domain extraction alone misses those; broad collection of every Google subdomain is also inappropriate. A provider-local reader must restrict the permitted cookie hosts and paths to the requests being made, preserve App-Bound Encryption limits and fail without switching to another account or browser.

Bootstrap and quota requests use the same default `/app` account route and reject redirects. Multi-account/index selection is deliberately not offered. Import and disconnect invalidate cached/mobile usage and supersede old in-flight requests. No stable source account identity is established, so the mobile subject ID and identity-dependent reset-history tracking are omitted.

The Web RPC is undocumented and may change. A collector needs deterministic framing/schema fixtures, a real authenticated fetch matching the screen, login/expiry errors and account-isolation checks before release.

## Primary references

- [Google Gemini desktop apps](https://support.google.com/gemini/answer/18263854?hl=en-12)
- [Google Gemini Apps limits and Usage Limits display](https://support.google.com/gemini/answer/16275805?hl=en)
- [Chromium Windows accessibility](https://new.chromium.org/developers/accessibility/windows-accessibility/)
- [Existing mobile sharing contract](android-sharing.md)

## Stable-release preparation follow-up — 2026-10-18

The user stopped the separate scheduled recorder because its console window interrupted work. The task was removed and is not part of the app or release. Do not restart this recorder. A bounded, one-off pre-update read again verified fresh quota from the same original selected browser connection; no Google sign-in automation was performed. The previously planned 24-hour measurement was not completed.

The release also fixes the 56-point in-process history truncation and time-based canonical archive retention. See the v0.71.1 release notes for current source and artifact validation; the earlier candidate-only statements above describe the historical checks on their listed hashes. The original post-2FA loading cause remains unproven.
