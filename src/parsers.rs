// each parser will read from a json file `movie_ids.json`, `person_ids.json`,
// `tv_series_ids.json` each parser will return a vector of structs
// there is a new object for each line in the json file
// create a macro that will generate the code for each parser, so that we don't
// have to write it out

// the parser will take three objects
// - a `DATA` static string
// - a `struct` that will be the return type, that implements
//   `serde::Deserialize`
//   - each struct will have different fields
// - the path to the json file
#[macro_export]
macro_rules! impl_parser {
	($struct:ident, $path:ident) => {
		static DATA: &str = include_str!(concat!(
			env!("CARGO_MANIFEST_DIR"),
			"/data/",
			stringify!($path),
			".json"
		));

		pub fn parse() -> Vec<$struct> {
			let mut entries: Vec<$struct> = Vec::new();
			for line in DATA.lines() {
				let entry = serde_json::from_str::<$struct>(line);
				match entry {
					Ok(entry) => entries.push(entry),
					Err(e) => println!("Error parsing {}: {}\n{line}", stringify!($struct), e),
				}
			}
			entries
		}
	};
}

pub mod movie;
pub mod person;
pub mod tv_series;
