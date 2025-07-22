# kv 项目说明文档

## 目录结构概览

- `src/`：核心库代码，包括存储引擎、命令处理、网络与 TLS 支持等。
- `examples/`：各类服务端、客户端、证书生成等示例代码，便于功能验证和集成测试。
- `fixtures/`：存放测试用的证书和密钥文件。

---

## 核心模块与服务说明（src/）

### 1. 存储引擎
- **MemTable**：基于内存的哈希表实现，适合测试和轻量级场景。
- **SledDb**：基于 [sled](https://github.com/spacejam/sled) 的嵌入式持久化存储，适合生产环境。
- 两者均实现了统一的 `Storage` trait，支持 get/set/delete/contains/get_all 等操作。

### 2. 命令与服务
- **CommandRequest/CommandResponse**：定义了客户端与服务端之间的消息协议，支持多种命令（如 hset/hget/hgetall 等）。
- **ServiceInner**：服务内部结构，持有存储引擎实例，并支持注册事件钩子（如收到请求、执行后、发送前/后）。
- **Service**：对外暴露的服务对象，封装了命令分发与事件通知逻辑，支持多线程安全 clone。

### 3. TLS 网络支持
- **TlsServerAcceptor**：服务器端 TLS 握手器，负责加载服务器证书/私钥、可选的客户端 CA 证书，实现单向或双向认证。
- **TlsClientConnector**：客户端 TLS 连接器，负责加载 CA 证书、可选的客户端证书/私钥，实现服务器身份校验和可选的客户端认证。

---

## 示例代码说明（examples/）

### 1. 证书生成
- **gen_cert.rs**
  - 用于一键生成自签名 CA 证书、服务器证书、客户端证书及对应私钥，输出到 `fixtures/` 目录。
  - 证书链说明：
    - `ca.cert`：根证书，签发 server/client 证书。
    - `server.cert`/`server.key`：服务器证书及私钥，由 CA 签发。
    - `client.cert`/`client.key`：客户端证书及私钥，由 CA 签发。

### 2. 服务端示例
- **dummy_server.rs**
  - 基于 `MemTable` 的内存型 KV 服务端，无持久化、无 TLS，适合功能演示和开发调试。
- **dummy_sled_server.rs**
  - 基于 `SledDb` 的持久化 KV 服务端，无 TLS，适合本地持久化测试。
- **dummy_sled_server_tls.rs**
  - 基于 `SledDb` 的持久化 KV 服务端，**支持 TLS/双向认证**。
  - 通过 `TlsServerAcceptor` 加载服务器证书/私钥和 CA 证书，实现安全通信。
  - 典型用法：
    ```rust
    let acceptor = TlsServerAcceptor::new(SERVER_CERT, SERVER_KEY, Some(CA_CERT))?;
    ```

### 3. 客户端示例
- **client.rs**
  - 简单的 KV 客户端，无 TLS，演示如何构造命令、发送请求、接收响应。
- **client_tls.rs**
  - 支持 TLS/双向认证的安全客户端。
  - 通过 `TlsClientConnector` 加载 CA 证书和客户端证书/私钥，实现安全通信。
  - 典型用法：
    ```rust
    let connector = TlsClientConnector::new(
        "kvserver.lianwei.inc",
        Some((CLIENT_CERT, CLIENT_KEY)),
        Some(CA_CERT),
    )?;
    ```

---

## TLS 证书与认证流程说明

1. **证书生成**：
   - 运行 `gen_cert.rs`，自动生成 CA、服务器、客户端证书及密钥。
   - 证书文件位于 `fixtures/` 目录。
2. **服务端配置**：
   - 启动 TLS 服务端时，需加载 `server.cert`、`server.key`，并指定 `ca.cert` 作为客户端证书信任根（双向认证时）。
3. **客户端配置**：
   - 启动 TLS 客户端时，需加载 `ca.cert` 作为服务器信任根，并可选加载 `client.cert`、`client.key` 作为客户端身份。
4. **认证逻辑**：
   - 单向认证：客户端验证服务器证书。
   - 双向认证：客户端验证服务器证书，服务器也验证客户端证书。

---

## 典型启动流程

1. 生成证书：
   ```bash
   cargo run --example gen_cert
   ```
2. 启动 TLS 服务端：
   ```bash
   cargo run --example dummy_sled_server_tls
   ```
3. 启动 TLS 客户端：
   ```bash
   cargo run --example client_tls
   ```

---

## 其他说明
- 所有示例端口均为 `127.0.0.1:8080`，可根据需要修改。
- 推荐优先使用 sled 持久化存储和 TLS 安全通信。
- 证书链和认证机制请参考本说明，确保安全配置。
