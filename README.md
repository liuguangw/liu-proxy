# liu-proxy
liu-proxy 是一个使用rust语言编写的代理服务器。代理服务器分为服务端和客户端，之间使用tls进行加密，使用websocket作为传输协议(wss协议)。

客户端使用连接池模式，当浏览器或者其他软件断开连接之后，客户端与服务端之间的底层TCP连接并不会断开，连接会被放入连接池，以供下次使用。从而可以避免每次都要重新握手的情况，减少延迟。如果不需要连接池，可以在配置文件里设置 `max_idle_conns = 0` 来关闭这个功能。

## 使用说明

这个程序既可以作为客户端运行，也可以作为服务端运行。关键在于输入的参数, 使用 `./liu-proxy --help` 可以看到命令行说明。

### 作为服务端运行

一般情况下是在linux服务器上运行服务端, 使用下面的命令就可以运行服务端

```
./liu-proxy server
```

默认的配置文件为 `./config/config.toml` ，可以使用 `-f` 参数自定义配置文件路径，例如：

```
./liu-proxy server -f /etc/path/to/config.toml
```

更多服务端部署说明可以[查看wiki](https://github.com/liuguangw/liu-proxy/wiki)

### 作为客户端运行

使用下面的命令就可以运行客户端程序

```
./liu-proxy client
```

默认的配置文件为 `./config/config.toml` ，可以使用 `-f` 参数自定义配置文件路径，例如：

```
./liu-proxy client -f C:/path/to/config.toml
```

## 配置文件

本项目下的 `config/config.toml` 就是程序的配置文件，对于服务端只需要 `[server]` 的那一部分，而客户端则是 `[client]`那部分。

值得注意的是，服务端定义的path要和客户端定义的server url中的path保持一致。

### 身份验证

服务端的auth_users可以配置多个用户，例如：

```toml
auth_users = [
    { user = "aaaa", key = "123456" },
    { user = "bbbb", key = "654321" },
]
```

客户端需要设置其中的一个，例如：

```toml
[client]
address = "127.0.0.1"
port = 8002
auth_user = { user = "aaaa", key = "123456" }
#.....
```

此外服务端时间和客户端时间需要保持准确，不能误差超过三分钟。否则一律会返回404错误，就好像 `websocket` 服务不存在一样。