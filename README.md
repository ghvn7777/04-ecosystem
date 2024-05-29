# Geektime Rust 语言训练营

召唤元素：Rust 生态系统概览

## proxy 测试方法
开两个 terminal，运行这两个程序
```
cargo run --example axum_serde
cargo run --example minginx
```

然后请求 test.rest 的
```
PATCH http://localhost:8081/
Content-Type: application/json

{
  "skills": ["Rust"]
}
```

这样我们的 minginx 可以代理 8081 的请求内容到 8080，得到正确的返回

## chat 测试方法

```
telent 127.0.0.1 8080 # chat server 就是按照 line 切换，文本交互的服务器，可以使用 telnet 登录
```

chat 服务器原理：
![chat introduce](./img/chat.png)

## tokio-console 使用
首先修改 `chat.rs` 的 main 函数里面使用 `console_subscriber::init();`

然后运行加上如下参数
```
RUSTFLAGS="--cfg tokio_unstable" cargo run --example chat
```

安装 tokio-console
```
cargo install --locked tokio-console
```

再执行 `tokio-console` 就可以看到效果了，更多用法可以看它的 github
