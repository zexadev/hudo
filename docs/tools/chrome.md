# Google Chrome

Google Chrome 浏览器，使用企业版 MSI 静默安装。

## 安装

```powershell
hudo install chrome
```

使用企业版 MSI 安装包，静默安装到系统目录（`%ProgramFiles%\Google\Chrome\`），需要 UAC 提权。

## 注意

- Chrome 不支持自定义安装路径，由 Google 安装程序决定
- 安装时会弹出 UAC 提示，点击「是」继续
- Chrome 不会添加到 PATH（不是命令行工具）

## 卸载

```powershell
hudo uninstall chrome
```

卸载时会自动调用 Chrome 内置卸载程序。若未找到，请通过「控制面板 → 程序」手动卸载。
