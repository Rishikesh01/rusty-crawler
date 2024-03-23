pub mod crawler;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    crawler::PlayStoreCrawler::new(reqwest::Client::new())
        .scrape()
        .await;
    env_logger::init();
    Ok(())
}
