//! CLI 命令公共模块
use nanoimage_core::OptimizerConfig;
use std::path::PathBuf;

/// 获取配置目录
pub fn config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("nanoimage"))
}

/// 获取配置文件路径
pub fn config_path() -> Option<PathBuf> {
    config_dir().map(|p| p.join("config.json"))
}

/// 从默认位置加载配置
pub fn load_config() -> OptimizerConfig {
    if let Some(path) = config_path() {
        if path.exists() {
            if let Ok(config) = OptimizerConfig::load_from_file(&path) {
                return config;
            }
        }
    }

    // 回退到当前目录
    if let Ok(config) = OptimizerConfig::load_from_file(&PathBuf::from("config.json")) {
        return config;
    }

    OptimizerConfig::default()
}

/// 保存配置到默认位置
pub fn save_config(config: &OptimizerConfig) -> anyhow::Result<()> {
    if let Some(dir) = config_dir() {
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("config.json");
        config.save_to_file(&path)?;
    }
    Ok(())
}
