use base64::{engine::general_purpose, Engine};
use toolbox_core::{register_tool, Mode, ProcessError, ProcessErrorKind, ProcessResult, Tool};

pub struct Base64Tool;

impl Tool for Base64Tool {
    fn id(&self) -> &'static str {
        "b64"
    }
    fn name(&self) -> &'static str {
        "Base64 工具"
    }
    fn modes(&self) -> &'static [Mode] {
        const MODES: &[Mode] = &[
            Mode {
                id: "encode",
                label: "Encode（编码）",
            },
            Mode {
                id: "decode",
                label: "Decode（解码）",
            },
        ];
        MODES
    }
    fn default_mode(&self) -> &'static str {
        "encode"
    }

    fn process(&self, mode_id: &str, input: &str) -> ProcessResult {
        match mode_id {
            "encode" => Ok(general_purpose::STANDARD.encode(input.as_bytes())),
            "decode" => {
                if input.trim().is_empty() {
                    return Ok(String::new());
                }
                let bytes = general_purpose::STANDARD
                    .decode(input.trim().as_bytes())
                    .map_err(|e| ProcessError {
                        kind: ProcessErrorKind::Decode,
                        message: format!("Base64 解码失败: {e}"),
                    })?;
                String::from_utf8(bytes).map_err(|e| ProcessError {
                    kind: ProcessErrorKind::Decode,
                    message: format!("UTF-8 解析失败: {e}"),
                })
            }
            _ => Err(ProcessError {
                kind: ProcessErrorKind::Unsupported,
                message: "不支持的模式".into(),
            }),
        }
    }
}

register_tool!(Base64Tool);
