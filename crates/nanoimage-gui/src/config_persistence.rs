//! 配置持久化模块
//!
//! 负责将 OptimizerConfig 保存到 ~/.hermes/nanoimage/config.json

use nanoimage_core::OptimizerConfig;
use std::path::{Path, PathBuf};

/// 返回配置目录路径: ~/.hermes/nanoimage/
fn config_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| String::new());
    PathBuf::from(home).join(".hermes").join("nanoimage")
}

/// 返回配置文件路径: ~/.hermes/nanoimage/config.json
fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

/// 从配置文件加载 OptimizerConfig
///
/// 如果文件不存在或加载失败，返回默认配置
pub fn load_config() -> OptimizerConfig {
    let path = config_path();
    if path.exists() {
        match OptimizerConfig::load_from_file(&path) {
            Ok(config) => {
                println!("加载配置: {}", path.display());
                config
            }
            Err(e) => {
                eprintln!("加载配置失败 (使用默认配置): {}", e);
                OptimizerConfig::default()
            }
        }
    } else {
        // 配置文件不存在，返回默认配置
        OptimizerConfig::default()
    }
}

/// 保存配置到文件
///
/// 如果保存失败，仅记录错误，不 panic
pub fn save_config(config: &OptimizerConfig) {
    let path = config_path();

    // 确保目录存在
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("创建配置目录失败: {}", e);
            return;
        }
    }

    if let Err(e) = config.save_to_file(&path) {
        eprintln!("保存配置失败: {}", e);
    }
}

/// 从指定路径加载配置
#[allow(dead_code)]
pub fn load_config_from_path(path: &Path) -> Result<OptimizerConfig, String> {
    OptimizerConfig::load_from_file(path)
        .map_err(|e| format!("加载配置失败: {}", e))
}

/// 保存配置到指定路径
#[allow(dead_code)]
pub fn save_config_to_path(config: &OptimizerConfig, path: &Path) -> Result<(), String> {
    config.save_to_file(path)
        .map_err(|e| format!("保存配置失败: {}", e))
}
