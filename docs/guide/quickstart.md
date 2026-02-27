# 快速上手

## 第一次运行

安装 hudo 后，直接运行：

```powershell
hudo
```

首次运行会询问工具安装根目录（如 `D:\`），之后进入交互式安装菜单。

## 安装工具

```powershell
# 进入交互菜单，方向键选择，空格勾选，回车确认
hudo

# 直接安装指定工具
hudo install git
hudo install nodejs
hudo install vscode
```

## 查看已安装工具

```powershell
hudo list
```

## 更新工具

```powershell
# 更新 hudo 自身
hudo update
```

## 卸载工具

```powershell
hudo uninstall git
```

## 配置档案

```powershell
# 导出当前配置
hudo profile export

# 在新电脑上还原
hudo profile import
```
