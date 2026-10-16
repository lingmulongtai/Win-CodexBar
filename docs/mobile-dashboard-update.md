# Mobile Connect dashboard update

Implemented on `codex/mobile-dashboard-history`, not published as a new stable release yet. The existing v0.70.6 download remains unchanged until publication. Android and Windows both need this update for the new shared data and controls.

## Changes

- **Settings → Mobile Connect** is a dedicated sidebar section. Listener state and failures stay visible; **Copy pairing code** is available while sharing runs. Listener readiness is distinguished from a confirmed phone connection.
- Windows persists encrypted quota history by opaque account/connection scope, retaining bounded 90-day archives and exporting up to 30 days to every paired phone/tablet. Switching accounts does not combine or erase the previous archive. New history begins with this update's observations; legacy provider-only burndown files are not assigned to an account by guesswork.
- Shared **Usage & Spend** totals use the same reporting period as the PC dashboard. Cost tracking must be enabled. Existing PC scans run at most once per five minutes for sharing, independently of device count. Current provider errors do not suppress independently available spend rows. Values can include multiple accounts, are reference usage costs, and are not subscription invoices.
- The additive v1 snapshot includes bounded daily spend/token totals and canonical quota samples. Unknown and verified-zero values remain distinct. The response budget sheds optional history before it can break basic quota sharing. No projects, paths, conversation contents or provider credentials are exported.
- **Allow settings changes from Android** defaults off. When enabled locally, paired devices can change only the fixed refresh interval and Usage & Spend period. A fixed interval disables Adaptive, matching the General page. Revisions prevent two devices from silently overwriting each other's changes. Disabling/revoking the listener invalidates old control handlers before shutdown.
- Settings operations use a separate authenticated v2 resource. Exact payload bytes and resource names are bound into HMAC authentication and AES-GCM response AAD. The existing v1 quota protocol remains compatible.
- The Android update adds long-press drag ordering, History/Spend tabs, tablet grids, thicker quota bars with time-budget guides and clearer reset notification permission/channel setup. New Android release assets use `CodexBar-Android-v<version>.apk` / `.aab`; older release names remain recognized by the updater.

## Native screen

Captured from the actual newly built Windows Tauri EXE with an isolated synthetic profile. It is a development build, not a screenshot of a published release. The installed resident and its credentials were retained. The copy action was exercised successfully while the listener ran. The code itself is never displayed in the image.

![Dedicated Mobile Connect pane](images/mobile-connect/windows-dashboard-mobile-connect.jpg)

![Android tablet PC spend view, emulator demo data](images/mobile-connect/android-dashboard-tablet-spend.png)

## Upstream and verification

Merged mainline `nesszer/Win-CodexBar` at `079b21a354ba55ec7757353792c8921f30dfa160` (328 upstream commits), preserving Gemini Apps renewal, Vivaldi, Mobile Connect, the owned settings namespace and updater/download links. Unmerged upstream feature branches were not included. Both repositories' `AGENTS.md` now require an upstream check before every development task.

Rust workspace tests: 4,841 core tests, 1 CLI test and 773 Tauri tests passed, with 2 pre-existing ignored tests. Frontend: 1,337 tests passed across 143 files; TypeScript/Vite and locale checks passed. Strict workspace/all-target Clippy passed. Oxlint completed with 11 existing upstream warnings in unrelated files. Android unit tests passed (426, plus 2 opt-in native tests skipped in the default run); debug lint/build passed, with 133 lint warnings and no errors. The two native tests were separately executed successfully, with no skips. Native Windows/Kotlin interoperability verified both quota decryption and two simultaneous settings updates: one succeeded and the other returned a conflict, then the original settings were restored.

An actual Android WorkManager job consumed the public synthetic Rust fixture. The baseline produced no reset alerts; a later background job produced four notifications (Codex reset, Codex reset-balance increase, Claude reset, Copilot reset). This proves the pipeline on the API 36.1 emulator. Android’s real PC settings dialog also changed the native EXE’s period to 7 days, reported success, and restored 30 days. Home-card drag order survived a force-stop/relaunch. Physical phone/tablet, real Tailscale off-LAN, OEM battery restrictions and Samsung Now Bar acceptance remain separate.

Validation uses only isolated demo settings. Windows remains an unsigned EXE. This task has not pushed branches, created a release, changed a firewall/VPN, restarted Gemini or replaced the installed resident.

## Fork commit sequence

| SHA | 件名 | 目的 |
| --- | --- | --- |
| `2c37e06d0` | Merge upstream main for spend trends and provider fixes | 上流の費用推移・プロバイダー修正を通常merge |
| `c5b12f645` | docs: require upstream review before fork development | フォーク機能を保持した毎回の上流確認を明記 |
| `42bd76875` | feat: give Mobile Connect a dedicated settings pane | 専用Mobile Connectタブと状態・コピー操作 |
| `398217f71` | feat: persist and share account-bound PC history and spend | アカウント別PC履歴保存、日別費用と背景集計 |
| `7b1aaad81` | feat: allow authenticated opt-in mobile settings changes | v2認証・許可制設定操作と同時変更の競合検出 |
| `b146a8bd2` | fix: keep upstream quota guards compatible with strict Clippy | 上流のクォータ防御式を厳格Clippyと両立 |
| `95595f62d` | test: isolate native proof runs and seed background reset fixtures | 常駐を残す検証起動と合成リセットfixture |
| `0be8d8a37` | fix: describe Gemini Apps accurately in sharing help | Gemini Apps / CLIの区別を共有画面にも反映 |
| `60e17e575` | fix: require a separate store before bypassing single-instance protection | 通常設定と異なる保存先でのみ検証の複数起動を許可 |
