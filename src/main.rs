pub mod crawler;

use anyhow::Result;

const PLAY_STORE_URL: &str = "https://play.google.com";
#[tokio::main]
async fn main() -> Result<()> {
    let mut paths = crawler::PlayStoreCrawler::new(reqwest::Client::new())
        .scrape(PLAY_STORE_URL)
        .await;
    for path in paths.iter() {
        println!("{:?}", path);
    }
    env_logger::init();
    Ok(())
}
