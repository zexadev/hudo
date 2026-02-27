# MinGW (C/C++)

MinGW-w64 GCC 编译器工具链，来自 [winlibs](https://winlibs.com) 独立构建版，无需 MSYS2。

## 安装

```powershell
hudo install c
```

安装到 `{install_root}\tools\mingw64\`，自动获取最新版本（UCRT 运行时，来自 winlibs 独立构建）。

> Rust 安装时若检测到缺少链接器，会自动提示安装 MinGW。

## 安装后

```powershell
gcc --version
g++ --version
```

## 卸载

```powershell
hudo uninstall c
```
