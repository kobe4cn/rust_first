use std::sync::Arc;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::{
    TlsAcceptor, TlsConnector,
    client::TlsStream as ClientTlsStream,
    rustls::{
        ClientConfig, RootCertStore, ServerConfig,
        pki_types::{CertificateDer, PrivateKeyDer, ServerName, pem::PemObject},
        server::WebPkiClientVerifier,
    },
    server::TlsStream as ServerTlsStream,
};

use crate::error::KvError;

const ALPN_KV: &str = "kv";
#[derive(Clone)]
#[allow(unused)]
pub struct TlsServerAcceptor {
    inner: Arc<ServerConfig>,
}

#[derive(Clone)]
#[allow(unused)]
pub struct TlsClientConnector {
    pub config: Arc<ClientConfig>,
    pub domain: Arc<String>,
}
#[allow(unused)]
impl TlsClientConnector {
    pub fn new(
        domain: impl Into<String>,
        identity: Option<(&str, &str)>,
        server_ca: Option<&str>, // 应该传入的是根证书 ca.cert
    ) -> Result<Self, KvError> {
        let mut root_cert_store = RootCertStore::empty();
        if let Some(server_ca) = server_ca {
            let certs = load_certs(server_ca)?;
            root_cert_store.add_parsable_certificates(certs);
        }

        let config = if let Some((cert, key)) = identity {
            let certs = load_certs(cert)?;
            let key = load_key(key)?;
            ClientConfig::builder()
                .with_root_certificates(root_cert_store)
                .with_client_auth_cert(certs, key)?
        } else {
            ClientConfig::builder()
                .with_root_certificates(root_cert_store)
                .with_no_client_auth()
        };

        Ok(Self {
            config: Arc::new(config),
            domain: Arc::new(domain.into()),
        })
    }
    pub async fn connect<S>(&self, stream: S) -> Result<ClientTlsStream<S>, KvError>
    where
        S: AsyncRead + AsyncWrite + Unpin + Send,
    {
        let dns_name =
            ServerName::try_from(self.domain.as_str().to_string()).expect("invalid DNS name");
        let stream = TlsConnector::from(self.config.clone())
            .connect(dns_name, stream)
            .await?;
        Ok(stream)
    }
}
#[allow(unused)]
impl TlsServerAcceptor {
    //client_ca 应该传入的是根证书 ca.cert
    pub fn new(cert: &str, key: &str, client_ca: Option<&str>) -> Result<Self, KvError> {
        let certs = load_certs(cert)?;
        let keys = load_key(key)?;
        let mut config = match client_ca {
            None => ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(certs, keys)?,
            Some(client_ca) => {
                let client_certs = load_certs(client_ca)?;
                let mut root_cert_store = RootCertStore::empty();
                root_cert_store.add_parsable_certificates(client_certs);
                let certs = load_certs(cert)?;
                let key = load_key(key)?;
                let verifier = WebPkiClientVerifier::builder(root_cert_store.into())
                    .build()
                    .unwrap();
                ServerConfig::builder()
                    .with_client_cert_verifier(verifier)
                    .with_single_cert(certs.clone(), key)?
            }
        };
        config.alpn_protocols = vec![ALPN_KV.as_bytes().to_vec()];

        Ok(Self {
            inner: Arc::new(config),
        })
    }

    pub async fn accept<S>(&self, stream: S) -> Result<ServerTlsStream<S>, KvError>
    where
        S: AsyncRead + AsyncWrite + Unpin + Send,
    {
        let stream = TlsAcceptor::from(self.inner.clone()).accept(stream).await?;
        Ok(stream)
    }
}
#[allow(unused)]
fn load_certs(key: &str) -> Result<Vec<CertificateDer<'static>>, KvError> {
    println!("load_certs: {}", key);
    let certs = CertificateDer::pem_file_iter(key)
        .unwrap()
        .map(|c| c.map_err(|e| KvError::CertParseError(key.to_string(), e.to_string())))
        .collect::<Result<Vec<_>, KvError>>()?;
    Ok(certs)
}

#[allow(unused)]
fn load_key(key: &str) -> Result<PrivateKeyDer<'static>, KvError> {
    let key = PrivateKeyDer::from_pem_file(key).unwrap();
    match key {
        PrivateKeyDer::Pkcs8(keys) => Ok(PrivateKeyDer::Pkcs8(keys)),
        PrivateKeyDer::Sec1(keys) => Ok(PrivateKeyDer::Sec1(keys)),
        PrivateKeyDer::Pkcs1(keys) => Ok(PrivateKeyDer::Pkcs1(keys)),
        _ => Err(KvError::CertParseError(
            "private".to_string(),
            "not a valid key".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {

    use std::net::SocketAddr;

    use super::*;
    use anyhow::Result;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::{TcpListener, TcpStream},
    };
    use tracing::info;

    const CA_CERT: &str = "fixtures/ca.cert";
    const CLIENT_CERT: &str = "fixtures/client.cert";
    const CLIENT_KEY: &str = "fixtures/client.key";
    const SERVER_CERT: &str = "fixtures/server.cert";
    const SERVER_KEY: &str = "fixtures/server.key";

    #[tokio::test]
    async fn tls_should_work() -> Result<()> {
        let ca = Some(CA_CERT);

        let addr = start_server(None).await?;

        let connector = TlsClientConnector::new("kvserver.kevin.inc", None, ca)?;
        let stream = TcpStream::connect(addr).await?;
        let mut stream = connector.connect(stream).await?;
        stream.write_all(b"hello world!").await?;
        let mut buf = [0; 12];
        stream.read_exact(&mut buf).await?;
        assert_eq!(&buf, b"hello world!");

        Ok(())
    }

    #[tokio::test]
    async fn tls_with_client_cert_should_work() -> Result<()> {
        let client_identity = Some((CLIENT_CERT, CLIENT_KEY));
        let ca = Some(CA_CERT);

        let addr = start_server(ca.clone()).await?;

        let connector =
            TlsClientConnector::new("kvserver.kevin.inc", client_identity, Some(SERVER_CERT))?;
        let stream = TcpStream::connect(addr).await?;
        let mut stream = connector.connect(stream).await?;
        stream.write_all(b"hello world!").await?;
        let mut buf = [0; 12];
        stream.read_exact(&mut buf).await?;
        assert_eq!(&buf, b"hello world!");

        Ok(())
    }

    #[tokio::test]
    async fn tls_with_bad_domain_should_not_work() -> Result<()> {
        let addr = start_server(Some(CA_CERT)).await?;

        // let connector = TlsClientConnector::new("kvserver1.acme.inc", None, Some(SERVER_CERT))?;
        let connector = TlsClientConnector::new("kvserver.kevin.inc", None, Some(SERVER_CERT))?;
        let stream = TcpStream::connect(addr).await?;
        let result = connector.connect(stream).await;
        println!("result: {:?}", result);
        assert!(result.is_ok());

        Ok(())
    }

    async fn start_server(ca: Option<&str>) -> Result<SocketAddr> {
        let acceptor = TlsServerAcceptor::new(SERVER_CERT, SERVER_KEY, ca)?;

        let echo = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = echo.local_addr().unwrap();

        tokio::spawn(async move {
            let (stream, _) = echo.accept().await.unwrap();
            let mut stream = acceptor.accept(stream).await.unwrap();
            let mut buf = [0; 12];
            stream.read_exact(&mut buf).await.unwrap();
            stream.write_all(&buf).await.unwrap();
        });

        Ok(addr)
    }
}
