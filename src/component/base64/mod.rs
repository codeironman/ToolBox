use ::base64::engine::general_purpose;
use ::base64::Engine;
use dioxus::prelude::*;

#[component]
pub fn Base64Tool() -> Element {
    // 状态
    let mut input = use_signal(|| "Hello, World!".to_string());
    let mut output = use_signal(String::new);
    let mut error_message = use_signal(String::new);

    // 样式：卡片容器
    let panel_style = "
        display:flex; flex-direction:column;
        background:#1b1b1b; border:1px solid #2c2c2c;
        border-radius:8px; overflow:hidden;
        box-shadow:0 2px 8px rgba(0,0,0,0.12);
    ";

    // 样式：标题
    let header_style = "
        height:36px; background:#262626; border-bottom:1px solid #333;
        padding:0 12px; display:flex; align-items:center;
        font-size:13px; font-weight:600; color:#bdbdbd; user-select:none;
    ";

    // 样式：按钮（分段控制风格）
    let btn = |active: bool| -> &'static str {
        if active {
            "min-width:120px; height:36px; padding:0 16px;
             border:1px solid #005c9f; background:#0e6ab8;
             color:#fff; border-radius:8px; cursor:pointer;
             font-weight:600;user-select:none; -webkit-user-select:none; -moz-user-select:none; -ms-user-select:none;"
        } else {
            "min-width:120px; height:36px; padding:0 16px;
             border:1px solid #3a3a3a; background:#2a2a2a;
             color:#d0d0d0; border-radius:8px; cursor:pointer;user-select:none; -webkit-user-select:none; -moz-user-select:none; -ms-user-select:none;"
        }
    };

    let encode_now = {
        move |_| {
            let src = input.read().clone();
            error_message.set(String::new());
            let encoded = general_purpose::STANDARD.encode(src.as_bytes());
            output.set(encoded);
        }
    };

    let decode_now = {
        move |_| {
            let src = input.read().clone();
            error_message.set(String::new());
            match general_purpose::STANDARD.decode(&src) {
                Ok(bytes) => match String::from_utf8(bytes) {
                    Ok(s) => output.set(s),
                    Err(e) => {
                        error_message.set(format!("解码错误：不是有效的 UTF-8 文本（{}）", e))
                    }
                },
                Err(e) => error_message.set(format!("Base64 解码错误：输入无效（{}）", e)),
            }
        }
    };

    rsx! {
        div {
            class: "tool-container",
            style: "display:flex; flex-direction:column; height:100%; gap:14px; padding:14px; background:#1e1e1e;",

            // 输入 Panel
            div {
                class: "panel input",
                style: "{panel_style}",
                div { style: "{header_style}", "输入" }
                textarea {
                    style: "
                        flex:1; background:#1e1e1e; color:#dedede; border:none;
                        padding:14px; resize:none;
                        font-family: Menlo, Monaco, Consolas, monospace;
                        font-size:13px; line-height:1.6; outline:noneuser-select:none; -webkit-user-select:none; -moz-user-select:none; -ms-user-select:none;;
                    ",
                    value: "{input}",
                    oninput: move |e| input.set(e.value().clone()),
                    placeholder: "在此输入待编码的文本，或粘贴一段 Base64 字符串用于解码…"
                }
            }

            // 中部操作区（两个按钮，居中）
            div {
              style: "
        display:flex; align-items:center;
        justify-content:space-evenly;  /* ✅ 自动均匀分布 */
        padding:16px 0;
        user-select:none;
    ",

                button { style: "{btn(true)}",  onclick: encode_now, "编码" }
                button { style: "{btn(false)}", onclick: decode_now, "解码" }
            }

            // 输出 Panel
            div {
                class: "panel output",
                style: "{panel_style}",
                div { style: "{header_style}", "输出" }
                pre {
                    style: "
                        flex:1; background:#1e1e1e; color:#dfe6ef; margin:0;
                        padding:14px; overflow:auto; white-space:pre-wrap;
                        word-break:break-word;
                        font-family: Menlo, Monaco, Consolas, monospace;
                        font-size:13px; line-height:1.6user-select:none; -webkit-user-select:none; -moz-user-select:none; -ms-user-select:none;;
                    ",
                    "{output}"
                }
            }

            // 错误提示（可选）
            if !error_message().is_empty() {
                div {
                    style: "
                        padding:10px 12px; color:#f48771;
                        background:rgba(244, 135, 113, 0.10);
                        border:1px solid #f48771; border-radius:8px;
                    ",
                    "{error_message}"
                }
            }
        }
    }
}
