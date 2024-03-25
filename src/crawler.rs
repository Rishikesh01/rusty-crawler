use std::sync::Arc;

use crossbeam_channel::{bounded, Receiver, Sender};
use reqwest::Error;
use scraper::{Html, Selector};

const VALID_PATTERN: &str = "/store/apps/details?id=";

pub struct PlayStoreCrawler {
    client: reqwest::Client,
    sender: Sender<String>,
    reciever: Receiver<String>,
}

impl PlayStoreCrawler {
    pub fn new(client: reqwest::Client) -> Self {
        let (sender, receiever) = bounded(100);
        Self {
            client,
            sender,
            reciever: receiever,
        }
    }

    async fn do_request(&self, url: String) -> Result<String, Error> {
        match self.client.get(url).send().await?.text().await {
            Ok(body) => Ok(body),
            Err(e) => Err(e),
        }
    }

    async fn scrape(&self) {
        if let Ok(url) = self.reciever.recv() {
            if let Ok(body) = self.do_request(url.to_string()).await {
                let document = Html::parse_document(&body);
                let selector = Selector::parse(".Si6A0c").unwrap();
                for element in document.select(&selector) {
                    if let Some(path) = element.value().attr("href") {
                        if path.starts_with(VALID_PATTERN) {
                            self.sender.send(path.to_string()).unwrap()
                        }
                    };
                }
            }
        }
    }

    pub async fn start_scraping(self, url: &str) -> Result<(), Error> {
        self.sender.send(url.to_string()).unwrap();
        let crawler = Arc::new(self); // Wrap self in Arc
        let mut tasks = Vec::new();
        for _ in 0..5 {
            let task = tokio::spawn(PlayStoreCrawler::manage(Arc::clone(&crawler)));
            tasks.push(task);
        }

        // Wait for all tasks to finish
        for task in tasks {
            task.await.expect("Task panicked");
        }

        Ok(())
    }

    async fn manage(crawler: Arc<Self>) {
        crawler.scrape().await;
    }
}
