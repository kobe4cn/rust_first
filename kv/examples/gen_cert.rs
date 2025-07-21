use anyhow::Result;
use certify::{CA, CertSigAlgo, CertType, generate_ca, generate_cert};
use tokio::fs;

struct CertPem {
    cert_type: CertType,
    cert: String,
    key: String,
}

fn create_ca() -> Result<CertPem> {
    let (cert, key) = generate_ca(
        "CH",
        "kevin, Inc.",
        "kevin",
        CertSigAlgo::ED25519,
        None,
        Some(10 * 365),
    )?;
    Ok(CertPem {
        cert_type: CertType::CA,
        cert,
        key,
    })
}

fn create_cert(ca: &CA, domains: &[&str], cn: &str, is_client: bool) -> Result<CertPem> {
    let (days, cert_type) = if is_client {
        (365, CertType::Client)
    } else {
        (365, CertType::Server)
    };
    let (cert, key) = generate_cert(
        ca,
        domains.to_vec(),
        "CN",
        "kevin, Inc.",
        cn,
        CertSigAlgo::ED25519,
        None,
        is_client,
        Some(days),
    )?;
    Ok(CertPem {
        cert_type,
        cert,
        key,
    })
}

async fn gen_files(pem: &CertPem) -> Result<()> {
    let name = match pem.cert_type {
        CertType::CA => "ca",
        CertType::Client => "client",
        CertType::Server => "server",
    };
    fs::write(format!("fixtures/{}.cert", name), pem.cert.as_bytes()).await?;
    fs::write(format!("fixtures/{}.key", name), pem.key.as_bytes()).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let ca = create_ca()?;
    gen_files(&ca).await?;
    let ca = CA::load(&ca.cert, &ca.key)?;
    // let client = create_cert(&ca, &["kvserver.kevin.inc"], "awesome-client", true)?;
    let client = create_cert(&ca, &["kvserver.kevin.inc"], "awesome-client", true)?;
    gen_files(&client).await?;
    let server = create_cert(&ca, &["kvserver.kevin.inc"], "awesome-server", false)?;
    gen_files(&server).await?;

    Ok(())
}
