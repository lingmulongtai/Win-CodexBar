# Win-CodexBar + Mobile Connect

Monitor de uso de IA para Windows con conexión cifrada a CodexBar para Android. Fork independiente de [Win-CodexBar](https://github.com/nesszer/Win-CodexBar), basado en [CodexBar](https://github.com/steipete/CodexBar).

**[Descargar EXE](https://github.com/lingmulongtai/Win-CodexBar-Mobile-Connect/releases/latest/download/Win-CodexBar-Mobile-Connect.exe)** · [Versiones y SHA256SUMS](https://github.com/lingmulongtai/Win-CodexBar-Mobile-Connect/releases/latest) · [APK firmado para Android](https://github.com/lingmulongtai/CodexBar-android/releases/latest/download/app-release.apk)

Windows 10/11 x64 con Microsoft WebView2 Runtime. Guarda y ejecuta el EXE; no necesita ZIP ni Node.js. El EXE de este fork **no está firmado**. Para actualizar, cierra la aplicación y reemplaza el archivo.

En **Avanzado → Compartir con Android**, selecciona LAN o Tailscale, activa la conexión y pega el código privado en Conexiones de Android. Ambos dispositivos deben estar en la misma red Tailscale y tener acceso permitido; el teléfono no necesita exit node. Windows debe seguir encendido. La aplicación no cambia el firewall ni la VPN.

Android permite recopilar en el teléfono o recibir los datos de Windows, con excepciones por servicio. Cambiar de modo conserva credenciales, emparejamiento e historiales por cuenta y origen. Los proveedores conocidos del PC se sincronizan al actualizar. Si Windows está desconectado, no se cambia automáticamente a otra cuenta del teléfono.

Los detalles muestran tokens y costos estimados en USD de los registros locales del PC, cuando están disponibles, para hoy, 30 días y el período elegido. Pueden incluir varias cuentas; no son cargos facturados. Se muestra la fecha y hora de la agregación. **Gemini corresponde a Gemini CLI; el uso de Google Gemini Apps no se recopila.**

![Configuración real de Windows](docs/images/mobile-connect/windows-release-v0.70.2-sharing-ja.jpg)

![Información y comprobación de actualizaciones del EXE publicado v0.70.2](docs/images/mobile-connect/windows-release-v0.70.2-about-ja.jpg)

[Guía de conexión](docs/android-sharing.md) · [Cambios v0.70.2](docs/releases/v0.70.2.md) · [Validación y límites](docs/mobile-validation.md) · [README completo en inglés](README.md) · [Reportar problemas en este fork](https://github.com/lingmulongtai/Win-CodexBar-Mobile-Connect/issues)

Capturas de Windows nativo y emulador Android con datos sintéticos. La aceptación en teléfonos físicos y con cuentas reales está pendiente. [Licencia MIT](LICENSE).
