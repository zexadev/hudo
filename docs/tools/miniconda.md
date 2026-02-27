# Miniconda

Conda 包管理器最小安装版，适合科学计算、数据分析场景。

## 安装

```powershell
hudo install miniconda
```

静默安装到 `{install_root}\tools\miniconda\`，仅安装当前用户，不注册为系统 Python，不自动修改 PATH（由 hudo 统一管理）。

## 安装后

```powershell
conda --version

# 创建环境
conda create -n myenv python=3.11

# 激活环境
conda activate myenv

# 安装包
conda install numpy pandas
```

## 卸载

```powershell
hudo uninstall miniconda
```
