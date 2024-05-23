use std::error::Error;
use std::{fs, io};
use std::path::{Path, PathBuf};
use directories::BaseDirs;
use glob::glob;
use serde::{Deserialize, Serialize};
use tracing::{info, error};

#[derive(Serialize, Deserialize)]
struct Version {
    version: String,
    hash: String,
}

fn find_subdirectory(parent_dir: &Path) -> io::Result<Option<PathBuf>> {
    for entry in fs::read_dir(parent_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

fn to_chinese() -> Result<(), Box<dyn Error>> {
    // 系统appdata路径
    let base_dirs = BaseDirs::new().ok_or("无法获取基本目录")?;
    let appdata_path = base_dirs.data_dir();

    let notion_dir = appdata_path.join("Notion");
    if !notion_dir.exists() || !notion_dir.exists() {
        return Err("Notion没有安装或当前版本不支持".into());
    }

    let notion_asset_dir_path = notion_dir.join("notionAssetCache-v2");
    if !notion_asset_dir_path.exists() {
        return Err("没有找到Notion资源缓存文件夹：notionAssetCache-v2".into());
    }

    // 获取版本信息
    let version_text: String;
    let notion_version_file_path = notion_asset_dir_path.join("latestVersion.json");
    if notion_version_file_path.exists() {
        let notion_version_text = fs::read(notion_version_file_path)?;
        let notion_version_data: Version = serde_json::from_slice(&notion_version_text)?;
        version_text = notion_version_data.version;
    } else {
        match find_subdirectory(notion_asset_dir_path.as_path())? {
            Some(path) => {
                version_text = path.file_name().unwrap().to_str().unwrap().to_string();
            }
            None => return Err("Notion资源缓存文件夹 notionAssetCache-v2 下是空的".into()),
        }
    }

    info!("Notion版本: {}", version_text);

    // notion版本下面的实际资源路径
    let notion_asset_dir_path = notion_asset_dir_path.join(version_text).join("assets").join("_assets");

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

    info!("notion 汉化成功，请按Ctrl+R刷新或重启, 按Enter退出");
    let mut temp = String::new();
    std::io::stdin().read_line(&mut temp).unwrap();
}
