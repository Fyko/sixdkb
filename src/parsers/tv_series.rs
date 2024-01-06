use crate::impl_parser;

// {"id":5,"original_name":"La Job","popularity":6.301}
#[derive(serde::Deserialize, Debug)]
pub struct TvSeriesIdEntry {
	pub id: u32,
	#[serde(alias = "original_name")]
	#[serde(alias = "original_title")]
	pub title: String,
	pub popularity: f32,
}

impl_parser!(TvSeriesIdEntry, tv_series);
