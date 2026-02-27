# PostgreSQL

PostgreSQL 关系型数据库。

## 安装

```powershell
hudo install pgsql
```

安装到 `{install_root}\lang\pgsql\`，自动获取最新版本，自动注册为 Windows 服务（需要 UAC 提权）。

## 安装后

```powershell
psql --version

# 连接
psql -U postgres
```

## 服务管理

```powershell
# 启动
net start PostgreSQL

# 停止
net stop PostgreSQL
```

## 卸载

```powershell
hudo uninstall pgsql
```

## 注意

- 服务注册需要管理员权限，安装时会弹出 UAC 提示
- 服务名为 `PostgreSQL`
