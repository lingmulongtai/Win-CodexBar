# Win-CodexBar + Mobile Connect

此分支在[Win-CodexBar](https://github.com/nesszer/Win-CodexBar)基礎上增加加密Android連線。Windows收集使用量，手機接收資料；原有收集器、系統匣與桌面介面保留。

**[直接下載Windows EXE](https://github.com/lingmulongtai/Win-CodexBar-Mobile-Connect/releases/latest/download/Win-CodexBar-Mobile-Connect.exe)** · [正式版本與校驗和](https://github.com/lingmulongtai/Win-CodexBar-Mobile-Connect/releases/latest) · [簽章Android APK](https://github.com/lingmulongtai/CodexBar-android/releases/latest/download/app-release.apk)

支援Windows 10/11 x64，需要[WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/)。不需ZIP解壓或Node.js。此分支EXE無數位簽章，請核對SHA256SUMS；更新時結束應用程式並替換EXE。

在Advanced → Android sharing選擇LAN或Tailscale，啟用共用，將配對碼貼到Android的Connections。兩部裝置在同一Tailscale網路即可，手機不需exit node。預設TCP連接埠43825，存取規則與防火牆需允許通訊。

Android 0.9.7可選手機收集或Windows收集，並設定個別服務的手機例外。配對、憑證、選擇和分別儲存的歷史記錄在切換後保留。PC離線時不自動改用手機的另一個帳號。新增PC本機記錄的今日、30天與所選期間Token數和USD估算費用，可能包含多個帳號，並非實際帳單。**Gemini指Gemini CLI，並非Gemini Apps。**

從v0.70.3起，可單獨選擇Vivaldi，匯入Windows標準設定檔中的Cookie。[設定與加密限制](docs/COOKIES.md)。

![Windows共用設定實拍](docs/images/mobile-connect/windows-release-v0.70.2-sharing-ja.jpg)

![已發布v0.70.2的應用程式資訊與更新檢查](docs/images/mobile-connect/windows-release-v0.70.2-about-ja.jpg)

完整說明、Android截圖與開發步驟見[English README](README.md)、[日本語](README.ja-JP.md)和[設定指南](docs/android-sharing.md)。截圖使用測試資料；驗證範圍見[記錄](docs/mobile-validation.md)。[MIT](LICENSE)；原始專案來自[CodexBar](https://github.com/steipete/CodexBar)。
