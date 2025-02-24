use crate::{log, re};

use crate::re::CompoundReplacer;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;

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

pub async fn authenticate(
    client: &Client,
    webui_url: &str,
    username: &str,
    password: &str,
) -> Result<(), String> {
    let auth_url = format!("{}/api/v2/auth/login", webui_url);
    let auth_params = [("username", username), ("password", password)];
    client
        .post(&auth_url)
        .form(&auth_params)
        .send()
        .await
        .and_then(|resp| resp.error_for_status())
        .map_err(|e| format!("Failed to authenticate: {}", e))?;

    log!("Authentication successful");
    Ok(())
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
        .and_then(|resp| resp.error_for_status())
        .map_err(|e| format!("Failed to fetch torrent info: {}", e))?;

    let torrent_info: Vec<TorrentInfo> = info_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse torrent info: {}", e))?;

    if torrent_info.is_empty() {
        return Err(format!("No torrent found with hash: {}", torrent_hash));
    }

    log!("Fetched torrent info for hash: {}", torrent_hash);
    Ok(torrent_info.into_iter().next().unwrap())
}

pub async fn get_torrent_files(
    client: &Client,
    webui_url: &str,
    torrent_hash: &str,
) -> Result<Vec<TorrentFile>, String> {
    // https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-4.1)#get-torrent-contents
    let files_url = format!("{}/api/v2/torrents/files?hash={}", webui_url, torrent_hash);
    let files_response = client
        .get(&files_url)
        .send()
        .await
        .and_then(|resp| resp.error_for_status())
        .map_err(|e| format!("Failed to fetch torrent files: {}", e))?;

    let torrent_files: Vec<TorrentFile> = files_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse torrent files: {}", e))?;

    Ok(torrent_files)
}

pub async fn rename_torrent(
    client: &Client,
    webui_url: &str,
    torrent_hash: &str,
    compound_replacer: &CompoundReplacer,
) -> Result<(), String> {
    let torrent = get_torrent_info(client, webui_url, torrent_hash).await?;
    let new_name = compound_replacer.replace(&torrent.name);

    if torrent.name != new_name {
        client
            .post(&format!("{}/api/v2/torrents/rename", webui_url))
            .form(&[("hash", &torrent.hash), ("name", &new_name)])
            .send()
            .await
            .and_then(|resp| resp.error_for_status())
            .map_err(|e| format!("Failed to rename torrent: {}", e))?;

        log!("Successfully renamed torrent to: {}", new_name);
    }

    Ok(())
}

pub async fn rename_files(
    client: &Client,
    webui_url: &str,
    torrent_hash: &str,
    compound_replacer: &CompoundReplacer,
) -> Result<(), String> {
    let torrent_files: Vec<TorrentFile> = get_torrent_files(client, webui_url, torrent_hash).await?;
    let rename_url = format!("{webui_url}/api/v2/torrents/renameFile");
    let mut tasks = JoinSet::new();

    // 并行处理每个文件重命名
    for file in torrent_files {
        let new_name = apply_rename_rules_to_file(&file.name, compound_replacer);
        if file.name == new_name {
            continue;
        }

        let (client, url, hash) = (client.clone(), rename_url.clone(), torrent_hash.to_owned());

        tasks.spawn(async move {
            // 发送重命名请求并处理响应
            let result = client
                .post(&url)
                .form(&[("hash", &hash), ("oldPath", &file.name), ("newPath", &new_name)])
                .send()
                .await
                .and_then(|resp| resp.error_for_status());
            // 返回处理结果与元数据
            (file.name, new_name, result)
        });
    }

    // 统一处理所有任务结果
    while let Some(res) = tasks.join_next().await {
        match res {
            Ok((old, new, Ok(_))) => log!("Success: {} -> {}",old,new),
            Ok((old, new, Err(e))) => log!("Failed: {} -> {} | {}",old,new,e),
            Err(e) => log!("Task execution failed: {}",e),
        }
    }
    Ok(())
}

/// 将文件名应用重命名规则，不改变文件扩展名
fn apply_rename_rules_to_file(name: &str, compound_replacer: &CompoundReplacer) -> String {
    let (stem, ext) = re::split_filename(name);

    // 仅对主名部分应用替换规则
    let stem = compound_replacer.replace(stem.as_str());

    // 重新组合主名和扩展名
    if ext.is_empty() {
        stem.to_string()
    } else {
        format!("{}.{}", stem, ext)
    }
}
