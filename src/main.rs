use futures::executor::block_on;
use std::env;
use std::error::Error;
use std::fmt::Display;
use reqwest::{Client, Method, Request, Url};

#[derive(Debug)]
struct Options {
    urls: Vec<String>,
    query: String,
}

impl<I, T> From<I> for Options
where
    I: Iterator<Item = T>,
    T: Into<String> + Display,

{
    fn from(val: I) -> Self {
        let mut _self = Self{query: "".to_owned(), urls: vec![]};
        let mut iter = val.skip(1);
        while let Some(item) = iter.next() {
            let item_str = item.into();
            match item_str.as_str() {
                "--urls" => {
                    let url_list = iter.next().unwrap().into();
                    _self.urls = url_list.split(",").map(|s| s.into()).collect();
                },
                _ => _self.query = item_str.into(),
            }
        }
        _self
    }
}

pub struct Match {
    url: String,
}

fn main() {
    let options = Options::from(env::args());

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            println!("Running with opts {:?}", options);
            let mut futures = vec![];
            for url in options.urls {
                futures.push(search_url(&options.query, url));
                // println!("URL {} contents: {}", url, fetch_url(url).await.unwrap());
            }

            for fut in futures {
                match fut.await {
                    Ok(Some(m)) => println!("Matched query {} on url {}", &options.query, m.url),
                    Ok(None) => println!("No match for {}", &options.query),
                    Err(e) => println!("Error?! {:?}", e),
                }
            }
        });
}


async fn search_url(query: &String, url: String) -> Result<Option<Match>, Box<dyn Error>> {
    let contents = fetch_url(&url).await?;
    if contents.contains(query) {
        Ok(Some(Match{url}))
    } else {
        Ok(None)
    }
}

async fn fetch_url(url: &String) -> Result<String, reqwest::Error> {
    println!("Checking url {}", url);
    let client = Client::builder().build().unwrap();
    let request = Request::new(Method::GET, Url::parse(url).unwrap());
    let resp_fut = client.execute(request).await;
    match resp_fut {
        Ok(response) => {
            let text = block_on(response.text())?;
            Ok(text)
        },
        Err(e) => Err(e)
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_options_from() {
        let args: Vec<String> = vec!["prog", "--urls", "one,two,three", "query"].iter().map(|&s| s.into()).collect();
        let options = Options::from(args.iter());
        assert_eq!(vec!["one", "two", "three"], options.urls);
        assert_eq!("query", options.query);
    }

    #[test]
    pub fn test_options_from_empty_args() {
        let args: Vec<String> =  vec![];
        let options = Options::from(args.iter());
        assert_eq!(Vec::<String>::new(), options.urls);
    }
}
