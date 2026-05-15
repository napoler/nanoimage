//! 配置管理
use serde::{Deserialize, Serialize};

/// 输出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// 保持原格式
    #[default]
    KeepOriginal,
    Jpeg,
    Png,
    WebP,
    Gif,
}

impl OutputFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            OutputFormat::KeepOriginal => "keep",
            OutputFormat::Jpeg => "jpg",
            OutputFormat::Png => "png",
            OutputFormat::WebP => "webp",
            OutputFormat::Gif => "gif",
        }
    }
}

/// 压缩质量配置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quality {
    /// 有损压缩质量 1-100
    pub lossy: u8,
    /// 无损压缩质量 (用于 PNG)
    pub lossless: u8,
}

impl Default for Quality {
    fn default() -> Self {
        Self {
            lossy: 85,
            lossless: 100,
        }
    }
}

/// 优化器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizerConfig {
    /// 压缩模式
    #[serde(default)]
    pub mode: CompressionMode,
    /// 质量设置
    #[serde(default)]
    pub quality: Quality,
    /// 最大宽度 (None = 不限制)
    #[serde(default)]
    pub max_width: Option<u32>,
    /// 最大高度 (None = 不限制)
    #[serde(default)]
    pub max_height: Option<u32>,
    /// 输出格式
    #[serde(default)]
    pub format: OutputFormat,
    /// 保留元数据
    #[serde(default = "default_true")]
    pub preserve_metadata: bool,
    /// 覆盖源文件
    #[serde(default)]
    pub overwrite: bool,
    /// 输出目录 (None = 同目录 optimized 子文件夹)
    #[serde(default)]
    pub output_dir: Option<std::path::PathBuf>,
    /// 工作线程数
    #[serde(default = "default_workers")]
    pub workers: usize,
}

fn default_true() -> bool {
    true
}

fn default_workers() -> usize {
    std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(4)
        .min(16)
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            mode: CompressionMode::Lossy,
            quality: Quality::default(),
            max_width: None,
            max_height: None,
            format: OutputFormat::KeepOriginal,
            preserve_metadata: true,
            overwrite: false,
            output_dir: None,
            workers: default_workers(),
        }
    }
}

/// 压缩模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompressionMode {
    /// 有损压缩
    #[default]
    Lossy,
    /// 无损压缩
    Lossless,
    /// 智能模式 (根据情况自动选择)
    Smart,
}

impl OptimizerConfig {
    /// 从文件加载配置
    pub fn load_from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: OptimizerConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// 保存配置到文件
    pub fn save_to_file(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// 获取有效质量值
    pub fn effective_quality(&self) -> u8 {
        match self.mode {
            CompressionMode::Lossy => self.quality.lossy,
            CompressionMode::Lossless => self.quality.lossless,
            CompressionMode::Smart => self.quality.lossy,
        }
    }
}