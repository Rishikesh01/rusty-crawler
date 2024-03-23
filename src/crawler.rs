use reqwest::Error;
use scraper::{Html, Selector};

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

    pub async fn scrape(&self, url: &str) -> Vec<String> {
        let mut urls: Vec<String> = vec![];
        let body = self.do_request(url).await.ok().unwrap();

        let document = Html::parse_document(&body);
        let selector = Selector::parse(".Si6A0c").unwrap();
        for element in document.select(&selector) {
            if let Some(path) = element.value().attr("href") {
                if path.starts_with(VALID_PATTERN) {
                    urls.push(path.to_string());
                }
            };
        }

        urls
    }
}
