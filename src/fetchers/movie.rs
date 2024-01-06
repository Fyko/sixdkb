use std::time::Duration;

use anyhow::Result;
use reqwest::Client;
use sqlx::types::Json;
use sqlx::PgPool;

use super::Governor;
use crate::backoff;
use crate::models::movie_credits::MovieCreditsResponse;
use crate::models::movie_details::MovieDetailsResponse;

#[inline]
pub async fn fetch_movie_details(
	client: &Client,
	db: &PgPool,
	governor: &Governor,
	id: u32,
) -> Result<MovieDetailsResponse> {
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

	let json: MovieDetailsResponse = serde_json::from_str::<_>(&text)?;

	sqlx::query!(
		r#"insert into movie_details(id, data) values ($1, $2) on conflict do nothing;"#,
		id as i32,
		Json(&json) as _
	)
	.execute(db)
	.await?;

	Ok(json)
}

#[inline]
pub async fn fetch_movie_credits(
	client: &Client,
	db: &PgPool,
	governor: &Governor,
	id: u32,
) -> Result<MovieCreditsResponse> {
	// wait until we are let through!
	while governor.check().is_err() {}

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
			let json: MovieCreditsResponse = serde_json::from_str::<_>(&text)?;

			sqlx::query!(
				r#"insert into movie_credits(id, data) values ($1, $2) on conflict do nothing;"#,
				id as i32,
				Json(&json) as _
			)
			.execute(db)
			.await?;

			return Ok(json);
		}
	}

	Err(anyhow::anyhow!("failed to fetch movie credits for id {}", id))
}
