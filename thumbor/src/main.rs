use std::{num::NonZero, sync::Arc};

use anyhow::Result;
use lru::LruCache;
use thumbor::{Cache, get_router, print_test_url};
use tokio::sync::Mutex;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{
    Layer as _,
    fmt::{self, Layer, time::LocalTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

#[tokio::main]
async fn main() -> Result<()> {
    let fmt_layer = fmt::layer().with_timer(LocalTime::rfc_3339());
    let layer = Layer::new().pretty().with_filter(LevelFilter::INFO);
    let cache: Cache = Arc::new(Mutex::new(LruCache::new(NonZero::new(1024).unwrap())));
    tracing_subscriber::registry()
        .with(layer)
        .with(fmt_layer)
        .init();

    let router = get_router(cache).await?;
    let addr = format!("0.0.0.0:{}", 8080);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    print_test_url(
        "https://images.pexels.com/photos/1562477/pexels-photo-1562477.jpeg?auto=compress&cs=tinysrgb&dpr=3&h=750&w=1260",
    );
    info!("listening on {}", addr);

    axum::serve(listener, router.into_make_service()).await?;
    Ok(())
}
