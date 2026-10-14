# Google Gemini Apps quota investigation

Status on 2026-10-15: authenticated Web and native-app sessions both retrieved Usage Limits data. The production integration uses the separate `geminiapps` provider and Android `GEMINI_APPS` service; `gemini` remains Gemini CLI and `antigravity` remains independent. See [connection instructions](gemini-apps.md).

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
