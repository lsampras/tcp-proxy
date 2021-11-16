use std::{convert::Infallible, net::{IpAddr, SocketAddr}};
// use serde::{Deserialize, Serialize};
use serde_json;

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
	pub fn from_json(data: String) -> Result<Self, Infallible> {
		Ok(serde_json::from_str(&data).unwrap())
	}
	pub fn to_json(&self) -> Result<String, Infallible> {
		Ok(serde_json::to_string(self).unwrap())
	}
}