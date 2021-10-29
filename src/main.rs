use futures::executor::block_on;
use std::env;
use reqwest::{Client, Method, Request, Url};

struct Options {
    urls: Vec<String>,
}

impl<I, T> From<I> for Options
where
    I: Iterator<Item = T>,
    T: Into<String>,

{
    fn from(val: I) -> Self {
        Self{urls: val.skip(1).map(|x| x.into()).collect()}
    }
}


fn main() {
    let options = Options::from(env::args());

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            for url in &options.urls {
                println!("URL {} contents: {}", url, fetch_url(url).await.unwrap());
            }
        });
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
        let args: Vec<String> = vec!["prog", "one", "two", "three"].iter().map(|&s| s.into()).collect();
        let options = Options::from(args.iter());
        assert_eq!(vec!["one", "two", "three"], options.urls);
    }

    #[test]
    pub fn test_options_from_empty_args() {
        let args: Vec<String> =  vec![];
        let options = Options::from(args.iter());
        assert_eq!(Vec::<String>::new(), options.urls);
    }
}
