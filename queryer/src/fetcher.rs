use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Fetch {
    type Error;
    async fn fetch(&self) -> Result<String, Self::Error>;
}

pub async fn retrieve_data(source: &str) -> Result<String> {
    match &source[..4] {
        "http" => HttpFetcher(source).fetch().await,
        "file" => FileFetcher(source).fetch().await,
        _ => Err(anyhow::anyhow!("Unsupported source: {}", source)),
    }
}

struct HttpFetcher<'a>(pub(crate) &'a str);
struct FileFetcher<'a>(pub(crate) &'a str);

#[async_trait]
impl<'a> Fetch for HttpFetcher<'a> {
    type Error = anyhow::Error;
    async fn fetch(&self) -> Result<String, Self::Error> {
        let response = reqwest::get(self.0).await?;
        let body = response.text().await?;
        Ok(body)
    }
}

#[async_trait]
impl<'a> Fetch for FileFetcher<'a> {
    type Error = anyhow::Error;
    async fn fetch(&self) -> Result<String, Self::Error> {
        let path = &self.0[7..];
        let body = std::fs::read_to_string(path)?;
        Ok(body)
    }
}
