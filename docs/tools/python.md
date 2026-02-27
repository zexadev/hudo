# Python (uv)

Python 环境通过 [uv](https://github.com/astral-sh/uv) 管理，uv 是 Rust 编写的极速 Python 包管理器。

## 安装

```powershell
hudo install uv
```

安装 uv 到 `{install_root}\tools\uv\`。

## 安装后

```powershell
uv --version

# 创建虚拟环境
uv venv

# 安装包
uv pip install requests

# 运行 Python
uv run python script.py
```

## 安装指定 Python 版本

```powershell
uv python install 3.12
uv python list
```

## 卸载

```powershell
hudo uninstall uv
```
