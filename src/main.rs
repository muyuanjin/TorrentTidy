mod logger;
mod q_bit;
mod re;

use crate::logger::LogUnwrap;
use crate::re::CompoundReplacer;
use clap::Parser;
use reqwest::Client;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, required=true, value_name = "URL", help = "URL of the qBittorrent WebUI")]
    webui_url: String,
    #[arg(short, long, required=true, value_name = "HASH", help = "Hash of the torrent to rename")]
    torrent_hash: String,
    #[arg(short, long, required=true, value_name = "PATTERN=REPLACEMENT", help = "Rename rules in the format 'pattern=replacement'")]
    rename_rules: Vec<String>,
    #[arg(short, long, required=false, value_name = "USERNAME", help = "Username for qBittorrent WebUI authentication")]
    username: Option<String>,
    #[arg(short, long, required=false, value_name = "PASSWORD", help = "Password for qBittorrent WebUI authentication")]
    password: Option<String>,
    #[arg(short, long, required=false, help = "Use VPN for the request")]
    vpn: bool,
    #[arg(short, long, required=false, value_name = "LOG_FILE_PATH", help = "Path to the log file")]
    log: Option<String>,
}

#[tokio::main]
async fn main() {
    // 解析命令行参数
    let args = Args::parse();
    // 配置日志输出
    if let Some(log) = args.log {
        logger::set_log_file(log);
    }
    // 提取参数 WebUI 地址、用户名、密码
    let webui_url = args.webui_url;
    // 提取参数 种子哈希
    let torrent_hash = args.torrent_hash;

    // 提取参数 重命名规则，提前编译正则表达式
    let compound_replacer = CompoundReplacer::new(args.rename_rules
        .iter()
        .map(|rule| {
            let index = rule.rfind('=').log_unwrap("Invalid rename rule: missing '='");
            let (pattern_part, replacement_with_eq) = rule.split_at(index);
            let replacement = &replacement_with_eq[1..];
            (
                pattern_part,
                replacement,
            )
        }));
    
    let mut builder = Client::builder().cookie_store(true);

    if !args.vpn {
        builder = builder.no_proxy();
    }

    let client = builder.build().log_unwrap("Failed to create http client");

    // 如果提供了用户名和密码，则进行认证
    if let (Some(username), Some(password)) = (args.username, args.password) {
        q_bit::authenticate(&client, &webui_url, &username, &password)
            .await
            .log_unwrap("Failed to authenticate with qBittorrent WebUI");
    } else {
        println!("Skipping authentication as username and/or password were not provided.");
    }
    // 并行执行重命名种子和重命名文件
    let (_, _) = tokio::join!(
        q_bit::rename_torrent(&client, &webui_url, &torrent_hash, &compound_replacer),
        q_bit::rename_files(&client, &webui_url, &torrent_hash, &compound_replacer)
    );
}