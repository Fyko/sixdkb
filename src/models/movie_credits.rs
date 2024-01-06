use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MovieCreditsResponse {
	pub id: i32,
	pub cast: Vec<CastMember>,
	pub crew: Vec<CrewMember>,
}

#[derive(Serialize, Deserialize)]
pub struct CastMember {
	pub id: i32,
	pub name: String,
	pub popularity: f32,
	pub profile_path: Option<String>,
	pub character: String,
}

#[derive(Serialize, Deserialize)]
pub struct CrewMember {
	pub id: i32,
	pub name: String,
	pub popularity: f32,
	pub profile_path: Option<String>,
	pub job: String,
}
