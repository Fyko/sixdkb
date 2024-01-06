use std::time::Duration;

use anyhow::Result;
use reqwest::Client;

use super::Governor;
use crate::backoff;
use crate::models::movie_credits::MovieCreditsResponse;
use crate::models::movie_details::MovieDetailsResponse;

#[inline]
pub async fn fetch_movie_details(client: &Client, governor: &Governor, id: u32) -> Result<MovieDetailsResponse> {
	// wait until we are let through!
	while governor.check().is_err() {}

	let response = client
		.get(format!("https://api.themoviedb.org/3/movie/{}", id))
		.send()
		.await?;

	let status = response.status().as_u16();
	let headers = response.headers().clone();
	let text = response.text().await?;

	if status > 299 {
		tracing::error!("failed to fetch movie details for id {id}: ({status}) {text} {headers:#?}");
		return Err(anyhow::anyhow!("failed to fetch movie details for id {}", id));
	}

	serde_json::from_str::<_>(&text).map_err(|e| {
		tracing::error!("failed to fetch movie details for id {}: {}", id, e);
		e.into()
	})
}

#[inline]
pub async fn fetch_movie_credits(client: &Client, governor: &Governor, id: u32) -> Result<MovieCreditsResponse> {
	// wait until we are let through!
	while governor.check().is_err() {}

	let mut res = None;
	for multiplier in backoff::new().take(10) {
		let response = client
			.get(format!("https://api.themoviedb.org/3/movie/{}/credits", id))
			.send()
			.await?;

		let status = response.status().as_u16();
		let headers = response.headers().clone();
		let text = response.text().await?;

		if status > 399 {
			let dur = Duration::from_secs(1) * (multiplier + 1);

			tracing::error!(
				"failed to fetch movie credits for movie {id}. retrying in {dur:.2?} ({status}) {text} {headers:#?}"
			);
			tokio::time::sleep(dur).await;
		} else {
			match serde_json::from_str::<_>(&text) {
				Ok(r) => {
					res = Some(r);
					break;
				}
				Err(e) => {
					tracing::error!("failed to fetch movie credits for id {}: {}", id, e);
					return Err(e.into());
				}
			}
		}
	}

	if let None = res {
		return Err(anyhow::anyhow!("failed to fetch movie credits for id {}", id));
	} else {
		return Ok(res.unwrap());
	}
}
