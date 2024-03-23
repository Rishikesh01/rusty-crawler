use reqwest::Error;
use scraper::{Html, Selector};

const PLAY_STORE_URL: &str = "https://play.google.com";
const VALID_PATTERN: &str = "/store/apps/details?id=";

pub struct PlayStoreCrawler {
    client: reqwest::Client,
}

impl PlayStoreCrawler {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    async fn do_request(&self, url: &str) -> Result<String, Error> {
        match self.client.get(url).send().await?.text().await {
            Ok(body) => Ok(body),
            Err(e) => Err(e),
        }
    }

    pub async fn scrape(&self) {
        let body = self.do_request(PLAY_STORE_URL).await.ok().unwrap();

        let document = Html::parse_document(&body);
        let selector = Selector::parse(".Si6A0c").unwrap();
        println!("hello");
        for element in document.select(&selector) {
            match element.value().attr("href") {
                Some(url) if url.starts_with(VALID_PATTERN) => {
                    println!("{:?}", url);
                }
                Some(_) => (),
                None => (),
            };
        }
    }
}
