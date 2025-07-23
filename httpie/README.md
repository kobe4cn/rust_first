# HTTPie - Rust HTTP客户端

一个用Rust编写的简单而强大的HTTP命令行客户端，灵感来自于著名的HTTPie工具。

## 功能特性

- 🚀 **简单易用** - 直观的命令行界面
- 🎨 **彩色输出** - 语法高亮的响应显示
- 📡 **支持GET和POST请求** - 覆盖常用HTTP方法
- 🔧 **JSON数据处理** - 自动格式化和美化JSON响应
- ⚡ **异步处理** - 基于Tokio的高性能异步HTTP客户端
- 🛡️ **错误处理** - 完善的错误处理机制

## 安装

确保你已经安装了Rust工具链，然后克隆并构建项目：

```bash
git clone https://github.com/kevin-yang-xgz/httpie
cd httpie
cargo build --release
```

## 使用方法

### GET请求

发送GET请求到指定URL：

```bash
# 基本GET请求
./target/release/httpie get -u https://api.github.com/users/octocat

# 简化URL（自动添加http://前缀）
./target/release/httpie get -u api.github.com/users/octocat
```

### POST请求

发送带有JSON数据的POST请求：

```bash
# POST请求with JSON数据
./target/release/httpie post -u https://httpbin.org/post -d name=John -d age=30

# 多个键值对
./target/release/httpie post -u https://api.example.com/users -d username=john -d email=john@example.com -d active=true
```

## 命令行参数

### 全局选项
- `--version` - 显示版本信息
- `--help` - 显示帮助信息

### GET命令
- `-u, --url <URL>` - 目标URL（必需）

### POST命令
- `-u, --url <URL>` - 目标URL（必需）
- `-d, --data <KEY=VALUE>` - JSON数据键值对（可重复使用）

## 输出格式

HTTPie会以彩色格式显示HTTP响应：

- **蓝色** - HTTP版本和状态码
- **红色** - 状态文本
- **绿色** - 响应头键名
- **紫色** - 响应头值
- **红色/黄色** - JSON响应体（键/值）

## 技术栈

- **Rust** - 系统编程语言
- **Tokio** - 异步运行时
- **Reqwest** - HTTP客户端库
- **Clap** - 命令行参数解析
- **Colored** - 终端彩色输出
- **Serde** - 序列化/反序列化
- **JSONxf** - JSON格式化
- **Anyhow** - 错误处理

## 项目结构

```
httpie/
├── Cargo.toml          # 项目配置和依赖
├── src/
│   └── main.rs         # 主程序入口
└── README.md           # 项目文档
```

## 核心功能实现

### URL解析
自动处理URL格式，如果没有协议前缀会自动添加`http://`：

```rust
fn parse_url(url: &str) -> Result<String> {
    let new_url = if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("http://{}", url)
    };
    // 验证URL格式
    let _url: Url = new_url.parse()?;
    Ok(new_url.into())
}
```

### 键值对解析
支持`key=value`格式的数据输入：

```rust
impl FromStr for KeyPair {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (key, value) = s
            .split_once('=')
            .ok_or_else(|| anyhow::anyhow!("failed to split key-value"))?;
        Ok(KeyPair {
            key: key.to_string(),
            value: value.to_string(),
        })
    }
}
```

## 示例

### 获取用户信息
```bash
./target/release/httpie get -u https://jsonplaceholder.typicode.com/users/1
```

### 创建新用户
```bash
./target/release/httpie post -u https://jsonplaceholder.typicode.com/users -d name="John Doe" -d email="john@example.com"
```

## 开发

### 运行开发版本
```bash
cargo run -- get -u https://httpbin.org/get
cargo run -- post -u https://httpbin.org/post -d hello=world
```

### 运行测试
```bash
cargo test
```

## 许可证

MIT License - 详见LICENSE文件

## 作者

kevin.yang.xgz@gmail.com

## 贡献

欢迎提交Issue和Pull Request！

---

*这是一个学习项目，用于演示Rust中的HTTP客户端开发、命令行工具构建和异步编程。*