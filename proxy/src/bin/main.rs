use std::fs;
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

    let args = Cli::from_args();

    let mut server = ProxyServer::new(
        args.port.unwrap_or(3000),
        args.rules.unwrap_or(String::from("test"))
    );
    server.run().await;
}