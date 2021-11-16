use std::{net::{IpAddr, SocketAddr}};
use serde_json::{self, Error};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Rule {
	pub source: IpAddr,
	pub res: String,
	pub destination: SocketAddr
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
	pub fn from_json(data: String) -> Result<Self, Error> {
		Ok(serde_json::from_str(&data)?)
	}
	pub fn to_json(&self) -> Result<String, Error> {
		Ok(serde_json::to_string(self)?)
	}
}