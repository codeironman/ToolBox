use serde_json;
use toolbox_core::{register_tool, Mode, ProcessError, ProcessErrorKind, ProcessResult, Tool};

pub struct JsonTool;

impl Tool for JsonTool {
    fn id(&self) -> &'static str {
        "json"
    }
    fn name(&self) -> &'static str {
        "JSON 工具"
    }
    fn modes(&self) -> &'static [Mode] {
        const MODES: &[Mode] = &[
            Mode {
                id: "pretty",
                label: "Pretty（美化）",
            },
            Mode {
                id: "minify",
                label: "Minify（压缩）",
            },
        ];
        MODES
    }
    fn default_mode(&self) -> &'static str {
        "pretty"
    }

    fn process(&self, mode_id: &str, input: &str) -> ProcessResult {
        if input.trim().is_empty() {
            return Ok(String::new());
        }
        let v: serde_json::Value = serde_json::from_str(input).map_err(|e| ProcessError {
            kind: ProcessErrorKind::Parse,
            message: format!("JSON 解析错误: {e}"),
        })?;

        match mode_id {
            "pretty" => serde_json::to_string_pretty(&v).map_err(|e| ProcessError {
                kind: ProcessErrorKind::Other,
                message: format!("JSON 格式化错误: {e}"),
            }),
            "minify" => serde_json::to_string(&v).map_err(|e| ProcessError {
                kind: ProcessErrorKind::Other,
                message: format!("JSON 压缩错误: {e}"),
            }),
            _ => Err(ProcessError {
                kind: ProcessErrorKind::Unsupported,
                message: "不支持的模式".into(),
            }),
        }
    }
}

// 把这个工具注册进全局注册表
register_tool!(JsonTool);
