use reqwest::{header, Client};

pub mod backoff;
pub mod db;
pub mod fetchers;
pub mod models;
pub mod parsers;

pub type BoxDynError = Box<dyn std::error::Error + Send + Sync>;

pub fn create_tmdb_client() -> Client {
	let tmdb_access_token = std::env::var("TMDB_ACCESS_TOKEN").expect("TMDB_ACCESS_TOKEN must be set");

	let mut headers = header::HeaderMap::new();
	let mut auth_value = header::HeaderValue::from_str(&format!("Bearer {}", tmdb_access_token)).unwrap();
	auth_value.set_sensitive(true);
	headers.insert(header::AUTHORIZATION, auth_value);

	Client::builder().default_headers(headers).build().unwrap()
}
