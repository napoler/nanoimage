//! 配置持久化模块
//!
//! 负责将 OptimizerConfig 保存到系统配置目录下的 nanoimage 子目录

use nanoimage_core::OptimizerConfig;
use std::path::{Path, PathBuf};

/// 返回配置目录路径
fn config_dir() -> PathBuf {
    dirs::config_dir()
        .map(|p| p.join("nanoimage"))
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| String::new());
            PathBuf::from(home).join(".config").join("nanoimage")
        })
}

/// 返回配置文件路径
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
                tracing::info!("加载配置: {}", path.display());
                config
            }
            Err(e) => {
                tracing::warn!("加载配置失败 (使用默认配置): {}", e);
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
            tracing::error!("创建配置目录失败: {}", e);
            return;
        }
    }

    if let Err(e) = config.save_to_file(&path) {
        tracing::error!("保存配置失败: {}", e);
    }
}

/// 从指定路径加载配置
///
/// 从给定文件路径加载 OptimizerConfig，失败时返回错误信息
#[allow(dead_code)]
pub fn load_config_from_path(path: &Path) -> Result<OptimizerConfig, String> {
    OptimizerConfig::load_from_file(path)
        .map_err(|e| format!("加载配置失败: {}", e))
}

/// 保存配置到指定路径
///
/// 将配置保存到给定文件路径，失败时返回错误信息
#[allow(dead_code)]
pub fn save_config_to_path(config: &OptimizerConfig, path: &Path) -> Result<(), String> {
    config.save_to_file(path)
        .map_err(|e| format!("保存配置失败: {}", e))
}
