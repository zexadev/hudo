# Rust

Rust 编程语言，通过 [rustup](https://rustup.rs) 安装和管理工具链。

## 安装

```powershell
hudo install rust
```

安装 rustup 到 `{install_root}\tools\rustup\`，Cargo 到 `{install_root}\lang\cargo\`。

> 注意：Rust 编译需要 C/C++ 链接器。hudo 会自动检测并提示安装 MinGW（GCC）。

## 安装后

```powershell
rustc --version
cargo --version
rustup show
```

## 卸载

```powershell
hudo uninstall rust
```
