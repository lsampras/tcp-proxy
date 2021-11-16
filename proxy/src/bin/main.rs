use std::fs;
use proxy::rules::{Configuration};
use proxy::web_service::ProxyServer;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Cli {
    #[structopt(name = "rules_path", long = "--rules", default_value="config.json")]
    rules: String,
    #[structopt(name = "ports", long = "--ports", default_value = "3000")]
    ports: Vec<u16>,
    #[structopt(name = "config_port", long = "--config_port", default_value = "8001")]
    config_port: u16
}

pub fn read_config(file_name: String) -> String {
    fs::read_to_string(file_name).unwrap_or(String::new())
}

#[tokio::main]
async fn main() {

    let args = Cli::from_args();
    let config = Configuration::from_json(read_config(args.rules)).unwrap_or(Configuration::new());

    let server = ProxyServer::new(
        args.ports,
        config,
        args.config_port
    );
    server.run().await;
}