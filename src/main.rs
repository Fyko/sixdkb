#![allow(clippy::exit)]
use std::env;
use std::fmt::Write;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use console::Emoji;
use governor::{Quota, RateLimiter};
use indicatif::{HumanDuration, ProgressBar, ProgressState, ProgressStyle};
use nonzero_ext::nonzero;
use sixdkb::db::{fetch_start_index, update_progress, write_movie, write_people};
use sixdkb::fetchers::movie::{fetch_movie_credits, fetch_movie_details};
use sixdkb::{create_tmdb_client, parsers};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use tokio::sync::Semaphore;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter, Registry};

static SPARKLE: Emoji<'_, '_> = Emoji("✨ ", ":-)");

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
	Registry::default()
		.with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "sixdkb=debug".into()))
		.with(fmt::layer())
		.init();

	let movies = parsers::movie::parse();
	tracing::info!("Fetching {} movies...", movies.len());

	let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
	let mut opts: PgConnectOptions = database_url.parse().expect("failed to parse database url");
	opts = opts.application_name(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")));

	let pool = PgPoolOptions::new()
		.max_connections(70)
		.min_connections(10)
		.acquire_timeout(Duration::from_secs(8))
		.idle_timeout(Duration::from_secs(8))
		.max_lifetime(None);

	let db = pool.connect_with(opts).await.expect("Failed to connect to database");

	let tmdb_client = Arc::new(create_tmdb_client());
	let db = Arc::new(db);

	let started = Instant::now();
	let sem = Arc::new(Semaphore::new(15));
	// their limit is technically 50 per second but just to be safe (https://developer.themoviedb.org/docs/rate-limiting)
	let governor = Arc::new(RateLimiter::direct(Quota::per_second(nonzero!(45u32))));

	// check if we have a progress already running for this day, create one if not
	let start_index = fetch_start_index(&db).await?;

	let movies_len = (movies.len() - start_index as usize) as u64;
	if start_index > 0 {
		tracing::info!(
			"Resuming from index {} ({:.2}%)",
			start_index,
			(start_index as f64 / movies.len() as f64) * 100f64
		);
	}

	let pb = create_progress_bar(movies_len);

	// worker to save progress every 5 seconds
	tokio::spawn({
		let db = db.clone();
		let pb = pb.clone();
		async move {
			let mut interval = tokio::time::interval(Duration::from_secs(5));

			loop {
				interval.tick().await;
				let position: i64 = pb.position().try_into().expect("infallible");
				let _ = update_progress(&db.clone(), position + start_index as i64).await;
			}
		}
	});

	// worker to save progress on shutdown
	tokio::spawn({
		let pb = pb.clone();
		let db = db.clone();
		async move {
			shutdown_signal().await;
			pb.abandon_with_message("Shutting down...");

			let position: i64 = pb.position().try_into().expect("infallible");
			let index = position + start_index as i64;
			let _ = update_progress(&db, index).await;

			db.close().await;

			tracing::info!("Shutdown complete, saved progress (index: {index})");

			std::process::exit(0);
		}
	});

	let mut handles = vec![];
	for movie in movies.into_iter().skip(start_index as usize) {
		handles.push(tokio::spawn({
			let permit = sem.clone().acquire_owned().await?;
			let tmdb_client = tmdb_client.clone();
			let governor = governor.clone();
			let db = db.clone();
			let pb = pb.clone();

			async move {
				let movie_details = fetch_movie_details(&tmdb_client, &db, &governor, movie.id)
					.await
					.expect("failed to parse movie details");
				let credits = fetch_movie_credits(&tmdb_client, &db, &governor, movie.id)
					.await
					.expect("failed to parse credits");
				drop(permit);

				let movie = write_movie(&db, &movie_details).await.expect("failed to write movie");
				write_people(&db, &movie, credits)
					.await
					.expect("failed to write people");

				pb.inc(1);
			}
		}));
	}

	for handle in handles {
		handle.await?;
	}

	pb.abandon_with_message(format!("{} Done in {}", SPARKLE, HumanDuration(started.elapsed())));

	Ok(())
}

fn create_progress_bar(movies_len: u64) -> ProgressBar {
	let pb = ProgressBar::new(movies_len);

	pb.set_style(
		ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{bar:.cyan/blue}] {pos}/{len} ({eta})")
			.unwrap()
			.with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
				let secs = state.eta().as_secs_f64();
				if secs < 60f64 {
					write!(w, "{secs:.1}s").unwrap();
				} else if secs < 3600f64 {
					let mins = secs / 60f64;
					let secs = secs % 60f64;

					write!(w, "{mins:.0}m{secs:.0}s").unwrap();
				} else {
					let hours = secs / 3600f64;
					let mins = (secs % 3600f64) / 60f64;
					let secs = secs % 60f64;

					write!(w, "{hours:.0}h{mins:.0}m{secs:.0}s").unwrap();
				}
			})
			.progress_chars("#>-"),
	);

	pb
}

async fn shutdown_signal() {
	let ctrl_c = async {
		tokio::signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
	};

	#[cfg(unix)]
	let terminate = async {
		tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
			.expect("failed to install signal handler")
			.recv()
			.await;
	};

	#[cfg(not(unix))]
	let terminate = std::future::pending::<()>();

	tokio::select! {
		_ = ctrl_c => {},
		_ = terminate => {},
	}
}
