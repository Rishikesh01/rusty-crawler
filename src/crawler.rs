use std::error::Error;

use crossbeam_channel::{Receiver, Sender};
use scraper::{Html, Selector};

const VALID_PATTERN: &str = "/store/apps/details?id=";
const PLAY_STORE_URL: &str = "https://play.google.com";

#[derive(Clone)]
pub struct PlayStoreCrawler {
    client: reqwest::Client,
    sender: Sender<String>,
    receiver: Receiver<String>,
}

impl PlayStoreCrawler {
    pub fn new(
        client: reqwest::Client,
        sender: Sender<String>,
        receiver: Receiver<String>,
    ) -> Self {
        Self {
            client,
            sender,
            receiver,
        }
    }

    async fn do_request(&self, url: String) -> Result<String, reqwest::Error> {
        match self.client.get(url).send().await?.text().await {
            Ok(body) => Ok(body),
            Err(e) => {
                println!("Error receiving URL: {:?}", e);

                Err(e)
            }
        }
    }

    async fn scrape(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        loop {
            let url = self.receiver.recv().map_err(|e| {
                Box::<dyn Error + Send + Sync>::from(format!(
                    "error while receiving in url: {:?}",
                    e
                ))
            })?;
            println!("{}", url);
            let page = self.do_request(url.clone()).await?;
            let document = Html::parse_document(&page);
            let result = Selector::parse(".Si6A0c").map_err(|e| {
                Box::<dyn Error + Send + Sync>::from(format!("error while parsing page: {:?}", e))
            })?;
            for element in document.select(&result) {
                if let Some(path) = element.value().attr("href") {
                    if path.starts_with(VALID_PATTERN) {
                        let full_url = format!("{}{}", PLAY_STORE_URL, path);
                        if let Err(err) = self.sender.send(full_url.clone()) {
                            eprintln!("Task Error sending URL: {:?}", err);
                        }
                    }
                }
            }
        }
    }

    pub async fn start_scraping(self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.scrape().await
    }
}
