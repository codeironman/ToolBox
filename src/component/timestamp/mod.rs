use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use dioxus::prelude::*;

#[component]
pub fn TimestampTool() -> Element {
    // --- 状态 ---
    // 上条：时间戳 -> 日期时间
    let mut ts_input = use_signal(current_unix_seconds_string);
    let mut dt_output_utc = use_signal(String::new);
    let mut dt_output_loc = use_signal(String::new);

    // 下条：日期时间 -> 时间戳
    let mut dt_input = use_signal(current_local_datetime_string);
    let mut ts_output_sec = use_signal(String::new);
    let mut ts_output_ms = use_signal(String::new);

    // 通用错误
    let mut error_message = use_signal(String::new);

    // 固定高度（结果框）
    const BOX_H: i32 = 32;

    // --- 操作：时间戳 -> 日期时间 ---
    let convert_ts_to_dt = move |_| {
        error_message.set(String::new());
        let raw = ts_input.read().trim().to_string();
        if raw.is_empty() {
            error_message.set("请输入时间戳（秒或毫秒）".into());
            return;
        }

        // 自动判断是秒还是毫秒
        let (secs, nanos) = match autodetect_ts_to_secs(&raw) {
            Ok((s, n)) => (s, n),
            Err(msg) => {
                error_message.set(msg);
                return;
            }
        };

        // UTC
        let Some(dt_utc) = DateTime::<Utc>::from_timestamp(secs, nanos) else {
            error_message.set("无效的时间戳".into());
            return;
        };

        // Local
        let dt_loc = dt_utc.with_timezone(&Local);

        dt_output_utc.set(dt_utc.format("%Y-%m-%d %H:%M:%S").to_string());
        dt_output_loc.set(dt_loc.format("%Y-%m-%d %H:%M:%S").to_string());
    };

    // --- 操作：日期时间 -> 时间戳 ---
    let convert_dt_to_ts = move |_| {
        error_message.set(String::new());
        let raw = dt_input.read().trim().to_string();
        if raw.is_empty() {
            error_message.set("请输入日期时间，格式：YYYY-MM-DD HH:MM:SS".into());
            return;
        }

        // 将输入按本地时区解析（常见使用场景更直觉）
        match NaiveDateTime::parse_from_str(&raw, "%Y-%m-%d %H:%M:%S") {
            Ok(naive) => {
                // 解释为 Local 时区
                let dt_local = Local.from_local_datetime(&naive).single().or_else(|| {
                    // 某些夏令时切换点可能有歧义，fallback 选择最早
                    Local.from_local_datetime(&naive).earliest()
                });

                let Some(dt_loc) = dt_local else {
                    error_message
                        .set("该本地时间在时区规则下可能不存在或不唯一，请调整后再试".into());
                    return;
                };

                let ts_sec = dt_loc.timestamp();
                let ts_ms = ts_sec.saturating_mul(1000);

                ts_output_sec.set(ts_sec.to_string());
                ts_output_ms.set(ts_ms.to_string());
            }
            Err(_) => error_message.set("日期格式错误，请使用：YYYY-MM-DD HH:MM:SS".into()),
        }
    };

    // --- UI 样式 ---
    let card_top = |title: &str| -> String {
        // 第一条卡片保留更紧凑的内边距/间距
        format!(
            "display:flex; flex-direction:column; gap:10px; \
             border:1px solid #343434; border-radius:10px; \
             background:linear-gradient(180deg,#232323,#1f1f1f); \
             padding:12px 14px 10px; box-shadow: 0 6px 18px rgba(0,0,0,0.25); \
             --title:'{title}';"
        )
    };
    let card = |title: &str| -> String {
        format!(
            "display:flex; flex-direction:column; gap:12px; \
             border:1px solid #343434; border-radius:10px; \
             background:linear-gradient(180deg,#232323,#1f1f1f); \
             padding:16px 16px 14px; box-shadow: 0 6px 18px rgba(0,0,0,0.25); \
             --title:'{title}';"
        )
    };
    let title_bar = "
        display:flex; align-items:center; justify-content:space-between;
        gap:8px; padding-bottom:6px; border-bottom:1px dashed #393939;
    ";
    let title_txt = "
        font-size:14px; font-weight:700; letter-spacing:.3px; color:#e8e8e8;
    ";
    let desc_txt = "
        font-size:11px; color:#a6a6a6;
    ";
    let label = "
        font-size:12px; color:#cfcfcf; margin-bottom:4px;
    ";
    let input_css = "
        background:#151515; color:#e0e0e0; border:1px solid #323232;
        outline:none; border-radius:8px; padding:8px 10px;
        font-family:'Monaco','Consolas',monospace; font-size:12px;
        transition:border-color .15s ease, box-shadow .15s ease;
    ";
    // 固定宽高的只读结果框
    let readonly_fixed = format!(
        "
    display:flex;
    align-items:center;     
    justify-content:flex-start;
    background:#0f0f0f;
    color:#d6d6d6;
    border:1px solid #2d2d2d;
    border-radius:8px;
    padding:0 12px;
    font-family:'Monaco','Consolas',monospace;
    font-size:12px;
    height:{box_h}px;
    overflow:hidden;
    white-space:nowrap;
    ",
        box_h = BOX_H
    );
    let btn = |primary: bool| -> String {
        if primary {
            "padding:8px 14px; border-radius:8px; border:none; \
             background:#0e7ad1; color:white; font-size:12px; \
             cursor:pointer; box-shadow:0 6px 14px rgba(14,122,209,.25); \
             transition:transform .05s ease; active:transform:translateY(1px);"
                .to_string()
        } else {
            "padding:8px 14px; border-radius:8px; border:1px solid #3a3a3a; \
             background:#252525; color:#d2d2d2; font-size:12px; cursor:pointer;"
                .to_string()
        }
    };

    rsx! {
        // 整体容器
        div {
            style: "height:100%; display:flex; flex-direction:column; gap:12px; padding:16px; background:#1b1b1b; color:#d0d0d0; overflow:auto;",

            // 顶部标题（居中）
            div {
                style: "display:grid; grid-template-columns: 1fr auto 1fr; align-items:center; margin-top:-2px;",
                div { style: "height:1px;" }
                // h2 {
                //     style: "margin:0; font-size:15px; font-weight:700; letter-spacing:.4px; color:#ededed;",
                //     "时间戳转换"
                // }
                div { style: "height:1px; justify-self:end;" }
            }

            // 第一条：时间戳 -> 日期时间（更紧凑）
            div {
                style: "{card_top(\"Timestamp → Datetime\")}",

                // 标题行（去掉副标题，减少留白）
                div {
                    style: "{title_bar}",
                    div {
                        style: "display:flex; align-items:center; gap:8px;",
                        span { style: "font-size:14px;", "⏱️" }
                        span { style: "{title_txt}", "时间戳 → 日期时间" }
                    }
                }

                // 输入
                div {
                    style: "display:flex; flex-direction:column; gap:4px;",
                    label { style: "{label}", "输入时间戳（秒或毫秒）" }
                    input {
                        style: "{input_css}",
                        value: "{ts_input}",
                        oninput: move |e| ts_input.set(e.value().to_string()),
                        placeholder: "例如：1700000000 或 1700000000000"
                    }
                }

                // 操作（紧凑）
                div {
                    style: "display:flex; align-items:center; gap:10px; padding-top:2px; padding-bottom:4px;",
                    button { style: "{btn(true)}", onclick: convert_ts_to_dt, "转换" }
                }

                // 结果区（固定宽高）
                div {
                    style: "display:grid; grid-template-columns: 1fr 1fr; gap:10px;",
                    div {
                        style: "display:flex; flex-direction:column; gap:4px;",
                        label { style: "{label}", "UTC" }
                        div { style: "{readonly_fixed}", "{dt_output_utc}" }
                    }
                    div {
                        style: "display:flex; flex-direction:column; gap:4px;",
                        label { style: "{label}", "本地（Local）" }
                        div { style: "{readonly_fixed}", "{dt_output_loc}" }
                    }
                }
            }

              div { style: "height:5px;" }

            // 第二条：日期时间 → 时间戳
            div {
                style: "{card(\"Datetime → Timestamp\")}",

                // 标题行
                div {
                    style: "{title_bar}",
                    div {
                        style: "display:flex; align-items:center; gap:8px;",
                        span { style: "font-size:14px;", "📅" }
                        span { style: "{title_txt}", "日期时间 → 时间戳" }
                    }
                    span { style: "{desc_txt}", "格式：YYYY-MM-DD HH:MM:SS（按本地时区解释）" }
                }

                // 输入
                div {
                    style: "display:flex; flex-direction:column; gap:4px;",
                    label { style: "{label}", "输入日期时间" }
                    input {
                        style: "{input_css}",
                        value: "{dt_input}",
                        oninput: move |e| dt_input.set(e.value().to_string()),
                        placeholder: "例如：2025-11-09 12:34:56"
                    }
                }

                // 操作
                div {
                    style: "display:flex; align-items:center; justify-content:space-between; padding-top:4px;",
                    button { style: "{btn(true)}", onclick: convert_dt_to_ts, "转换" }
                    button {
                        style: "{btn(false)}",
                        onclick: move |_| { dt_input.set(current_local_datetime_string()); },
                        "填入当前本地时间"
                    }
                }

                // 结果区（固定宽高）
                div {
                    style: "display:grid; grid-template-columns: 1fr 1fr; gap:10px;",
                    div {
                        style: "display:flex; flex-direction:column; gap:4px;",
                        label { style: "{label}", "时间戳（秒）" }
                        div { style: "{readonly_fixed}", "{ts_output_sec}" }
                    }
                    div {
                        style: "display:flex; flex-direction:column; gap:4px;",
                        label { style: "{label}", "时间戳（毫秒）" }
                        div { style: "{readonly_fixed}", "{ts_output_ms}" }
                    }
                }
            }

            // 错误提示
            if !error_message().is_empty() {
                div {
                    style: "margin-top:-2px; padding:10px 12px; border:1px solid #7f3a32; \
                            background:linear-gradient(180deg,rgba(127,58,50,.18),rgba(127,58,50,.08)); \
                            color:#ffb7a7; border-radius:10px; font-size:12px;",
                    "{error_message}"
                }
            }
        }
    }
}

// ============== 小工具函数 ==============

fn current_unix_seconds_string() -> String {
    let now = Utc::now();
    now.timestamp().to_string()
}

fn current_local_datetime_string() -> String {
    let now = Local::now();
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 自动判断输入是“秒”还是“毫秒”
/// 返回：（秒, 纳秒）
fn autodetect_ts_to_secs(s: &str) -> Result<(i64, u32), String> {
    if !s.chars().all(|c| c.is_ascii_digit()) {
        return Err("时间戳必须是纯数字（不含空格/小数/符号）".into());
    }
    let len = s.len();
    if len == 13 {
        // 毫秒
        let ms: i128 = s.parse().map_err(|_| "时间戳过大或无效")?;
        let secs = (ms / 1000) as i64;
        let rem_ms = (ms % 1000) as i64;
        let nanos = (rem_ms.max(0) as u32) * 1_000_000;
        Ok((secs, nanos))
    } else if len == 10 {
        // 秒
        let secs: i64 = s.parse().map_err(|_| "时间戳过大或无效")?;
        Ok((secs, 0))
    } else {
        // 自由长度：用数量级判断
        let v: i128 = s.parse().map_err(|_| "时间戳过大或无效")?;
        if v >= 1_000_000_000_000i128 {
            // >= 1e12 视为毫秒
            let secs = (v / 1000) as i64;
            let rem_ms = (v % 1000) as i64;
            let nanos = (rem_ms.max(0) as u32) * 1_000_000;
            Ok((secs, nanos))
        } else {
            // 视为秒
            Ok((v as i64, 0))
        }
    }
}
