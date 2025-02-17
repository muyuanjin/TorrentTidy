use std::sync::LazyLock;
use crate::log;

use crate::logger::LogUnwrap;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::task;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TorrentInfo {
    pub hash: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TorrentFile {
    pub name: String,
    pub index: u32,
}

#[derive(Serialize, Debug)]
pub struct RenameRequest {
    pub hash: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct RenameFileRequest {
    pub hash: String,
    #[serde(rename = "oldPath")]
    pub old_path: String,
    #[serde(rename = "newPath")]
    pub new_path: String,
}

pub async fn authenticate(
    client: &Client,
    webui_url: &str,
    username: &str,
    password: &str,
) -> Result<(), String> {
    let auth_url = format!("{}/api/v2/auth/login", webui_url);
    let auth_params = [("username", username), ("password", password)];
    let auth_response = client
        .post(&auth_url)
        .form(&auth_params)
        .send()
        .await
        .log_unwrap("Failed to authenticate with qBittorrent WebUI");

    if auth_response.status().is_success() {
        log!("Authentication successful");
        Ok(())
    } else {
        log!("Authentication failed: {}", auth_response.status());
        Err(format!("Authentication failed: {}", auth_response.status()))
    }
}

pub async fn get_torrent_info(
    client: &Client,
    webui_url: &str,
    torrent_hash: &str,
) -> Result<TorrentInfo, String> {
    // https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-4.1)#get-torrent-list
    let info_url = format!("{}/api/v2/torrents/info?hashes={}", webui_url, torrent_hash);
    let info_response = client
        .get(&info_url)
        .send()
        .await
        .log_unwrap("Failed to fetch torrent info");

    if !info_response.status().is_success() {
        return Err(format!(
            "Failed to fetch torrent info: {}",
            info_response.status()
        ));
    }

    let torrent_info: Vec<TorrentInfo> = info_response
        .json()
        .await
        .log_unwrap("Failed to parse torrent info");

    if torrent_info.is_empty() {
        return Err(format!("No torrent found with hash: {}", torrent_hash));
    }

    log!("Fetched torrent info for hash: {}", torrent_hash);
    Ok(torrent_info[0].clone())
}

pub async fn rename_torrent(
    client: &Client,
    webui_url: &str,
    torrent_hash: &str,
    rename_rules: &Vec<(Regex, &str)>,
) -> Result<(), String> {
    let torrent = get_torrent_info(client, webui_url, torrent_hash)
        .await
        .log_unwrap("Failed to get torrent info");

    let new_name = apply_rename_rules(&torrent.name, rename_rules);

    if torrent.name != new_name {
        let rename_url = format!("{}/api/v2/torrents/rename", webui_url);
        let rename_request = RenameRequest {
            hash: torrent.hash.clone(),
            name: new_name.to_string(),
        };
        let rename_response = client
            .post(&rename_url)
            .form(&rename_request)
            .send()
            .await
            .log_unwrap("Failed to rename torrent");

        if !rename_response.status().is_success() {
            return Err(format!(
                "Failed to rename torrent: {}",
                rename_response.status()
            ));
        }

        log!("Torrent renamed to: {}", new_name);
    }

    Ok(())
}

pub async fn rename_files(
    client: &Client,
    webui_url: &str,
    torrent_hash: &str,
    rename_rules: &Vec<(Regex, &str)>,
) -> Result<(), String> {
    // https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-4.1)#get-torrent-contents
    let files_url = format!("{}/api/v2/torrents/files?hash={}", webui_url, torrent_hash);
    let files_response = client
        .get(&files_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch torrent files: {}", e))?;

    if !files_response.status().is_success() {
        return Err(format!(
            "Failed to fetch torrent files: {}",
            files_response.status()
        ));
    }

    let torrent_files: Vec<TorrentFile> = files_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse torrent files: {}", e))?;

    let mut tasks = Vec::new();

    for file in torrent_files {
        let new_path = apply_rename_rules_to_file(&file.name, rename_rules);

        if file.name != new_path {
            let rename_file_url = format!("{}/api/v2/torrents/renameFile", webui_url);
            let rename_file_request = RenameFileRequest {
                hash: torrent_hash.to_string(),
                old_path: file.name.clone(),
                new_path: new_path.clone(),
            };
            let client = client.clone();

            // 使用 tokio::spawn 并发执行每个重命名请求
            let task = task::spawn(async move {
                let result = client
                    .post(&rename_file_url)
                    .form(&rename_file_request)
                    .send()
                    .await;
                match result {
                    Ok(response) if response.status().is_success() => {
                        log!("File renamed: {} -> {}", file.name, new_path);
                        Ok(())
                    }
                    Ok(response) => {
                        log!("Failed to rename file: {} -> {}", file.name, new_path);
                        Err(format!("Failed to rename file: {}", response.status()))
                    }
                    Err(e) => {
                        log!("Failed to rename file: {} -> {}", file.name, new_path);
                        Err(format!("Failed to rename file: {}", e))
                    }
                }
            });
            tasks.push(task);
        }
    }

    // 等待所有任务完成
    for task in tasks {
        if let Err(e) = task.await.map_err(|e| format!("Task failed: {}", e))? {
            log!("Error during renaming: {}", e);
        }
    }

    Ok(())
}

fn apply_rename_rules(name: &str, compiled_rules: &Vec<(Regex, &str)>) -> String {
    let mut new_name = name.to_string();

    for (re, replacement) in compiled_rules {
        new_name = re.replace_all(&new_name, *replacement).into_owned();
    }

    new_name.trim().to_string()
}

/// 将文件名应用重命名规则，不改变文件扩展名
fn apply_rename_rules_to_file(name: &str, compiled_rules: &Vec<(Regex, &str)>) -> String {
    let (mut stem, ext) = split_filename(name);

    // 仅对主名部分应用替换规则
    for (re, replacement) in compiled_rules {
        stem = re.replace_all(&stem, *replacement).into_owned();
    }

    let stem = stem.trim();

    // 重新组合主名和扩展名
    if ext.is_empty() {
        stem.to_string()
    } else {
        format!("{}.{}", stem, ext)
    }
}



/// 将文件名拆分为主名和扩展名 FILE_EXTENSION_SPLIT 
fn split_filename(filename: &str) -> (String, String) {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^(.*?)\.(tar\.(?:gz|xz|bz2)|cpio\.(?:gz|bz2)|(?:7z|rar|zip)\.\d{3}|[^.]+)$").unwrap()
    });

    RE.captures(filename)
        .map(|caps| (caps[1].to_string(), caps[2].to_string()))
        .unwrap_or_else(|| (filename.to_string(), String::new()))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_2025_02_17_16_36_27() {
        assert_eq!(split_filename(""), ("".into(), "".into()));
        assert_eq!(split_filename("."), (".".into(), "".into()));
        assert_eq!(split_filename("f"), ("f".into(), "".into()));
        assert_eq!(split_filename(".f"), ("".into(), "f".into()));
        assert_eq!(split_filename("f."), ("f.".into(), "".into()));
        assert_eq!(split_filename("a.b.c.d.f"), ("a.b.c.d".into(), "f".into()));
        assert_eq!(split_filename("abc.tar.gz"), ("abc".into(), "tar.gz".into()));
        assert_eq!(split_filename("abc.7z.001"), ("abc".into(), "7z.001".into()));
        assert_eq!(split_filename("file.with.dots.txt"), ("file.with.dots".into(), "txt".into()));
        assert_eq!(split_filename("no_extension"), ("no_extension".into(), "".into()));
    }
}