use anyhow::Result;
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use kv::{CommandRequest, CommandResponse};
use prost::Message;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let addr = "127.0.0.1:8080";
    let stream = TcpStream::connect(addr).await?;
    let mut stream = Framed::new(stream, LengthDelimitedCodec::new());
    let mut cmds = Vec::new();
    // 创建命令
    cmds.push(CommandRequest::new_hset("test", "key", "test".into()));
    cmds.push(CommandRequest::new_hset("test", "key1", 10.into()));
    cmds.push(CommandRequest::new_hset("test", "key3", 10.1.into()));
    cmds.push(CommandRequest::new_hset("test", "key2", true.into()));
    cmds.push(CommandRequest::new_hget("test", "key"));
    cmds.push(CommandRequest::new_hget("test", "key1"));
    cmds.push(CommandRequest::new_hget("test", "key2"));
    cmds.push(CommandRequest::new_hget("test", "key3"));
    cmds.push(CommandRequest::new_hgetall("test"));
    for cmd in cmds {
        // 序列化并发送命令
        let data = cmd.encode_to_vec();
        stream.send(Bytes::from(data)).await?;

        // 接收响应info!("Sent command");

        // 接收响应
        if let Some(Ok(n)) = stream.next().await {
            let response = CommandResponse::decode(&n[..])?;
            info!("Got response: {:?}", response);
        }
    }

    Ok(())
}
