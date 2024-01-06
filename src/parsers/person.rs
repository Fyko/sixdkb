use crate::impl_parser;

// {"adult":false,"id":27436,"name":"Costa-Gavras","popularity":2.807}
#[derive(serde::Deserialize, Debug)]
pub struct PersonIdEntry {
	pub adult: bool,
	pub id: u32,
	pub name: String,
	pub popularity: f32,
}

impl_parser!(PersonIdEntry, person);
