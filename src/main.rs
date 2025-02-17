mod logger;
mod q_bit;

use crate::logger::LogUnwrap;
use clap::{Arg, ArgAction, Command};
use regex::Regex;
use reqwest::Client;

#[tokio::main]
async fn main() {
    // 解析命令行参数
    let matches = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author("muyuanjin <muyuanjin@gmail.com>")
        .about("Renames torrents and files in qBittorrent")
        .arg(
            Arg::new("webui_url")
                .short('w')
                .long("webui-url")
                .required(true)
                .value_name("URL")
                .help("URL of the qBittorrent WebUI"),
        )
        .arg(
            Arg::new("username")
                .short('u')
                .long("username")
                .required(false)
                .value_name("USERNAME")
                .help("Username for qBittorrent WebUI authentication"),
        )
        .arg(
            Arg::new("password")
                .short('p')
                .long("password")
                .required(false)
                .value_name("PASSWORD")
                .help("Password for qBittorrent WebUI authentication"),
        )
        .arg(
            Arg::new("torrent_hash")
                .short('t')
                .long("torrent-hash")
                .required(true)
                .value_name("HASH")
                .help("Hash of the torrent to rename"),
        )
        .arg(
            Arg::new("rename_rules")
                .short('r')
                .long("rename-rules")
                .required(true)
                .action(ArgAction::Append)
                .value_name("PATTERN=REPLACEMENT")
                .help("Rename rules in the format 'pattern=replacement'"),
        )
        .arg(
            Arg::new("vpn")
                .short('v')
                .long("use-vpn")
                .required(false)
                .help("Use VPN for the request"),
        )
        .arg(
            Arg::new("log")
                .short('l')
                .long("log-file")
                .required(false)
                .help("Path to the log file"),
        )
        .get_matches();

    // 配置日志输出
    if let Some(log) = matches.get_one::<String>("log") {
        logger::set_log_file(log);
    }

    // 提取参数 WebUI 地址、用户名、密码
    let webui_url = matches
        .get_one::<String>("webui_url")
        .log_unwrap("Missing WebUI URL");
    let username = matches.get_one::<String>("username"); // 可选参数
    let password = matches.get_one::<String>("password"); // 可选参数

    // 提取参数 种子哈希
    let torrent_hash = matches
        .get_one::<String>("torrent_hash")
        .log_unwrap("Missing torrent hash");

    // 提取参数 重命名规则，提前编译正则表达式
    let rename_rules: Vec<(Regex, &str)> = matches
        .get_many::<String>("rename_rules")
        .log_unwrap("Missing rename rules")
        .map(|rule| {
            let index = rule.rfind('=').log_unwrap("Invalid rename rule: missing '='");
            let (pattern_part, replacement_with_eq) = rule.split_at(index);
            let replacement = &replacement_with_eq[1..];
            (
                Regex::new(pattern_part).log_unwrap("Invalid regex pattern"),
                replacement,
            )
        })
        .collect();

    let vpn = matches.contains_id("vpn");

    let client = if vpn {
        Client::new()
    } else {
        Client::builder()
            .no_proxy()
            .build()
            .log_unwrap("Failed to create reqwest client")
    };

    // 如果提供了用户名和密码，则进行认证
    if let (Some(username), Some(password)) = (username, password) {
        q_bit::authenticate(&client, webui_url, username, password)
            .await
            .log_unwrap("Failed to authenticate with qBittorrent WebUI");
    } else {
        println!("Skipping authentication as username and/or password were not provided.");
    }
    // 并行执行重命名种子和重命名文件
    let (_, _) = tokio::join!(
        q_bit::rename_torrent(&client, webui_url, torrent_hash, &rename_rules),
        q_bit::rename_files(&client, webui_url, torrent_hash, &rename_rules)
    );
}