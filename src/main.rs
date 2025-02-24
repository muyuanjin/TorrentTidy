mod logger;
mod q_bit;
mod re;

use crate::logger::LogUnwrap;
use crate::re::CompoundReplacer;
use clap::Parser;
use reqwest::Client;
use tokio::task::JoinSet;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, required=true, value_name = "URL", help = "URL of the qBittorrent WebUI")]
    webui_url: String,
    #[arg(short, long, required=true, value_name = "HASH", help = "Hash of the torrent to rename")]
    torrent_hash: String,
    #[arg(short, long, required=true, value_name = "PATTERN=REPLACEMENT", help = "Rename rules in the format 'pattern=replacement',or points to a file one rule for every two lines")]
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
    if let Some(log_path) = args.log { logger::set_log_file(log_path) }
    // 提取参数 重命名规则，提前编译正则表达式
    let mut rules = vec![];
    for rule in args.rename_rules {
        if let Some((p,r)) = rule.rsplit_once('=') {
            rules.push((p.to_string(), r.to_string()));
        } else {
            // 如果没有等号，则认为是文件路径
            let content = std::fs::read_to_string(&rule)
                .log_unwrap("Failed to read rename rules file");
            rules.extend(
                content.lines()
                    .collect::<Vec<_>>()
                    .chunks_exact(2)
                    .map(|c| (c[0].to_string(), c[1].to_string()))
            );
        }
    }
    
    let mut builder = Client::builder().cookie_store(true);
    if !args.vpn { builder = builder.no_proxy(); }
    let client = Box::leak(Box::new(builder.build().unwrap()));
    let webui_url = Box::leak(Box::new(args.webui_url));
    let torrent_hash = Box::leak(Box::new(args.torrent_hash));
    let compound_replacer = Box::leak(Box::new(CompoundReplacer::new(rules)));

    // 如果提供了用户名和密码，则进行认证
    if let (Some(u), Some(p)) = (args.username, args.password) {
        q_bit::authenticate(client, webui_url, &u, &p)
            .await
            .log_unwrap("Failed to authenticate with qBittorrent WebUI");
    } else {
        log!("Skipping authentication as username and/or password were not provided.");
    }

    let mut tasks = JoinSet::new();
    tasks.spawn(q_bit::rename_torrent(client, webui_url, torrent_hash, compound_replacer));
    tasks.spawn(q_bit::rename_files(client, webui_url, torrent_hash, compound_replacer));

    while let Some(res) = tasks.join_next().await {
        match res { 
            Ok(Ok(_)) => {},
            Ok(Err(e)) => log!("Task failed: {:?}", e),
            Err(_) => log!("Task failed"),
        }
    }
}