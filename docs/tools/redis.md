# Redis

Redis 内存数据库，使用 [redis-windows](https://github.com/redis-windows/redis-windows) 提供的 Windows 预编译包。

## 安装

```powershell
hudo install redis
```

安装到 `{install_root}\tools\redis\`，自动获取最新版本，自动注册为 Windows 服务（需要 UAC 提权）。

## 安装后

```powershell
redis-server --version

# 连接
redis-cli
```

## 服务管理

```powershell
# 启动
net start Redis

# 停止
net stop Redis
```

## 卸载

```powershell
hudo uninstall redis
```

## 注意

- 服务注册需要管理员权限，安装时会弹出 UAC 提示
- 服务名为 `Redis`
- 默认绑定 `127.0.0.1:6379`
- 数据目录在 `tools\redis\data\`
