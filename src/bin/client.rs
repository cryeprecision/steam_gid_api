use log::LevelFilter;
use reqwest::Client;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use steam_gid::ExpectLog;

const HELP_TEXT: &str = "\
This program fetches the following properties for Steam groups:
    Group ID 8, Group ID 64, Group Name and URL to Group
You can enter any of the following identifiers
    Group ID 8, Group ID 64, Full Group URL, Group URL Suffix";

#[tokio::main(flavor = "current_thread")]
async fn main() {
    TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();

    println!("{HELP_TEXT}");

    let client = Client::new();
    loop {
        let ident = steam_gid::prompt_input().await;
        let data = steam_gid::GroupData::fetch(&client, ident)
            .await
            .expect_log("couldn't get group data");
        println!("{data:#?}");
    }
}
