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

- **CommandRequest/CommandResponse**：基于 Protocol Buffers 定义的客户端与服务端消息协议，支持完整的哈希表操作命令。
- **ServiceInner**：服务内部结构，持有存储引擎实例，并支持注册事件钩子（如收到请求、执行后、发送前/后）。
- **Service**：对外暴露的服务对象，封装了命令分发与事件通知逻辑，支持多线程安全 clone。

#### 支持的命令类型

- **hget/hset** - 单个键值的获取和设置操作
- **hgetall** - 获取指定表中的所有键值对
- **hmget/hmset** - 批量键值的获取和设置操作
- **hdel/hmdel** - 单个/批量键的删除操作
- **hexists/hmexists** - 单个/批量键的存在性检查

#### 支持的数据类型

- **字符串** (string_value) - UTF-8 编码的文本数据
- **字节数组** (bytes_value) - 二进制数据
- **64 位整数** (int64_value) - 有符号长整型
- **双精度浮点数** (double_value) - IEEE 754 双精度浮点数
- **布尔值** (bool_value) - true/false 值

### 3. TLS 网络支持

- **TlsServerAcceptor**：服务器端 TLS 握手器，负责加载服务器证书/私钥、可选的客户端 CA 证书，实现单向或双向认证。
- **TlsClientConnector**：客户端 TLS 连接器，负责加载 CA 证书、可选的客户端证书/私钥，实现服务器身份校验和可选的客户端认证。

### 4. 错误处理系统

- **统一错误类型**：定义了完整的 `KvError` 枚举，涵盖各种错误场景
- **错误类型包括**：
  - `NotFound` - 键或表不存在
  - `InvalidCommand` - 无效的命令格式
  - `ConvertError` - 数据类型转换错误
  - `StorageError` - 存储引擎相关错误
  - `CertParseError` - 证书解析错误
  - `IoError` - 网络 IO 错误
- **统一响应格式**：所有响应都包含状态码、消息和数据，便于客户端统一处理

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
        "kvserver.kevin.inc",
        Some((CLIENT_CERT, CLIENT_KEY)),
        Some(CA_CERT),
    )?;
    ```

---

## TLS 证书与认证流程说明

### 证书链结构

- **ca.cert** - 根证书，用于签发服务器和客户端证书
- **server.cert/server.key** - 服务器证书及私钥，由 CA 签发
- **client.cert/client.key** - 客户端证书及私钥，由 CA 签发

### 配置流程

1. **证书生成**：

   - 运行 `gen_cert.rs`，自动生成 CA、服务器、客户端证书及密钥
   - 证书文件位于 `fixtures/` 目录
   - 服务器证书域名：`kvserver.kevin.inc`

2. **服务端配置**：

   - 加载 `server.cert`、`server.key` 作为服务器身份
   - **重要**：`client_ca` 参数应传入根证书 `ca.cert`（双向认证时）
   - 单向认证：`TlsServerAcceptor::new(SERVER_CERT, SERVER_KEY, None)`
   - 双向认证：`TlsServerAcceptor::new(SERVER_CERT, SERVER_KEY, Some(CA_CERT))`

3. **客户端配置**：
   - **重要**：`server_ca` 参数应传入根证书 `ca.cert` 用于验证服务器
   - 可选加载 `client.cert`、`client.key` 作为客户端身份（双向认证时）
   - 单向认证：`TlsClientConnector::new("kvserver.kevin.inc", None, Some(CA_CERT))`
   - 双向认证：`TlsClientConnector::new("kvserver.kevin.inc", Some((CLIENT_CERT, CLIENT_KEY)), Some(CA_CERT))`

### 认证模式

- **单向认证**：仅客户端验证服务器证书，确保连接到正确的服务器
- **双向认证**：客户端和服务器互相验证证书，提供更高的安全性

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

## 技术特性

### 核心特性

- **异步处理** - 基于 Tokio 的高性能异步网络 IO
- **类型安全** - 使用 Protocol Buffers 确保消息格式一致性
- **灵活存储** - 统一的 Storage trait 支持多种存储后端
- **完整安全** - 支持 TLS 1.3，包含单向和双向认证
- **事件系统** - 支持请求生命周期的事件钩子注册

### 性能特点

- **零拷贝** - 高效的二进制序列化
- **连接复用** - 支持长连接和连接池
- **内存优化** - MemTable 提供高速内存访问
- **持久化** - SledDb 提供可靠的磁盘存储

## 配置说明

- **默认端口**：`127.0.0.1:8080`，可根据需要修改
- **推荐配置**：生产环境使用 SledDb + TLS 双向认证
- **开发调试**：使用 MemTable + 无 TLS 配置
- **证书域名**：确保客户端连接时使用正确的域名 `kvserver.kevin.inc`

## 注意事项

- 证书文件路径必须正确，建议使用绝对路径
- 双向认证时，客户端和服务端都需要配置 CA 证书
- 生产环境建议定期更新证书，避免过期
- 存储引擎选择：测试用 MemTable，生产用 SledDb
