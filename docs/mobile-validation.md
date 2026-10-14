# Mobile Connect validation

## Gemini Apps update (v0.70.5 / Android v0.9.8)

Gemini Apps now has its own Windows collector and Android provider identity. The earlier unsupported-Gemini statements below describe older releases. See [Gemini Apps setup](gemini-apps.md) and [v0.70.5 validation and limits](releases/v0.70.5.md) for current behavior. The native connection screen uses the existing styled credential buttons and does not offer an unsupported account switch action. The saved Windows-user DPAPI session was verified in the native Rust and Tauri collectors while Gemini was running; v0.70.5 repairs the HTTP/2 bootstrap failure observed in v0.70.4.

## Mobile Connect v0.70.2 local checks

- Locked Rust workspace tests: 4,178 core tests passed, one ignored; one CLI test and 715 Tauri tests passed. Rust formatting and workspace clippy with warnings denied passed.
- Frontend: the full run passed 1,102 tests; eight Advanced-settings fixture failures were corrected and all eight reran successfully, covering 1,110 tests. TypeScript/Vite and locale checks passed. Lint retained seven existing warnings and no errors. Hosted CI runs the complete suite again on the final source.
- The downloaded, published v0.70.2 EXE passed the native GUI verification described below. Its own Windows.Graphics.Capture screenshots replace the earlier implementation-build images in every active README.
- The actual native Windows listener and Kotlin receiver exchanged authenticated encrypted synthetic quota and local-log totals: 12,500 tokens today, 250,000 over 30 days, estimated USD 0.25/5.25. Both loopback and a detected Tailscale-address listener passed the forced native interoperability test.
- Android v0.9.7 API 36.1 preserved the existing pairing across debug APK updates; its normal dashboard displayed three Windows sources. Phone mode stopped automatic PC polling and retained pairing. Windows mode restored PC sources. Native screenshots include both modes and PC detail totals.
- Android unit tests: 408 passed, one environment-dependent native interop test skipped in the regular suite; that test separately passed against the native Windows server. Lint and debug APK build passed.
- The updated Android landing page passed its asset/link validator and browser checks at 390px: both new screenshots loaded and no horizontal overflow was present.

CUA Driver is unavailable on this host. The bundled Computer Use capture uses Windows.Graphics.Capture for real native pixels; CDP drives only the app's own WebView2. Android images are actual ADB captures, not reconstructed UI. Proof profiles and pairing codes remain outside Git.

## Published EXE GUI verification

On 2026-10-14 (Asia/Tokyo), the exact EXE downloaded from the stable v0.70.2 release was launched with an isolated proof profile. Its SHA-256 is `7c389d63d289631fc79ac5ca1b0ad41f73889e92a781323de74b46f59cdd7db0`; release provenance binds it to source commit `97577a553954d4b3ac45d03e4d4d862b196213c0`. This is the published binary, not a local rebuild. The earlier Windows network permission dialog closed when the previous test process exited; the new launch and captures have no permission overlay. No access grant, firewall or VPN change was needed.

- The native window and app information show **Win-CodexBar + Mobile Connect**, version **0.70.2**, stable channel.
- Clicking the sharing toggle in the real WebView2 GUI started and stopped the native listener. The final capture shows sharing active on the isolated loopback endpoint, port 43825.
- The detected Tailscale and LAN buttons selected the corresponding discovery result and saved it. Reloading retained the Tailscale selection; saved native state also matched LAN. Endpoint selection was checked with sharing stopped; off-LAN phone connectivity remains outside this check.
- The app information screen completed **Check for updates** without an error and displayed the latest-version result. The loaded published About module contains the owned repository/README website target.
- Native captures show the complete sharing controls and app information. Neither capture contains a pairing code, real account identity or the host's LAN/Tailscale address. The rendered pages have no horizontal overflow.

CDP drove the app's own observed WebView2 target and checked persisted/native state. The bundled Computer Use API captured the real Windows window using Windows.Graphics.Capture because CUA Driver is not installed. Images were saved directly from those captures without reconstructing or editing the UI. This completes the previously unfinished published-EXE GUI capture. Tray-menu and physical-phone checks below remain separate.

![Published v0.70.2 native Windows sharing settings](images/mobile-connect/windows-release-v0.70.2-sharing-ja.jpg)

![Published v0.70.2 app information after update checking](images/mobile-connect/windows-release-v0.70.2-about-ja.jpg)

## Portable EXE gate correction

The downloaded PR EXE passed x64 GUI/product/version inspection and uses only Windows 10/11 OS UCRT API sets. It has no VCRUNTIME/MSVCP/WebView2Loader imports. The original v0.70.1 gate incorrectly rejected these OS components; v0.70.2 distinguishes them from redistributable and debug DLLs. Dependency-free allow/reject tests and inspection of that actual EXE passed. +crt-static stays enabled, while documentation avoids claiming that every OS CRT API is embedded. [Microsoft UCRT deployment](https://learn.microsoft.com/en-us/cpp/windows/universal-crt-deployment).

## Limits and release checks

Synthetic fixtures prove projection, encryption, receiver interoperability and presentation. They do not prove live-account collection, off-LAN physical-phone Tailscale connectivity, physical launchers, tray-menu pixels or Samsung Now Bar. Google Gemini Apps is unsupported; the inherited collector is Gemini CLI. Tailscale access rules and Windows Firewall must allow the selected endpoint. No system routing or firewall settings were changed.

The stable workflow binds the tag to the exact owned-main commit, checks versions, builds an x64 GUI EXE without external VC++ runtime or WebView2 loader DLL dependencies, verifies product/version/imports and hashes, then publishes a normal latest release. Windows is unsigned and updates are manual. Android keeps the pinned production certificate. Hosted release results and downloaded assets require independent verification; local debug builds are not release artifacts.

## Implementation commits and purposes

| Commit | Purpose |
| --- | --- |
| `8b984a94d` | fix: permit Windows system UCRT in portable EXE verification |
| `b86a096cd` | chore: prepare the verified portable release as version 0.70.2 |
| `427c912b` | docs: publish Mobile Connect setup and native screen evidence |
| `034a8fa9` | feat: discover Tailscale sharing addresses without exit nodes |
| `064abfac` | feat: select Tailscale or LAN sharing endpoints in settings |
| `a342b1c0` | chore: identify the Mobile Connect fork and owned links |
| `c25380eb` | feat: share sanitized PC token and cost totals with Android |
| `2da936e7` | test: seed local log totals only in validated native proof mode |
| `ee86e614` | fix: open the Mobile Connect project website from About |
| `fbdbc0c6` | fix: display only the selected sharing endpoint |
| `071084a3` | chore: prepare Mobile Connect version 0.70.1 |
| `7d29e467` | ci: publish verified stable Windows EXE releases |
| `312741a9` | test: provide network discovery in Advanced settings fixtures |
| `6606eba0` | fix: release the mobile listener before draining clients |
| `7c551114` | fix: translate sharing controls in every supported locale |

---

## Historical preview evidence

# Mobile Preview 1 validation

The preview was built from the owned `codex/android-bridge` branch on Windows using locked dependencies. The actual optimized Tauri executable was launched with a separate proof profile and a synthetic Codex observation (25% used). Existing Windows residents and real-account pairings were preserved.

## Results

- `cargo test --locked --workspace`: 4,178 core tests passed, one ignored; one CLI test and 711 Tauri tests passed.
- `cargo fmt --all -- --check` and focused core/Tauri clippy with warnings denied passed.
- All 1,108 frontend Vitest tests passed. TypeScript/Vite build, locale consistency and lint passed; lint retained seven existing warnings.
- `pnpm run tauri:build`: optimized native executable built successfully. The portable local build is unsigned; its SHA-256 is `c1334fc2143be6189ace060ca0546c8b8bfa068a629771088e2a773f4398b77d`.
- The actual Windows listener returned an authenticated encrypted snapshot to the Kotlin receiver's native interop test.
- Android API 36.1 successfully paired through the real Connections UI, stored the pairing with Android Keystore, and displayed the Windows observation as **75% left** in the normal Android dashboard. The emulator used its `10.0.2.2` host route; an incidental ADB reverse mapping was removed before dashboard verification.
- Updating the debug APK preserved the saved pairing. Restarting the optimized Windows executable preserved its DPAPI-protected pairing and enabled listener. The private pairing code was absent from the visible page.
- Revoking the pairing rejected the old key and accepted the new key. Disabling sharing from the actual UI while an invalid address draft remained closed the TCP listener and retained the last valid saved address.

CUA Driver was unavailable. The bundled Computer Use plugin captured actual Windows pixels through Windows.Graphics.Capture; CDP assertions targeted the native WebView2 application's observed debugging endpoint. The image below is from the freshly rebuilt optimized executable, not a browser recreation.

![Japanese Android sharing settings in the optimized native Windows app](images/mobile-preview/windows-sharing-ja.jpg)

## Review and practical limits

Two independent read-only reviews found and corrected stale source publication, queued credential changes, unknown quota windows, cross-language bounds, canonical provider IDs, failure caching, reset/history scope and fork identity issues. CI's temporary path alias mismatch was fixed by giving Windows fixtures a consistent long temporary path.

Synthetic data validates the transport and UI without exposing accounts. Physical-phone pairing, real-account collection, tray menu pixels, Niagara Launcher and Samsung Now Bar remain unverified. Google Gemini Apps usage is not collected; the inherited Gemini collector is Gemini CLI. Preview updates are manual and the Windows executable is unsigned.

## Commit purposes

| Commit | Purpose |
| --- | --- |
| `10265e23` | Bind GitHub mutations to the owned fork |
| `fceffe70` | Authenticate and encrypt bounded Android snapshots |
| `11291ca7` | Separate fork configuration and updater ownership |
| `5ed26d87` | Keep current Rust compiler checks passing |
| `f0938d30` | Verify request authentication with an independent vector |
| `10ee602f` | Project only verified desktop observations |
| `bbffd04c` | Add Android sharing controls to Advanced settings |
| `7c77fda4` | Export canonical provider IDs |
| `17b3486b` | Separate notification, cache and support identities |
| `f67800e2` | Build owned previews without upstream signing |
| `0e2c72b0` | Document setup and preview limitations |
| `cb2227ef` | Normalize CI temporary paths and avoid duplicate branch runs |

The screenshot and this report are a separate evidence commit. Hosted CI and release asset verification are recorded on the PR/release, independently of these local checks.
