pub mod crawler;

use anyhow::Result;

const PLAY_STORE_URL: &str = "https://play.google.com";
#[tokio::main]
async fn main() -> Result<()> {
    crawler::PlayStoreCrawler::new(reqwest::Client::new())
        .start_scraping(PLAY_STORE_URL)
        .await
        .unwrap();
    env_logger::init();
    Ok(())
}
