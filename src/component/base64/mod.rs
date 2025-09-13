mod find;
mod view;

use ::base64::engine::general_purpose;
use ::base64::Engine;
use dioxus::prelude::*;

#[component]
pub fn Base64Tool() -> Element {
    let mut input = use_signal(|| "Hello, World!".to_string());
    let mut output = use_signal(|| String::new());
    let mut mode = use_signal(|| "encode".to_string()); // "encode" or "decode"
    let mut error_message = use_signal(|| String::new());

    let process = move |_| {
        let input_str = input.read().clone();
        error_message.set(String::new());

        if *mode.read() == "encode" {
            let encoded = general_purpose::STANDARD.encode(input_str.as_bytes());
            output.set(encoded);
        } else {
            match general_purpose::STANDARD.decode(&input_str) {
                Ok(decoded) => match String::from_utf8(decoded) {
                    Ok(s) => output.set(s),
                    Err(e) => error_message.set(format!("解码错误: 无效的UTF-8序列: {}", e)),
                },
                Err(e) => error_message.set(format!("Base64 解码错误: 无效的Base64输入: {}", e)),
            }
        }
    };

    let get_button_style = |is_active: bool| {
        if is_active {
            "padding: 6px 12px; border: 1px solid #3c3c3c; border-radius: 2px; background: #007acc; color: #cccccc; cursor: pointer;"
        } else {
            "padding: 6px 12px; border: 1px solid #3c3c3c; border-radius: 2px; background: #2d2d30; color: #cccccc; cursor: pointer;"
        }
    };

    let is_encode = *mode.read() == "encode";
    let is_decode = !is_encode;

    rsx! {
        div {
            class: "tool-container",
            style: "display: flex; flex-direction: column; height: 100%;",

            div {
                class: "mode-toggle",
                style: "padding: 10px; display: flex; gap: 10px;",
                button {
                    style: "{get_button_style(is_encode)}",
                    onclick: move |_| mode.set("encode".to_string()),
                    "编码"
                }
                button {
                    style: "{get_button_style(is_decode)}",
                    onclick: move |_| mode.set("decode".to_string()),
                    "解码"
                }
            }

            div {
                class: "input-output-container",
                style: "display: flex; flex: 1; padding: 0 10px 10px; gap: 10px;",

                // 输入区域
                div {
                    class: "input-panel",
                    style: "flex: 1; display: flex; flex-direction: column; border: 1px solid #3c3c3c; border-radius: 4px; overflow: hidden;",
                    h3 {
                        style: "margin: 0; padding: 8px 12px; background: #2d2d30; font-size: 13px; font-weight: 600;",
                        "输入"
                    }
                    textarea {
                        style: "flex: 1; background: #1e1e1e; color: #cccccc; border: none; padding: 10px; resize: none; font-family: 'Monaco', 'Consolas', monospace; font-size: 12px;",
                        value: "{input}",
                        oninput: move |e| input.set(e.value().clone()),
                        placeholder: "在此输入数据..."
                    }
                }

                // 输出区域
                div {
                    class: "output-panel",
                    style: "flex: 1; display: flex; flex-direction: column; border: 1px solid #3c3c3c; border-radius: 4px; overflow: hidden;",
                    h3 {
                        style: "margin: 0; padding: 8px 12px; background: #2d2d30; font-size: 13px; font-weight: 600;",
                        "输出"
                    }
                    pre {
                        style: "flex: 1; background: #1e1e1e; color: #cccccc; margin: 0; padding: 10px; overflow: auto; white-space: pre-wrap; font-family: 'Monaco', 'Consolas', monospace; font-size: 12px;",
                        "{output}"
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

            // 操作按钮
            div {
                class: "actions",
                style: "padding: 0 10px 10px; display: flex; justify-content: flex-end; gap: 10px;",
                button {
                    style: "background: #007acc; color: white; border: none; padding: 6px 12px; border-radius: 2px; cursor: pointer;",
                    onclick: process,
                    "执行"
                }
            }
        }
    }
}
