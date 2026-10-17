# Google Gemini Apps quota investigation

Status on 2026-10-15: authenticated Web and native-app sessions both retrieved Usage Limits data. The production integration uses the separate `geminiapps` provider and Android `GEMINI_APPS` service; `gemini` remains Gemini CLI and `antigravity` remains independent. See [connection instructions](gemini-apps.md).

## Session stability follow-up — 2026-10-17

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
- Authenticated multi-hour collection has **not started**; an initial interactive Google sign-in is required. Mock tests, anonymous native tests and time spent waiting for login never count as elapsed authenticated uptime. Authenticated app-restart and sleep/resume checks remain pending too. A private 24-hour observer is running in `waiting-for-authentication`, with zero elapsed authenticated seconds. It records only sanitized mobile-cache status, quota fractions and real collection timestamps. Before starting the duration clock, it requires a fresh ready quota and verifies that its mobile history scope belongs to the currently selected persistent-browser connection. Elapsed wall time, cached snapshots and observer exit success alone are not a pass; unique collection timestamps, failures and gaps must be reviewed.

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
