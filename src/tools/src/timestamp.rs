use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use toolbox_core::{register_tool, Mode, ProcessError, ProcessErrorKind, ProcessResult, Tool};

pub struct TimestampTool;

impl Tool for TimestampTool {
    fn id(&self) -> &'static str {
        "ts"
    }
    fn name(&self) -> &'static str {
        "时间戳 工具"
    }
    fn modes(&self) -> &'static [Mode] {
        const MODES: &[Mode] = &[
            Mode {
                id: "to_human",
                label: "Unix → 人类可读",
            },
            Mode {
                id: "to_unix",
                label: "人类可读 → Unix",
            },
        ];
        MODES
    }
    fn default_mode(&self) -> &'static str {
        "to_human"
    }

    fn process(&self, mode_id: &str, input: &str) -> ProcessResult {
        match mode_id {
            "to_human" => ts_to_human(input),
            "to_unix" => human_to_unix(input),
            _ => Err(ProcessError {
                kind: ProcessErrorKind::Unsupported,
                message: "不支持的模式".into(),
            }),
        }
    }
}

fn ts_to_human(input: &str) -> ProcessResult {
    let raw = input.trim().replace(['_', ','], "");
    if raw.is_empty() {
        return Ok(String::new());
    }
    let n: i128 = raw.parse().map_err(|_| ProcessError {
        kind: ProcessErrorKind::Parse,
        message: "请输入整数型 Unix 时间戳（秒或毫秒）".into(),
    })?;

    let (secs, nsecs) = if n.abs() > 10_000_000_000_000i128 {
        (n / 1000, (n % 1000) as i64 * 1_000_000)
    } else {
        (n, 0)
    };
    let dt_utc = Utc
        .timestamp_opt(secs as i64, nsecs as u32)
        .single()
        .ok_or_else(|| ProcessError {
            kind: ProcessErrorKind::Parse,
            message: "无法构造时间，请检查数值范围".into(),
        })?;
    let dt_local: DateTime<Local> = DateTime::from(dt_utc);

    Ok(format!(
        "UTC : {}\nLocal: {}",
        dt_utc.to_rfc3339(),
        dt_local.format("%Y-%m-%d %H:%M:%S%.3f %z")
    ))
}

fn human_to_unix(s: &str) -> ProcessResult {
    let s = s.trim();
    if s.is_empty() {
        return Ok(String::new());
    }

    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.timestamp().to_string());
    }
    const SEC: &str = "%Y-%m-%d %H:%M:%S";
    const MSEC: &str = "%Y-%m-%d %H:%M:%S%.3f";

    if let Ok(naive) = NaiveDateTime::parse_from_str(s, MSEC) {
        let dt = Local
            .from_local_datetime(&naive)
            .single()
            .ok_or_else(|| ProcessError {
                kind: ProcessErrorKind::Parse,
                message: "本地时间不唯一/无效（涉夏令时）".into(),
            })?;
        return Ok(dt.timestamp().to_string());
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(s, SEC) {
        let dt = Local
            .from_local_datetime(&naive)
            .single()
            .ok_or_else(|| ProcessError {
                kind: ProcessErrorKind::Parse,
                message: "本地时间不唯一/无效（涉夏令时）".into(),
            })?;
        return Ok(dt.timestamp().to_string());
    }

    Err(ProcessError {
        kind: ProcessErrorKind::Parse,
        message: "不支持的时间格式：RFC3339 或 \"YYYY-MM-DD HH:MM:SS[.sss]\"".into(),
    })
}

register_tool!(TimestampTool);
