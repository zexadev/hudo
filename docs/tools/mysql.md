# MySQL

MySQL Community Server，关系型数据库。

## 安装

```powershell
hudo install mysql
```

安装到 `{install_root}\lang\mysql\`，自动获取最新版本，自动注册为 Windows 服务（需要 UAC 提权）。

## 安装后

```powershell
mysql --version

# 连接（初始无密码）
mysql -u root
```

## 服务管理

```powershell
# 启动
net start MySQL

# 停止
net stop MySQL
```

## 卸载

```powershell
hudo uninstall mysql
```

## 注意

- 服务注册需要管理员权限，安装时会弹出 UAC 提示
- 服务名为 `MySQL`
