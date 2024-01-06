use anyhow::Result;
use sqlx::PgPool;

use crate::models::movie_credits::MovieCreditsResponse;
use crate::models::movie_details::MovieDetailsResponse;

pub async fn write_movie(db: &PgPool, movie_details: &MovieDetailsResponse) -> Result<String> {
	let movie_row = sqlx::query!(
		r#"
			insert into movie (
				tmdb_id, imdb_id, title,
				backdrop_path, poster_path, popularity
			) values ($1, $2, $3, $4, $5, $6)
			on conflict(tmdb_id)
			do update set
				popularity = EXCLUDED.popularity
			returning movie_id;
		"#,
		movie_details.id,
		movie_details.imdb_id,
		movie_details.title,
		movie_details.backdrop_path,
		movie_details.poster_path,
		movie_details.popularity,
	)
	.fetch_one(db)
	.await?;
	let movie_id = movie_row.movie_id;

	let _ = sqlx::query!("select from movie where movie_id = $1", movie_id)
		.fetch_one(db)
		.await?;

	Ok(movie_id)
}

pub async fn write_people(db: &PgPool, movie_row_id: &String, credits: MovieCreditsResponse) -> Result<()> {
	for cast in credits.cast {
		let _ = insert_person(
			db,
			movie_row_id,
			cast.id,
			cast.name,
			cast.profile_path,
			cast.popularity,
			cast.character,
		)
		.await;
	}

	for crew in credits.crew {
		let _ = insert_person(
			db,
			movie_row_id,
			crew.id,
			crew.name,
			crew.profile_path,
			crew.popularity,
			crew.job,
		)
		.await;
	}

	Ok(())
}

pub async fn fetch_start_index(db: &PgPool) -> Result<i64> {
	let progress = sqlx::query!(
		r#"
			select progress_id, movie_index from progress
			where created_at::date = current_date
			limit 1;
		"#
	)
	.fetch_optional(db)
	.await?;

	let index = if let Some(progress) = progress {
		progress.movie_index
	} else {
		let progress = sqlx::query!(
			r#"
		            insert into progress (movie_index)
		            values (0)
		            returning progress_id, movie_index;
		        "#
		)
		.fetch_one(db)
		.await?;

		progress.movie_index
	};

	Ok(index)
}

pub async fn insert_person(
	db: &PgPool,
	movie_row_id: &String,
	person_id: i32,
	name: String,
	profile_path: Option<String>,
	popularity: f32,
	character_or_job: String,
) -> Result<()> {
	let person_row = sqlx::query!(
		r#"
				insert into person (
					tmdb_id, name, profile_path, popularity
				) values ($1, $2, $3, $4)
				on conflict(tmdb_id)
				do update set
					name = EXCLUDED.name,
					profile_path = EXCLUDED.profile_path,
					popularity = EXCLUDED.popularity,
					updated_at = current_timestamp
				returning person_id;
			"#,
		person_id,
		name,
		profile_path,
		popularity
	)
	.fetch_one(db)
	.await?;
	let person_id = person_row.person_id;
	let _ = sqlx::query!("select from person where person_id = $1", person_id)
		.fetch_one(db)
		.await?;

	let movie_role_row = sqlx::query!(
		r#"
				insert into movie_role (
					movie_id, person_id, character_or_job
				) values ($1, $2, $3)
				returning role_id;
			"#,
		movie_row_id,
		person_id,
		character_or_job
	)
	.fetch_one(db)
	.await?;

	let role_id = movie_role_row.role_id;
	let _ = sqlx::query!("select from movie_role where role_id = $1", role_id)
		.fetch_one(db)
		.await?;

	Ok(())
}

pub async fn update_progress(db: &PgPool, index: i64) -> Result<()> {
	let _ = sqlx::query!(
		r#"
			update progress
			set movie_index = $1
			where created_at::date = current_date;
		"#,
		index
	)
	.execute(db)
	.await?;

	Ok(())
}
