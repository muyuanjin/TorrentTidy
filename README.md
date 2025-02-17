# TorrentTidy 🧹

### 简介
TorrentTidy 是为 qBittorrent 设计的轻量级自动化清理工具，通过正则表达式在下载开始时重命名种子和文件。可有效清理文件名中的广告、冗余标识等无关内容，保持媒体库整洁。

### 功能特性
- 🚀 **自动化重命名**：触发下载即刻执行
- 🔍 **正则表达式替换**：支持多组自定义规则
- 🔒 **WebUI 集成**：使用 qBittorrent API 同时修改软件内任务名称和物理文件
- 📜 **日志记录**：可选文件日志跟踪操作

---

### Introduction
TorrentTidy is a lightweight automation tool for qBittorrent that intelligently renames torrents and files using regex patterns upon download initiation. Effectively cleans advertisements, redundant tags and irrelevant content in filenames.

### Features
- 🚀 **Auto-Renaming**：Instant execution upon download trigger
- 🔍 **Regex Replacement**：Multi-rule customization support
- 🔒 **WebUI Integration**：Use the qBittorrent API to modify the task name and physical files in the software simultaneously
- 📜 **Logging**：Optional file logging for operation tracking

---

## 🛠️ 安装/Installation

### 预编译二进制 Precompiled binary
从 [Release 页面](https://github.com/muyuanjin/TorrentTidy/releases) 下载对应平台的二进制文件 
Download the binary file of the corresponding platform from [Release page](https://github.com/muyuanjin/TorrentTidy/releases)

### 源码编译
```bash
# 编译项目
git clone https://github.com/muyuanjin/TorrentTidy.git
cd TorrentTidy
cargo build --release
```

---

## 🚦 使用方法/Usage

### QBittorrent 配置 QBittorrent Configuration
1. 进入 `Web UI`，勾选 `Web 用户界面（远程控制）`， 并记录端口号如 `8080`， 如果设置了用户名和密码，也需要传递给程序 Go to `Web UI`, check `Web User Interface (Remote Control)`, and note the port number such as `8080`. If a username and password are set, they also need to be passed to the program.
![QBittorrent WebUI](images/qBittorrent01.png)
2. 进入 `设置 -> 下载 -> 运行外部程序 -> 新增 Torrent 时运行` 输入框添加：Go to `Settings -> Downloads -> Run an External Program -> On Torrent Added` input box and add:
```bash
/path/to/torrent-tidy.exe -w "http://localhost:8080" -u "用户名" -p "密码" -t "%I" -r "规则1" -r "规则2"
```
如果没有设置用户名和密码，可以省略 `-u` 和 `-p` 参数 If no username and password are set, you can omit the `-u` and `-p` parameters.
![QBittorrent WebUI](images/qBittorrent02.png)

### 命令行参数 Command line parameters
```text
-w, --webui-url     [必需] qBittorrent WebUI 地址 [Required] qBittorrent WebUI address
-t, --torrent-hash  [必需] 种子哈希值 (使用 %I 占位符) [Required] Torrent hash (use %I placeholder)
-r, --rename-rules  [必需] 替换规则 (格式: 正则模式=替换文本)，支持多个 [Required] Replacement rules (format: regex pattern=replacement text), multiple supported
-u, --username      WebUI 用户名，如果设置了用户名密码则需要 WebUI username, required if username and password are set
-p, --password      WebUI 密码，如果设置了用户名密码则需要 WebUI password, required if username and password are set
-v, --use-vpn       是否通过 VPN 连接 qBittorrent Whether to connect to qBittorrent via VPN
-l, --log-file      日志文件路径，如果不设置则不记录日志 Log file path, if not set, no logging will be done
```

### 正则规则示例 Example of regular rules
```bash
# 清理常见发布组标识 Clean up common publish group logos
-r "[\[【].*?(电影|高清|原盘|蓝光|发布).*?[】\]]=" 

# 标准化分辨率标识 Standardized resolution marking
-r "(2160[pP])=4K" -r "(1080[pP])=FHD"

# 移除广告链接 Remove ad links
-r "\s*www\..+?(com|net)\s*="
```

---

## 📸 效果示例  Effect example

**原始文件名**
`【高清影视家园发布 www.XXX.com】小丑2：双重妄想[HDR+杜比视界双版本][中文字幕].2024.2160p.UHD.BluRay.Remux.DV.HEVC.TrueHD7.1-ParkHD`

**处理后文件名**
`小丑2：双重妄想[HDR+杜比视界双版本][中文字幕].2024.4K.UHD.BluRay.Remux.DV.HEVC.TrueHD7.1-ParkHD`

---

## 📄 许可证/License
MIT License © 2024 Muyuanjin