use crate::component::util::{hignlight::highlight_json_with_search, search::SearchBar};
use dioxus::prelude::*;

#[component]
pub fn JsonFormatterTool() -> Element {
    // ------- 编辑缓冲区（左侧输入） -------
    let mut input = use_signal(String::new);

    // ------- 输出与错误 -------
    let mut output = use_signal(String::new);
    let mut error_message = use_signal(String::new);

    // ------- 查找/替换状态（输入面板） -------
    let mut input_show_find = use_signal(|| false);
    let mut input_show_replace = use_signal(|| false);
    let mut input_find_query = use_signal(String::new);
    let input_replace_query = use_signal(String::new);
    let mut input_match_positions = use_signal(Vec::<usize>::new);
    let mut input_current_match_idx = use_signal(|| 0usize);

    // ------- 查找/替换状态（输出面板） -------
    let mut output_show_find = use_signal(|| false);
    let mut output_show_replace = use_signal(|| false);
    let mut output_find_query = use_signal(String::new);
    let output_replace_query = use_signal(String::new);
    let mut output_match_positions = use_signal(Vec::<usize>::new);
    let mut output_current_match_idx = use_signal(|| 0usize);

    // 在"输出区（格式化后）"上做二次高亮（文本，非位置等价）
    let mut highlighted_output = use_signal(String::new);

    // 当前活动侧（不是“焦点”而是“光标/鼠标所在侧”）："input" 或 "output"
    let mut active_panel = use_signal(|| "input".to_string());

    // ========== 公用：根据输入刷新输出与高亮 ==========
    let mut repaint = {
        move || {
            let src = input.read().clone();
            if src.trim().is_empty() {
                output.set(String::new());
                highlighted_output.set(String::new());
                error_message.set(String::new());
                return;
            }

            match serde_json::from_str::<serde_json::Value>(&src) {
                Ok(v) => match serde_json::to_string_pretty(&v) {
                    Ok(formatted) => {
                        output.set(formatted.clone());
                        error_message.set(String::new());
                        let html =
                            highlight_json_with_search(&formatted, &output_find_query.read());
                        highlighted_output.set(html);
                    }
                    Err(e) => {
                        output.set(String::new());
                        highlighted_output.set(String::new());
                        error_message.set(format!("格式化错误: {}", e));
                    }
                },
                Err(e) => {
                    output.set(String::new());
                    highlighted_output.set(String::new());
                    error_message.set(format!("JSON 解析错误: {}", e));
                }
            }
        }
    };

    // 初次渲染
    {
        use_effect(move || repaint());
    }

    // ========== 输入面板查找逻辑 ==========
    let mut input_recompute_matches = {
        move || {
            let text = input.read().clone();
            let q = input_find_query.read().clone();
            let mut pos = Vec::<usize>::new();

            if !q.is_empty() && !text.is_empty() {
                let mut start = 0usize;
                while let Some(p) = text[start..].find(&q) {
                    pos.push(start + p);
                    start = start + p + q.len();
                }
            }
            input_current_match_idx.set(0);
            input_match_positions.set(pos);
        }
    };

    // 输入面板下一个/上一个匹配
    let mut input_next_match = {
        move |_| {
            let total = input_match_positions.read().len();
            if total == 0 {
                return;
            }
            let idx = *input_current_match_idx.read();
            input_current_match_idx.set((idx + 1) % total);
        }
    };
    let mut input_prev_match = {
        move |_| {
            let total = input_match_positions.read().len();
            if total == 0 {
                return;
            }
            let idx = *input_current_match_idx.read();
            input_current_match_idx.set(if idx == 0 { total - 1 } else { idx - 1 });
        }
    };

    // ========== 输入面板替换逻辑 ==========
    let mut input_replace_one = {
        move |_| {
            let q = input_find_query.read().clone();
            if q.is_empty() {
                return;
            }

            let mut text = input.read().clone();
            let positions = input_match_positions.read().clone();
            if positions.is_empty() {
                return;
            }

            let idx = *input_current_match_idx.read();
            let start = positions[idx];
            let end = start + q.len();

            text.replace_range(start..end, &input_replace_query.read());
            input.set(text);

            input_recompute_matches();
            repaint();
        }
    };

    let mut input_replace_all = {
        move |_| {
            let q = input_find_query.read().clone();
            if q.is_empty() {
                return;
            }
            let mut text = input.read().clone();
            let rep = input_replace_query.read().clone();

            if !text.is_empty() {
                text = text.replace(&q, &rep);
                input.set(text);
                input_recompute_matches();
                repaint();
            }
        }
    };

    // ========== 输出面板查找逻辑 ==========
    let mut output_recompute_matches = {
        move || {
            let text = output.read().clone();
            let q = output_find_query.read().clone();
            let mut pos = Vec::<usize>::new();

            if !q.is_empty() && !text.is_empty() {
                let mut start = 0usize;
                while let Some(p) = text[start..].find(&q) {
                    pos.push(start + p);
                    start = start + p + q.len();
                }
            }
            output_current_match_idx.set(0);
            output_match_positions.set(pos);
            repaint(); // 重新高亮显示
        }
    };

    // 输出面板下一个/上一个匹配
    let mut output_next_match = {
        move |_| {
            let total = output_match_positions.read().len();
            if total == 0 {
                return;
            }
            let idx = *output_current_match_idx.read();
            output_current_match_idx.set((idx + 1) % total);
        }
    };
    let mut output_prev_match = {
        move |_| {
            let total = output_match_positions.read().len();
            if total == 0 {
                return;
            }
            let idx = *output_current_match_idx.read();
            output_current_match_idx.set(if idx == 0 { total - 1 } else { idx - 1 });
        }
    };

    // ========== 键盘快捷键 ==========
    // Cmd/Ctrl+F：打开/聚焦查找（根据“活动侧”）
    // Cmd/Ctrl+H：展开替换（仅输入面板）
    // Cmd/Ctrl+G / Shift+Cmd/Ctrl+G：下一个/上一个
    let on_keydown = {
        move |e: Event<KeyboardData>| {
            let meta = e.modifiers().contains(Modifiers::META)
                || e.modifiers().contains(Modifiers::CONTROL);
            let shift = e.modifiers().contains(Modifiers::SHIFT);
            let alt = e.modifiers().contains(Modifiers::ALT);

            match e.code() {
                Code::KeyF if meta => {
                    e.stop_propagation();
                    if *active_panel.read() == "input" {
                        input_show_find.set(true);
                        input_show_replace.set(false);
                        input_find_query.set(String::new());
                        // 互斥关闭另一侧
                        output_show_find.set(false);
                        output_show_replace.set(false);
                    } else {
                        output_show_find.set(true);
                        output_show_replace.set(false);
                        output_find_query.set(String::new());
                        input_show_find.set(false);
                        input_show_replace.set(false);
                    }
                }
                Code::KeyH if meta => {
                    e.stop_propagation();
                    if *active_panel.read() == "input" {
                        input_show_find.set(true);
                        input_show_replace.set(true);
                        output_show_find.set(false);
                        output_show_replace.set(false);
                    } else {
                        // 输出侧不支持替换，你也可以把这一行改为 false
                        output_show_find.set(true);
                        output_show_replace.set(true);
                        input_show_find.set(false);
                        input_show_replace.set(false);
                    }
                }
                Code::KeyG if meta && !shift => {
                    e.stop_propagation();
                    if *active_panel.read() == "input" {
                        input_next_match(());
                    } else {
                        output_next_match(());
                    }
                }
                Code::KeyG if meta && shift => {
                    e.stop_propagation();
                    if *active_panel.read() == "input" {
                        input_prev_match(());
                    } else {
                        output_prev_match(());
                    }
                }
                Code::Enter if meta && !alt => {
                    // 替换当前（仅输入面板）
                    e.stop_propagation();
                    if *active_panel.read() == "input" {
                        input_replace_one(());
                    }
                }
                Code::Enter if meta && alt => {
                    // 全部替换（仅输入面板）
                    e.stop_propagation();
                    if *active_panel.read() == "input" {
                        input_replace_all(());
                    }
                }
                _ => {}
            }
        }
    };

    // ========== 输入变更：自动格式化 ==========
    let on_input_change = {
        move |e: Event<FormData>| {
            active_panel.set("input".to_string()); // 认为此时活动在左侧
            input.set(e.value().to_string());
            repaint();
        }
    };

    // ====== 渲染 ======
    rsx! {
        div {
            class: "tool-container",
            tabindex: "0",
            onkeydown: on_keydown,
            style: "display:flex; flex-direction:column; height:100%; background:#1e1e1e; color:#ccc;",

            div {
                class: "input-output-container",
                style: "display:flex; flex:1; padding:16px; gap:16px; overflow:hidden;",

                // 左侧：输入
                div {
                    class: "input-panel",
                    style: "flex:1; display:flex; flex-direction:column; border:1px solid #3c3c3c; border-radius:6px; overflow:hidden; user-select:none; -webkit-user-select:none; -moz-user-select:none; -ms-user-select:none;",

                    // —— 查找条（SearchBar）
                    SearchBar {
                        show: input_show_find,
                        show_replace: input_show_replace,
                        query: input_find_query,
                        replace: input_replace_query,
                        on_prev: move |_| input_prev_match(()),
                        on_next: move |_| input_next_match(()),
                        on_close: move |_| { input_show_find.set(false); input_show_replace.set(false); },
                        on_query_input: move |q| { input_find_query.set(q); input_recompute_matches(); },
                        on_replace_one: Some(EventHandler::new(move |_| input_replace_one(()))),
                        on_replace_all: Some(EventHandler::new(move |_| input_replace_all(()))),
                        replace_disabled: false,
                    }

                    // —— 标题
                    h3 {
                        style: "margin:0; padding:12px 16px; background:#2d2d30; font-size:14px; font-weight:600; border-bottom:1px solid #3c3c3c;",
                        "输入"
                    }

                    // —— 输入编辑器（给定 id，并更新 active_panel）
                    textarea {
                        id: "json-input",
                        style: "flex:1; background:#1e1e1e; color:#cccccc; border:none; padding:16px; resize:none; font-family:'Monaco','Consolas',monospace; font-size:13px; overflow:auto; line-height:1.5;",
                        value: "{input}",
                        oninput: on_input_change,
                        onclick: move |_| active_panel.set("input".to_string()),
                        onfocus: move |_| active_panel.set("input".to_string()),
                        onfocusin: move |_| active_panel.set("input".to_string()),
                        onmouseenter: move |_| active_panel.set("input".to_string()),
                        placeholder: "在此输入 JSON 数据..."
                    }
                }

                // 右侧：输出
                div {
                    class: "output-panel",
                    style: "flex:1; display:flex; flex-direction:column; border:1px solid #3c3c3c; border-radius:6px; overflow:hidden; user-select:none; -webkit-user-select:none; -moz-user-select:none; -ms-user-select:none;",

                    // —— 查找条（SearchBar，禁用替换）
                    SearchBar {
                        show: output_show_find,
                        show_replace: output_show_replace,
                        query: output_find_query,
                        replace: output_replace_query,
                        on_prev: move |_| output_prev_match(()),
                        on_next: move |_| output_next_match(()),
                        on_close: move |_| { output_show_find.set(false); output_show_replace.set(false); },
                        on_query_input: move |q| { output_find_query.set(q); output_recompute_matches(); },
                        on_replace_one: None,
                        on_replace_all: None,
                        replace_disabled: true,
                    }

                    h3 {
                        style: "margin:0; padding:12px 16px; background:#2d2d30; font-size:14px; font-weight:600; border-bottom:1px solid #3c3c3c;",
                        "输出"
                    }

                    // —— 高亮输出视图（可聚焦，更新 active_panel）
                    div {
                        id: "json-output",
                        tabindex: "0",
                        style: "flex:1; background:#1e1e1e; color:#cccccc; margin:0; padding:16px; overflow:auto; white-space:pre-wrap; font-family:'Monaco','Consolas',monospace; font-size:13px; line-height:1.5;",
                        dangerous_inner_html: "{highlighted_output.read().clone()}",
                        onclick: move |_| active_panel.set("output".to_string()),
                        onfocus: move |_| active_panel.set("output".to_string()),
                        onfocusin: move |_| active_panel.set("output".to_string()),
                        onmouseenter: move |_| active_panel.set("output".to_string()),
                    }
                }
            }

            // 错误信息
            if !error_message().is_empty() {
                div {
                    class: "error-message",
                    style: "padding: 12px 16px; color: #f48771; background: rgba(244, 135, 113, 0.1); border: 1px solid #f48771; border-radius: 4px; margin: 0 16px 16px; font-size: 13px;",
                    "{error_message}"
                }
            }
        }
    }
}
