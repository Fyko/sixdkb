use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieDetailsResponse {
	pub backdrop_path: Option<String>,
	pub id: i32,
	pub imdb_id: Option<String>,
	#[serde(default)]
	pub popularity: f32,
	pub poster_path: Option<String>,
	pub title: String,
}
