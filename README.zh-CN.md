# Win-CodexBar + Mobile Connect

此分支在[Win-CodexBar](https://github.com/nesszer/Win-CodexBar)基础上增加加密Android连接。Windows采集使用量，手机接收数据；原有采集器、托盘和桌面界面保留。

**[直接下载Windows EXE](https://github.com/lingmulongtai/Win-CodexBar-Mobile-Connect/releases/latest/download/Win-CodexBar-Mobile-Connect.exe)** · [正式版本与校验和](https://github.com/lingmulongtai/Win-CodexBar-Mobile-Connect/releases/latest) · [签名Android APK](https://github.com/lingmulongtai/CodexBar-android/releases/latest/download/app-release.apk)

支持Windows 10/11 x64，需要[WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/)。无需ZIP解压或Node.js。此分支EXE无数字签名，请核对SHA256SUMS；更新时退出应用并替换EXE。

在Advanced → Android sharing中选择LAN或Tailscale，启用共享，将配对码粘贴到Android的Connections。两台设备在同一Tailscale网络即可，手机无需exit node。默认TCP端口43825，访问规则与防火墙需允许通信。

Android 0.9.7可选择手机采集或Windows采集，并设置个别服务的手机例外。配对、凭据、选择和分别保存的历史记录在切换后保留。PC离线时不自动改用手机的另一个账号。新增PC本地日志的今日、30天与所选期间的Token数和USD估算费用，可能包含多个账号，并非实际账单。**Gemini指Gemini CLI，不是Gemini Apps。**

![Windows共享设置实拍](docs/images/mobile-connect/windows-release-v0.70.2-sharing-ja.jpg)

![已发布v0.70.2的应用信息与更新检查](docs/images/mobile-connect/windows-release-v0.70.2-about-ja.jpg)

完整说明、Android截图与开发步骤见[English README](README.md)、[日本語](README.ja-JP.md)和[设置指南](docs/android-sharing.md)。截图使用测试数据；验证范围见[记录](docs/mobile-validation.md)。[MIT](LICENSE)；原始项目来自[CodexBar](https://github.com/steipete/CodexBar)。
