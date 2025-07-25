use anyhow::Result;

use futures::{SinkExt as _, StreamExt};
use kv::{CommandRequest, Service, ServiceInner, sleddb::SledDb};
use prost::Message;
use tokio::net::TcpListener;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
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
        let (stream, _) = listener.accept().await?;
        let mut stream = Framed::new(stream, LengthDelimitedCodec::new());
        let svc = service.clone();
        //复用Bytemut 避免重新分配
        tokio::spawn(async move {
            while let Some(Ok(mut n)) = stream.next().await {
                if n.is_empty() {
                    break;
                }
                let cmd = CommandRequest::decode(&n[..]).unwrap();
                let response = svc.exec(cmd);
                info!("Got response: {:?}", response);
                response.encode(&mut n).unwrap();
                stream.send(n.freeze()).await.unwrap();
            }
        });
        ////////////////////////////////////////////////
        //重新分配Vec 性能略低，安全性健壮性比较高
        // tokio::spawn(async move {
        //     while let Some(Ok(n)) = stream.next().await {
        //         if n.is_empty() {
        //             break;
        //         }
        //         let cmd = CommandRequest::decode(&n[..]).unwrap();
        //         let response = svc.exec(cmd);
        //         stream
        //             .send(Bytes::from(response.encode_to_vec()))
        //             .await
        //             .unwrap();
        //     }
        // });
        ////////////////////////////////////////////////
    }
}
