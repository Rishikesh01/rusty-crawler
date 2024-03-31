pub mod crawler;

use anyhow::Result;
use crossbeam_channel::bounded;

const PLAY_STORE_URL: &str = "https://play.google.com";
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let (sender, receiver) = bounded(100000);

    let client = reqwest::Client::new();
    let crawler = crawler::PlayStoreCrawler::new(client.clone(), sender.clone(), receiver.clone());

    sender.send(PLAY_STORE_URL.to_string())?;
    tokio_scoped::scope(|scope| {
        // Use the scope to spawn the future.

        for _ in 0..5 {
            scope.spawn(async {
                let _ = crawler.clone().start_scraping().await;
            });
        }
    });

    Ok(())
}
