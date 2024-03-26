use std::sync::Arc;

use crossbeam_channel::{bounded, Receiver, Sender};
use reqwest::Error;
use scraper::{Html, Selector};

const VALID_PATTERN: &str = "/store/apps/details?id=";
const PLAY_STORE_URL: &str = "https://play.google.com";

pub struct PlayStoreCrawler {
    client: reqwest::Client,
    sender: Sender<String>,
    receiver: Receiver<String>,
}

impl PlayStoreCrawler {
    pub fn new(client: reqwest::Client) -> Self {
        let (sender, receiver) = bounded(100000);
        Self {
            client,
            sender,
            receiver,
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

    async fn scrape(&self, sender: Sender<String>, receiver: Receiver<String>, num: i32) {
        println!("Task {}: Start scraping", num);
        loop {
            match receiver.recv() {
                Ok(url) => {
                    println!("Task {}: Received URL: {}", num, url);
                    match self.do_request(url.clone()).await {
                        Ok(body) => {
                            println!("Task {}: Successfully fetched URL: {}", num, url);
                            let document = Html::parse_document(&body);
                            match Selector::parse(".Si6A0c") {
                                Ok(selector) => {
                                    for element in document.select(&selector) {
                                        if let Some(path) = element.value().attr("href") {
                                            if path.starts_with(VALID_PATTERN) {
                                                let full_url =
                                                    format!("{}{}", PLAY_STORE_URL, path);
                                                if let Err(err) = sender.send(full_url.clone()) {
                                                    eprintln!(
                                                        "Task {}: Error sending URL: {:?}",
                                                        num, err
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(err) => {
                                    eprintln!("Task {}: Error parsing selector: {:?}", num, err);
                                    // Handle the error, e.g., break the loop or continue
                                }
                            }
                        }
                        Err(err) => {
                            eprintln!("Task {}: Error fetching URL: {:?}", num, err);
                            // Handle the error, e.g., break the loop or continue
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Task {}: Error receiving URL: {:?}", num, err);
                    // Handle the error, e.g., break the loop or continue
                }
            }
        }
    }

    pub async fn start_scraping(self, url: &str) -> Result<(), Error> {
        let (sender_clone, receiver_clone) = (self.sender.clone(), self.receiver.clone()); // Clone the sender and receiver
        let mut tasks = Vec::new();
        let crawler = Arc::new(self);

        sender_clone.send(url.to_string()).unwrap();
        for num in 0..5 {
            let sender_clone = sender_clone.clone();
            let receiver_clone = receiver_clone.clone();
            let crawler_clone = Arc::clone(&crawler);
            let task = tokio::spawn(async move {
                PlayStoreCrawler::manage(crawler_clone, sender_clone, receiver_clone, num).await;
            });

            tasks.push(task);
        }

        // Wait for all tasks to finish
        for task in tasks {
            task.await.expect("Task panicked");
        }

        Ok(())
    }

    async fn manage(
        crawler: Arc<Self>,
        sender: Sender<String>,
        receiver: Receiver<String>,
        num: i32,
    ) {
        crawler.scrape(sender, receiver, num).await;
    }
}
