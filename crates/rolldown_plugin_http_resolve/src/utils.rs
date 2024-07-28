use reqwest::Client;
use std::ffi::OsStr;
use std::path::Path;
use url::Url;

static SUPPORTED_EXTENSIONS: [&str; 6] = ["js", "jsx", "ts", "tsx", "css", "json"];

fn is_supported_extension(ext: &str) -> bool {
  SUPPORTED_EXTENSIONS.contains(&ext)
}

pub fn get_extension(url: &Url) -> Option<&str> {
  let path = url.path();
  let ext = Path::new(path).extension().and_then(OsStr::to_str);
  ext
}

fn is_supported_url(url: &Url) -> bool {
  let path = url.path();
  let ext = Path::new(path).extension().and_then(OsStr::to_str);
  ext.map_or(false, is_supported_extension)
}

pub fn valid_and_parse(url: &str) -> Option<Url> {
  let url = Url::parse(url).ok()?;
  if (url.scheme() == "http" || url.scheme() == "https") && is_supported_url(&url) {
    Some(url)
  } else {
    None
  }
}

pub async fn fetch_module(url: &Url) -> anyhow::Result<String> {
  println!("Rolldown is fetching module from: {url}");
  let client = Client::new();
  let res = client.get(url.as_str()).send().await?;
  let body = res.text().await?;
  Ok(body)
}
