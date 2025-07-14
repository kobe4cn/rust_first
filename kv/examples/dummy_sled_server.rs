use anyhow::Result;
use kv::{CommandRequest, Service, ServiceInner, sleddb::SledDb};
use prost::Message;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
use tracing::info;
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let service: Service<SledDb> = ServiceInner::new(SledDb::new("tmp/kvserver"))
        .fn_berfore_send(|res| match res.message.as_ref() {
            "" => res.message = "message is empty".into(),
            s => res.message = format!("{}:{}", s, res.status),
        })
        .into();
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    info!("Listening on 127.0.0.1:8080");
    loop {
        let (mut stream, _) = listener.accept().await?;
        let svc = service.clone();
        tokio::spawn(async move {
            let mut buf = vec![0; 1024];
            while let Ok(n) = stream.read(&mut buf).await {
                if n == 0 {
                    break;
                }
                let cmd = CommandRequest::decode(&buf[..n]).unwrap();
                let response = svc.exec(cmd);
                // info!("Got command: {:?}", cmd);
                // let mut response = CommandResponse::default();
                // response.status = 0;
                // response.message = "Not Found".to_string();
                // let data = response.encode_to_vec();
                info!("Got response: {:?}", response);
                stream.write_all(&response.encode_to_vec()).await.unwrap();
            }
        });
    }
}
