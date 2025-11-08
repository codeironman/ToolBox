use dioxus::core_macro::{component, rsx};
use dioxus::dioxus_core::Element;
use dioxus::prelude::*;
#[component]
pub fn TimestampTool() -> Element {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use std::time::{SystemTime, UNIX_EPOCH};

    let mut timestamp_input = use_signal(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_default()
    });

    let mut datetime_output = use_signal(String::new);
    let mut datetime_input = use_signal(String::new);
    let mut timestamp_output = use_signal(String::new);
    let mut error_message = use_signal(String::new);

    let convert_timestamp = move |_| {
        error_message.set(String::new());
        let ts_str = timestamp_input.read().clone();

        match ts_str.parse::<i64>() {
            Ok(timestamp) => {
                if let Some(dt) = DateTime::<Utc>::from_timestamp(timestamp, 0) {
                    let formatted = dt.format("%Y-%m-%d %H:%M:%S").to_string();
                    datetime_output.set(formatted);
                } else {
                    error_message.set("无效的时间戳".to_string());
                }
            }
            Err(_) => error_message.set("请输入有效的时间戳".to_string()),
        }
    };

    let convert_datetime = move |_| {
        error_message.set(String::new());
        let dt_str = datetime_input.read().clone();

        match NaiveDateTime::parse_from_str(&dt_str, "%Y-%m-%d %H:%M:%S") {
            Ok(dt) => {
                let dt_utc = dt.and_utc();
                let timestamp = dt_utc.timestamp();
                timestamp_output.set(timestamp.to_string());
            }
            Err(_) => {
                error_message.set("日期格式错误，请使用 YYYY-MM-DD HH:MM:SS 格式".to_string())
            }
        }
    };

    rsx! {
        div {
            class: "tool-container",
            style: "display: flex; flex-direction: column; height: 100%;",

            div {
                class: "input-output-container",
                style: "display: flex; flex: 1; padding: 10px; gap: 10px;",

                // 时间戳转日期
                div {
                    class: "timestamp-to-datetime",
                    style: "flex: 1; display: flex; flex-direction: column; border: 1px solid #3c3c3c; border-radius: 4px; overflow: hidden;",
                    h3 {
                        style: "margin: 0; padding: 8px 12px; background: #2d2d30; font-size: 13px; font-weight: 600;",
                        "时间戳转日期"
                    }
                    div {
                        style: "padding: 10px; display: flex; flex-direction: column; gap: 10px; flex: 1;",
                        input {
                            style: "background: #1e1e1e; color: #cccccc; border: 1px solid #3c3c3c; padding: 8px; font-family: 'Monaco', 'Consolas', monospace; font-size: 12px;",
                            value: "{timestamp_input}",
                            oninput: move |e| timestamp_input.set(e.value().clone()),
                            placeholder: "输入时间戳..."
                        }
                        button {
                            style: "align-self: flex-start; background: #007acc; color: white; border: none; padding: 6px 12px; border-radius: 2px; cursor: pointer;",
                            onclick: convert_timestamp,
                            "转换"
                        }
                        div {
                            style: "margin-top: 10px; padding: 10px; background: #1e1e1e; border: 1px solid #3c3c3c; border-radius: 4px; flex: 1;",
                            p {
                                style: "margin: 0; color: #cccccc; font-family: 'Monaco', 'Consolas', monospace; font-size: 12px;",
                                "结果: {datetime_output}"
                            }
                        }
                    }
                }

                // 日期转时间戳
                div {
                    class: "datetime-to-timestamp",
                    style: "flex: 1; display: flex; flex-direction: column; border: 1px solid #3c3c3c; border-radius: 4px; overflow: hidden;",
                    h3 {
                        style: "margin: 0; padding: 8px 12px; background: #2d2d30; font-size: 13px; font-weight: 600;",
                        "日期转时间戳"
                    }
                    div {
                        style: "padding: 10px; display: flex; flex-direction: column; gap: 10px; flex: 1;",
                        input {
                            style: "background: #1e1e1e; color: #cccccc; border: 1px solid #3c3c3c; padding: 8px; font-family: 'Monaco', 'Consolas', monospace; font-size: 12px;",
                            value: "{datetime_input}",
                            oninput: move |e| datetime_input.set(e.value().clone()),
                            placeholder: "格式: YYYY-MM-DD HH:MM:SS"
                        }
                        button {
                            style: "align-self: flex-start; background: #007acc; color: white; border: none; padding: 6px 12px; border-radius: 2px; cursor: pointer;",
                            onclick: convert_datetime,
                            "转换"
                        }
                        div {
                            style: "margin-top: 10px; padding: 10px; background: #1e1e1e; border: 1px solid #3c3c3c; border-radius: 4px; flex: 1;",
                            p {
                                style: "margin: 0; color: #cccccc; font-family: 'Monaco', 'Consolas', monospace; font-size: 12px;",
                                "结果: {timestamp_output}"
                            }
                        }
                    }
                }
            }

            // 错误信息显示
            if !error_message().is_empty() {
                div {
                    class: "error-message",
                    style: "padding: 10px; color: #f48771; background: rgba(244, 135, 113, 0.1); border: 1px solid #f48771; border-radius: 4px; margin: 0 10px 10px;",
                    "{error_message}"
                }
            }
        }
    }
}
