use crate::impl_parser;

// {"adult":false,"id":9,"original_title":"Sonntag im
// August","popularity":2.341,"video":false}
#[derive(serde::Deserialize, Debug)]
pub struct MovieIdEntry {
	pub adult: bool,
	pub id: u32,
	pub original_title: String,
	pub popularity: f32,
	pub video: bool,
}

impl_parser!(MovieIdEntry, movie);
