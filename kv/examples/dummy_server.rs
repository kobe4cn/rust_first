use anyhow::Result;
use bytes::Bytes;
use futures::{SinkExt as _, StreamExt};
use kv::{CommandRequest, MemTable, Service, ServiceInner};
use prost::Message;
use tokio::net::TcpListener;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let service: Service<MemTable> = ServiceInner::new(MemTable::new()).into();
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    info!("Listening on 127.0.0.1:8080");
    loop {
        let (stream, _) = listener.accept().await?;
        let mut stream = Framed::new(stream, LengthDelimitedCodec::new());
        let svc = service.clone();
        tokio::spawn(async move {
            // let mut buf = vec![0; 1024];
            while let Some(Ok(n)) = stream.next().await {
                if n.is_empty() {
                    break;
                }
                let cmd = CommandRequest::decode(&n[..]).unwrap();
                let response = svc.exec(cmd);
                // info!("Got command: {:?}", cmd);
                // let mut response = CommandResponse::default();
                // response.status = 0;
                // response.message = "Not Found".to_string();
                // let data = response.encode_to_vec();
                info!("Got response: {:?}", response);
                stream
                    .send(Bytes::from(response.encode_to_vec()))
                    .await
                    .unwrap();
            }
        });
    }
}
