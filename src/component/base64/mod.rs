use ::base64::engine::general_purpose;
use ::base64::Engine;
use dioxus::prelude::*;

#[component]
pub fn Base64Tool() -> Element {
    // 状态
    let mut input = use_signal(|| "Hello, World!".to_string());
    let mut output = use_signal(String::new);
    let mut error_message = use_signal(String::new);

    let encode_now = move |_| {
        let src = input.read().clone();
        error_message.set(String::new());
        let encoded = general_purpose::STANDARD.encode(src.as_bytes());
        output.set(encoded);
    };

    let decode_now = move |_| {
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
    };

    let copy_output = move |_| {
        let text = output.read().clone();
        if text.is_empty() {
            return;
        }
        let escaped = serde_json::to_string(&text).unwrap_or_else(|_| "\"\"".to_string());
        let js = format!(
            "navigator.clipboard.writeText({t}).then(function(){{var b=document.getElementById('b64-copy-btn');if(b){{var o=b.textContent;b.textContent='已复制 ✓';setTimeout(function(){{b.textContent=o;}},1200);}}}}).catch(function(){{var b=document.getElementById('b64-copy-btn');if(b){{b.textContent='复制失败';setTimeout(function(){{b.textContent='复制';}},1200);}}}});",
            t = escaped
        );
        dioxus::document::eval(&js);
    };

    rsx! {
        div {
            class: "tool-container",
            style: "display:flex; flex-direction:column; height:100%; gap:14px; padding:14px; background:var(--bg-app);",

            // 输入 Panel
            div {
                class: "tb-panel",
                style: "flex:1; display:flex; flex-direction:column;",
                div {
                    class: "tb-panel-header",
                    span { style: "flex:1;", "输入" }
                }
                textarea {
                    class: "tb-textarea tb-scroll",
                    value: "{input}",
                    oninput: move |e| input.set(e.value().clone()),
                    placeholder: "在此输入待编码的文本，或粘贴一段 Base64 字符串用于解码…"
                }
            }

            // 中部操作区
            div {
                style: "display:flex; align-items:center; justify-content:center; gap:12px; user-select:none;",
                button { class: "tb-btn-primary", onclick: encode_now, "编码" }
                button { class: "tb-btn", onclick: decode_now, "解码" }
            }

            // 输出 Panel
            div {
                class: "tb-panel",
                style: "flex:1; display:flex; flex-direction:column;",
                div {
                    class: "tb-panel-header",
                    span { style: "flex:1;", "输出" }
                    button {
                        id: "b64-copy-btn",
                        class: "tb-btn-ghost",
                        style: "padding:3px 10px; font-size:11px;",
                        onclick: copy_output,
                        "复制"
                    }
                }
                pre {
                    class: "tb-scroll",
                    style: "flex:1; background:var(--bg-input); color:var(--text); margin:0; padding:14px; overflow:auto; white-space:pre-wrap; word-break:break-word; font-family:'Menlo','Monaco','Consolas',monospace; font-size:13px; line-height:1.6;",
                    "{output}"
                }
            }

            // 错误提示
            if !error_message().is_empty() {
                div {
                    style: "padding:10px 12px; color:var(--danger); background:rgba(244,135,113,.1); border:1px solid var(--danger); border-radius:var(--radius-sm); font-size:13px;",
                    "{error_message}"
                }
            }
        }
    }
}
