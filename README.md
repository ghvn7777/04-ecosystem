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
