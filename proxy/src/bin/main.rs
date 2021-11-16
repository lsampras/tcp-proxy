use std::fs;
use proxy::rules::{Configuration, Rule};
use proxy::web_service::ProxyServer;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Cli {
    #[structopt(name = "port", long = "--port")]
    port: Option<u16>,
    #[structopt(name = "rules_path", long = "--rules")]
    rules: Option<String>
}

pub fn read_config(file_name: String) -> String {
    fs::read_to_string(file_name).unwrap()
}

#[tokio::main]
async fn main() {
    // let rule = Rule {
    //     source: "127.0.0.1"
    // }

    let args = Cli::from_args();
    let config = match args.rules {
        Some(path) => Configuration::from_json(read_config(path)).unwrap(),
        None => Configuration::from_json(read_config(args.rules.unwrap_or(String::from("config.json")))).unwrap()
    };

    let mut server = ProxyServer::new(
        args.port.unwrap_or(3000),
        config
    );
    server.run().await;
}