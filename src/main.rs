use std::error::Error;
use std::fs;
use std::path::PathBuf;
use directories::BaseDirs;
use glob::glob;
use serde::{Deserialize, Serialize};
use tracing::{info, error};

#[derive(Serialize, Deserialize)]
struct Version {
    version: String,
    hash: String,
}

fn to_chinese() -> Result<(), Box<dyn Error>> {
    // 系统appdata路径
    let base_dirs = BaseDirs::new().ok_or("无法获取基本目录")?;
    let appdata_path = base_dirs.data_dir();

    let notion_asset_dir_path = appdata_path.join("Notion").join("notionAssetCache-v2");
    let notion_version_file_path = notion_asset_dir_path.join("latestVersion.json");

    if !notion_asset_dir_path.exists() || !notion_version_file_path.exists() {
        return Err("Notion没有安装或当前版本不支持".into());
    }

    // 获取版本文件
    let notion_version_text = fs::read(notion_version_file_path)?;
    let notion_version_data: Version = serde_json::from_slice(&notion_version_text)?;
    info!("Notion版本: {}", notion_version_data.version);

    // notion版本下面的实际资源路径
    let notion_asset_dir_path = notion_asset_dir_path.join(notion_version_data.version).join("assets").join("_assets");

    // 查找 localeSetup-zh-CN-*.js 文件名字
    let notion_js_pattern = notion_asset_dir_path.join("localeSetup-zh-CN-*.js");
    let notion_js_file_name;
    let mut notion_js_file = glob(&notion_js_pattern.to_str().unwrap())?.filter_map(Result::ok);
    if let Some(path) = notion_js_file.next() {
        notion_js_file_name = path.file_name().unwrap().to_string_lossy().to_string();
    } else {
        return Err("找不到localeSetup-zh-CN-*.js文件".into());
    }
    info!("找到中文文件：{}", notion_js_file_name);

    // 查找 index-en-US-*.html 文件名字
    let notion_html_pattern = notion_asset_dir_path.join("index-en-US-*.html");
    let notion_html_file_path: PathBuf;
    let mut notion_html_file = glob(&notion_html_pattern.to_str().unwrap())?.filter_map(Result::ok);
    if let Some(path) = notion_html_file.next() {
        notion_html_file_path = path;
    } else {
        return Err("找不到index-en-US-*.html".into());
    }
    info!("找到启动html：{:?}", notion_html_file_path);

    // 替换为中文
    let mut notion_html_text = fs::read_to_string(&notion_html_file_path)?;
    let js_script = format!("<!doctype html><script defer=\"defer\" src=\"/_assets/{}\"></script>", notion_js_file_name);

    if notion_html_text.contains(&js_script) {
        info!("Notion已经经过汉化了");
        return Ok(());
    }

    notion_html_text = notion_html_text.replace("<!doctype html>", &js_script);
    fs::write(&notion_html_file_path, &notion_html_text)?;

    Ok(())
}

fn main() {
    tracing_subscriber::fmt::init();

    if let Err(e) = to_chinese() {
        error!("Notion汉化失败: {}", e);
        let mut temp = String::new();
        std::io::stdin().read_line(&mut temp).unwrap();
        return;
    }

    info!("notion 汉化成功，请按Ctrl+R刷新或重启");
    let mut temp = String::new();
    std::io::stdin().read_line(&mut temp).unwrap();
}
