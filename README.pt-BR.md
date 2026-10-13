# Win-CodexBar + Mobile Connect

Monitor de uso de IA para Windows com compartilhamento criptografado para o CodexBar no Android. Fork independente de [Win-CodexBar](https://github.com/nesszer/Win-CodexBar), baseado em [CodexBar](https://github.com/steipete/CodexBar).

**[Baixar EXE](https://github.com/lingmulongtai/Win-CodexBar-Mobile-Connect/releases/latest/download/Win-CodexBar-Mobile-Connect.exe)** · [Versões e SHA256SUMS](https://github.com/lingmulongtai/Win-CodexBar-Mobile-Connect/releases/latest) · [APK Android assinado](https://github.com/lingmulongtai/CodexBar-android/releases/latest/download/app-release.apk)

Requer Windows 10/11 x64 e Microsoft WebView2 Runtime. Salve e execute o EXE; não é preciso extrair ZIP nem instalar Node.js. O EXE deste fork **não é assinado**. Para atualizar, encerre o aplicativo e substitua o EXE.

Em **Avançado → Compartilhar com Android**, selecione LAN ou Tailscale, ative o compartilhamento e cole o código privado em Conexões no Android. Os dispositivos precisam estar na mesma rede Tailscale, com acesso permitido. O telefone não precisa de exit node. O PC deve continuar ligado. O aplicativo não altera a VPN nem o firewall.

O Android oferece coleta no telefone ou recepção dos dados coletados no Windows, com exceções por serviço. A troca preserva credenciais, pareamento e históricos por conta e origem. Serviços conhecidos do PC são sincronizados ao atualizar. Se o PC estiver offline, não há troca automática para outra conta do telefone.

Os detalhes mostram tokens e custos estimados em USD dos logs locais do PC, quando disponíveis: hoje, 30 dias e período selecionado. Podem incluir várias contas e não representam cobranças. A data e hora da agregação são exibidas. **Gemini significa Gemini CLI; o uso do Google Gemini Apps não é coletado.**

![Configuração real do Windows](docs/images/mobile-connect/windows-sharing-ja.jpg)

[Guia de conexão](docs/android-sharing.md) · [Mudanças v0.70.1](docs/releases/v0.70.1.md) · [Validação e limites](docs/mobile-validation.md) · [README completo em inglês](README.md) · [Problemas deste fork](https://github.com/lingmulongtai/Win-CodexBar-Mobile-Connect/issues)

Capturas do Windows nativo e do emulador Android usam dados sintéticos. A validação em telefone físico e com contas reais permanece pendente. [Licença MIT](LICENSE).
