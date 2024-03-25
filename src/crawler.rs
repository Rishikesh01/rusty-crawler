use std::{
    borrow::Borrow,
    collections::HashSet,
    sync::{Arc, Mutex},
};

use crossbeam_channel::{bounded, Receiver, Sender};
use reqwest::Error;
use scraper::{Html, Selector};

const VALID_PATTERN: &str = "/store/apps/details?id=";
const PLAY_STORE_URL: &str = "https://play.google.com";

pub struct PlayStoreCrawler {
    client: reqwest::Client,
    sender: Sender<String>,
    filter: Mutex<HashSet<String>>,
    reciever: Receiver<String>,
}

impl PlayStoreCrawler {
    pub fn new(client: reqwest::Client) -> Self {
        let (sender, receiever) = bounded(100);
        Self {
            client,
            sender,
            filter: Mutex::new(HashSet::new()),
            reciever: receiever,
        }
    }

    async fn do_request(&self, url: String) -> Result<String, Error> {
        match self.client.get(url).send().await?.text().await {
            Ok(body) => Ok(body),
            Err(e) => {
                println!("Error receiving URL: {:?}", e);

                Err(e)
            }
        }
    }

    async fn scrape(&self, num: i32) {
        println!("{}", num);
        loop {
            match self.reciever.recv() {
                Ok(url) => {
                    if let Ok(body) = self.do_request(url.to_string()).await {
                        let document = Html::parse_document(&body);
                        match Selector::parse(".Si6A0c") {
                            Ok(selector) => {
                                println!("{} task num {} ", url, num);
                                for element in document.select(&selector) {
                                    if let Some(path) = element.value().attr("href") {
                                        if path.starts_with(VALID_PATTERN) {
                                            let full_url = format!("{}{}", PLAY_STORE_URL, path);

                                            if let Err(err) = self.sender.send(full_url) {
                                                eprintln!("Error sending path: {:?}", err);
                                            }
                                        }
                                    }
                                }
                            }
                            Err(_) => todo!(),
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Error receiving URL: {:?}", err);
                    continue; // Continue looping even if there's an error receiving URL
                }
            }
        }
    }
    pub async fn start_scraping(self, url: &str) -> Result<(), Error> {
        self.sender.send(url.to_string()).unwrap();
        let crawler = Arc::new(self); // Wrap self in Arc
        let mut tasks = Vec::new();
        for num in 0..5 {
            let task = tokio::spawn(PlayStoreCrawler::manage(Arc::clone(&crawler), num));
            tasks.push(task);
        }

        // Wait for all tasks to finish
        for task in tasks {
            task.await.expect("Task panicked");
        }

        Ok(())
    }

    async fn manage(crawler: Arc<Self>, num: i32) {
        crawler.scrape(num).await;
    }
}
