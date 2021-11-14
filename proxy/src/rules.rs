use std::convert::Infallible;
// use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Deserialize, Serialize)]
pub struct Rule {
	pub url_path: String,
	pub res: String
}

pub type Rules = Vec<Rule>;


#[derive(Debug, Deserialize, Serialize)]
pub struct Configuration {
	pub rules: Rules,
}

impl Configuration {
	pub fn new() -> Self {
		Configuration {
			rules: vec![]
		}
	}
	pub fn from_json(data: String) -> Result<Self, Infallible> {
		Ok(serde_json::from_str(&data).unwrap())
	}
	pub fn to_json(&self) -> Result<String, Infallible> {
		Ok(serde_json::to_string(self).unwrap())
	}
}