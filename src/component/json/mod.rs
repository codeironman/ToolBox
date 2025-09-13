mod view;

use dioxus::prelude::*;
use serde_json;

// VSCode 风格：在"当前面板"内处理查找/替换
#[component]
pub fn JsonFormatterTool() -> Element {
    // ------- 编辑缓冲区（左侧输入） -------
    let input = use_signal(|| String::new());

    // ------- 输出与错误 -------
    let output = use_signal(|| String::new());
    let error_message = use_signal(|| String::new());

    // ------- 查找/替换状态（输入面板） -------
    let mut input_show_find = use_signal(|| false);
    let input_show_replace = use_signal(|| false);
    let mut input_find_query = use_signal(|| String::new());
    let mut input_replace_query = use_signal(|| String::new());
    let input_match_positions = use_signal(|| Vec::<usize>::new());
    let input_current_match_idx = use_signal(|| 0usize);

    // ------- 查找/替换状态（输出面板） -------
    let mut output_show_find = use_signal(|| false);
    let output_show_replace = use_signal(|| false);
    let mut output_find_query = use_signal(|| String::new());
    let mut output_replace_query = use_signal(|| String::new());
    let output_match_positions = use_signal(|| Vec::<usize>::new());
    let output_current_match_idx = use_signal(|| 0usize);

    // 在"输出区（格式化后）"上做二次高亮（文本，非位置等价）
    let highlighted_output = use_signal(|| String::new());
    let mut focused_panel = use_signal(|| "input".to_string()); // "input" or "output"

    // ========== 公用：根据输入刷新输出与高亮 ==========
    let mut repaint = {
        let input = input.clone();
        let mut output = output.clone();
        let mut error_message = error_message.clone();
        let mut highlighted_output = highlighted_output.clone();
        let output_find_query = output_find_query.clone();

        move || {
            let src = input.read().clone();
            if src.trim().is_empty() {
                output.set(String::new());
                highlighted_output.set(String::new());
                error_message.set(String::new());
                return;
            }

            match serde_json::from_str::<serde_json::Value>(&src) {
                Ok(v) => {
                    match serde_json::to_string_pretty(&v) {
                        Ok(formatted) => {
                            output.set(formatted.clone());
                            error_message.set(String::new());

                            // 基于格式化文本做"二次高亮"与"当前命中"标注
                            let html =
                                highlight_json_with_search(&formatted, &output_find_query.read());
                            highlighted_output.set(html);
                        }
                        Err(e) => {
                            output.set(String::new());
                            highlighted_output.set(String::new());
                            error_message.set(format!("格式化错误: {}", e));
                        }
                    }
                }
                Err(e) => {
                    output.set(String::new());
                    highlighted_output.set(String::new());
                    error_message.set(format!("JSON 解析错误: {}", e));
                }
            }
        }
    };

    // 初次渲染
    use_effect({
        let mut repaint = repaint.clone();
        move || repaint()
    });

    // ========== 输入面板查找逻辑 ==========
    let input_recompute_matches = {
        let input = input.clone();
        let mut input_find_query = input_find_query.clone();
        let mut input_match_positions = input_match_positions.clone();
        let mut input_current_match_idx = input_current_match_idx.clone();

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

    // 输入面板查找 UI
    let mut input_do_find = {
        let mut input_recompute_matches = input_recompute_matches.clone();
        move |_| {
            input_recompute_matches();
        }
    };

    // 输入面板下一个/上一个匹配
    let mut input_next_match = {
        let input_match_positions = input_match_positions.clone();
        let mut input_current_match_idx = input_current_match_idx.clone();

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
        let input_match_positions = input_match_positions.clone();
        let mut input_current_match_idx = input_current_match_idx.clone();

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
    // 替换当前
    let mut input_replace_one = {
        let mut input = input.clone();
        let input_find_query = input_find_query.clone();
        let mut input_replace_query = input_replace_query.clone();
        let mut input_match_positions = input_match_positions.clone();
        let mut input_current_match_idx = input_current_match_idx.clone();
        let mut input_recompute_matches = input_recompute_matches.clone();
        let mut repaint = repaint.clone();

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

            // 执行替换
            text.replace_range(start..end, &input_replace_query.read());
            input.set(text);

            // 重新计算匹配（保持体验）
            input_recompute_matches();
            repaint();
        }
    };

    // 替换全部
    let mut input_replace_all = {
        let mut input = input.clone();
        let mut input_find_query = input_find_query.clone();
        let mut input_replace_query = input_replace_query.clone();
        let mut input_recompute_matches = input_recompute_matches.clone();
        let mut repaint = repaint.clone();

        move |_| {
            let q = input_find_query.read().clone();
            if q.is_empty() {
                return;
            }
            let mut text = input.read().clone();
            let rep = input_replace_query.read().clone();

            // 全部替换
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
        let output = output.clone();
        let mut output_find_query = output_find_query.clone();
        let mut output_match_positions = output_match_positions.clone();
        let mut output_current_match_idx = output_current_match_idx.clone();
        let mut repaint = repaint.clone();

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

    // 输出面板查找 UI
    let mut output_do_find = {
        let mut output_recompute_matches = output_recompute_matches.clone();
        move |_| {
            output_recompute_matches();
        }
    };

    // 输出面板下一个/上一个匹配
    let mut output_next_match = {
        let output_match_positions = output_match_positions.clone();
        let mut output_current_match_idx = output_current_match_idx.clone();

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
        let output_match_positions = output_match_positions.clone();
        let mut output_current_match_idx = output_current_match_idx.clone();

        move |_| {
            let total = output_match_positions.read().len();
            if total == 0 {
                return;
            }
            let idx = *output_current_match_idx.read();
            output_current_match_idx.set(if idx == 0 { total - 1 } else { idx - 1 });
        }
    };

    // ========== 键盘快捷键（只在本面板生效） ==========
    // - Cmd/Ctrl+F：打开/聚焦查找
    // - Cmd/Ctrl+H：展开替换（仅输入面板）
    // - Cmd/Ctrl+G / Shift+Cmd/Ctrl+G：下一个/上一个
    // - Enter / Shift+Enter：在查找框中下一个/上一个
    let on_keydown = {
        let mut input_show_find = input_show_find.clone();
        let mut input_show_replace = input_show_replace.clone();
        let mut output_show_find = output_show_find.clone();
        let mut output_show_replace = output_show_replace.clone();
        let mut input_next_match = input_next_match.clone();
        let mut input_prev_match = input_prev_match.clone();
        let mut output_next_match = output_next_match.clone();
        let mut output_prev_match = output_prev_match.clone();
        let mut input_replace_one = input_replace_one.clone();
        let mut input_replace_all = input_replace_all.clone();
        let mut input_find_query = input_find_query.clone();
        let mut output_find_query = output_find_query.clone();
        let focused_panel = focused_panel.clone();

        move |e: Event<KeyboardData>| {
            let meta = e.modifiers().contains(Modifiers::META)
                || e.modifiers().contains(Modifiers::CONTROL);
            let shift = e.modifiers().contains(Modifiers::SHIFT);
            let alt = e.modifiers().contains(Modifiers::ALT);

            match e.code() {
                Code::KeyF if meta => {
                    e.stop_propagation();
                    if *focused_panel.read() == "input" {
                        input_show_find.set(true);
                        input_show_replace.set(false);
                        input_find_query.set(String::new());
                    } else {
                        output_show_find.set(true);
                        output_show_replace.set(false);
                        output_find_query.set(String::new());
                    }
                }
                Code::KeyH if meta => {
                    e.stop_propagation();
                    if *focused_panel.read() == "input" {
                        input_show_find.set(true);
                        input_show_replace.set(true);
                    } else {
                        output_show_find.set(true);
                        output_show_replace.set(true);
                    }
                }
                Code::KeyG if meta && !shift => {
                    e.stop_propagation();
                    if *focused_panel.read() == "input" {
                        input_next_match(());
                    } else {
                        output_next_match(());
                    }
                }
                Code::KeyG if meta && shift => {
                    e.stop_propagation();
                    if *focused_panel.read() == "input" {
                        input_prev_match(());
                    } else {
                        output_prev_match(());
                    }
                }
                Code::Enter if meta && !alt => {
                    // 替换当前（仅输入面板）
                    e.stop_propagation();
                    if *focused_panel.read() == "input" {
                        input_replace_one(());
                    }
                }
                Code::Enter if meta && alt => {
                    // 全部替换（仅输入面板）
                    e.stop_propagation();
                    if *focused_panel.read() == "input" {
                        input_replace_all(());
                    }
                }
                _ => {}
            }
        }
    };

    // ========== 输入变更：自动格式化 ==========
    let on_input_change = {
        let mut input = input.clone();
        let mut repaint = repaint.clone();
        move |e: Event<FormData>| {
            input.set(e.value().to_string());
            repaint();
        }
    };

    // ========== 渲染 ==========
    rsx! {
        div {
            class: "tool-container",
            style: "display: flex; flex-direction: column; height: 100%; background-color: #1e1e1e; color: #cccccc; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;",
            tabindex: "0",
            onkeydown: on_keydown,

            // 主体：输入/输出
            div {
                class: "input-output-container",
                style: "display: flex; flex: 1; padding: 16px; gap: 16px; overflow: hidden;",

                // 输入
                div {
                    class: "input-panel",
                    style: "flex: 1; display: flex; flex-direction: column; border: 1px solid #3c3c3c; border-radius: 6px; overflow: hidden; box-shadow: 0 2px 4px rgba(0,0,0,0.2);",
                    onfocus: move |_| focused_panel.set("input".to_string()),
                    onfocusin: move |_| focused_panel.set("input".to_string()),
                    h3 {
                        style: "margin: 0; padding: 12px 16px; background: #2d2d30; font-size: 14px; font-weight: 600; flex: 0 0 auto; border-bottom: 1px solid #3c3c3c;",
                        "输入"
                    }

                    // 输入面板的搜索框
                    if *input_show_find.read() {
                        div {
                            style: "display: flex; flex-direction: column; gap: 0; padding: 0; background: #333333; border-bottom: 1px solid #3c3c3c;",

                            // 查找行
                            div {
                                style: "display: flex; gap: 0; align-items: center; padding: 6px;",
                                // 展开/收起替换行按钮
                                button {
                                    style: "background: transparent; color: #cccccc; border: none; width: 28px; height: 28px; cursor: pointer; display: flex; align-items: center; justify-content: center; font-size: 16px; border-radius: 4px; margin-right: 4px;",
                                    onclick: {
                                        let mut input_show_replace = input_show_replace.clone();
                                        move |_| {
                                            let current = *input_show_replace.read();
                                            input_show_replace.set(!current);
                                        }
                                    },
                                    if *input_show_replace.read() { "▼" } else { "▶" }
                                }
                                input {
                                    style: "flex: 1; background: #ffffff; color: #000000; border: 1px solid #454545; padding: 6px 10px; font-family: 'Monaco', 'Consolas', monospace; font-size: 13px; border-radius: 4px; margin-right: 6px;",
                                    value: "{input_find_query}",
                                    placeholder: "查找",
                                    oninput: move |e| input_find_query.set(e.value().clone()),
                                    onkeydown: move |e: KeyboardEvent| {
                                        if e.code() == Code::Enter && e.modifiers().is_empty() {
                                            e.stop_propagation();
                                            input_do_find(());
                                        } else if e.code() == Code::Enter && e.modifiers().contains(Modifiers::SHIFT) {
                                            e.stop_propagation();
                                            input_prev_match(());
                                        }
                                    }
                                }
                                button {
                                    style: "background: #3c3c3c; color: #cccccc; border: 1px solid #555; padding: 6px 10px; border-radius: 4px; cursor: pointer; font-size: 12px; margin-right: 4px;",
                                    onclick: move |_| input_prev_match(()),
                                    "↑"
                                }
                                button {
                                    style: "background: #3c3c3c; color: #cccccc; border: 1px solid #555; padding: 6px 10px; border-radius: 4px; cursor: pointer; font-size: 12px; margin-right: 4px;",
                                    onclick: move |_| input_next_match(()),
                                    "↓"
                                }
                                button {
                                    style: "background: #3c3c3c; color: #cccccc; border: 1px solid #555; padding: 6px 10px; border-radius: 4px; cursor: pointer; font-size: 12px; margin-right: 6px;",
                                    onclick: move |_| input_do_find(()),
                                    "🔍"
                                }
                                div {
                                    style: "color: #cccccc; font-size: 13px; min-width: 60px; text-align: center; padding: 0 6px;",
                                    {
                                        let total = input_match_positions.read().len();
                                        let idx = *input_current_match_idx.read();
                                        if total > 0 {
                                            format!("{}/{}", idx + 1, total)
                                        } else {
                                            "0/0".to_string()
                                        }
                                    }
                                }
                                button {
                                    style: "background: #3c3c3c; color: #cccccc; border: 1px solid #555; padding: 6px 10px; border-radius: 4px; cursor: pointer; font-size: 12px;",
                                    onclick: move |_| input_show_find.set(false),
                                    "✕"
                                }
                            }

                            // 替换行
                            if *input_show_replace.read() {
                                div {
                                    style: "display: flex; gap: 6px; align-items: center; padding: 6px; border-top: 1px solid #3c3c3c;",
                                    input {
                                        style: "flex: 1; background: #ffffff; color: #000000; border: 1px solid #454545; padding: 6px 10px; font-family: 'Monaco', 'Consolas', monospace; font-size: 13px; border-radius: 4px;",
                                        value: "{input_replace_query}",
                                        placeholder: "替换为",
                                        oninput: move |e| input_replace_query.set(e.value().clone()),
                                    }
                                    button {
                                        style: "background: #3c3c3c; color: #cccccc; border: 1px solid #555; padding: 6px 10px; border-radius: 4px; cursor: pointer; font-size: 12px;",
                                        onclick: move |_| input_replace_one(()),
                                        "替换"
                                    }
                                    button {
                                        style: "background: #007acc; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; font-size: 12px;",
                                        onclick: move |_| input_replace_all(()),
                                        "全部替换"
                                    }
                                }
                            }
                        }
                    }

                    div {
                        style: "flex: 1; display: flex; flex-direction: column; overflow: hidden;",
                        textarea {
                            style: "flex: 1; background: #1e1e1e; color: #cccccc; border: none; padding: 16px; resize: none; font-family: 'Monaco', 'Consolas', monospace; font-size: 13px; overflow: auto; line-height: 1.5;",
                            value: "{input}",
                            oninput: on_input_change, // 直接传回调
                            onfocus: move |_| focused_panel.set("input".to_string()),
                            onfocusin: move |_| focused_panel.set("input".to_string()),
                            placeholder: "在此输入 JSON 数据..."
                        }
                    }
                    div {
                        style: "padding: 12px; background: #2d2d30; display: flex; justify-content: flex-end; gap: 10px; flex: 0 0 auto; border-top: 1px solid #3c3c3c;",
                        button {
                            style: "background: #007acc; color: white; border: none; padding: 8px 16px; border-radius: 4px; cursor: pointer; font-size: 13px; font-weight: 500;",
                            onclick: move |_| repaint(),
                            "格式化"
                        }
                        button {
                            style: "background: #3c3c3c; color: #cccccc; border: 1px solid #3c3c3c; padding: 8px 16px; border-radius: 4px; cursor: pointer; font-size: 13px;",
                            onclick: move |_| input_show_find.set(true),
                            "查找 (Cmd/Ctrl+F)"
                        }
                    }
                }

                // 输出（格式化 + 高亮）
                div {
                    class: "output-search-container",
                    style: "flex: 1; display: flex; flex-direction: column; border: 1px solid #3c3c3c; border-radius: 6px; overflow: hidden; box-shadow: 0 2px 4px rgba(0,0,0,0.2);",
                    onfocus: move |_| focused_panel.set("output".to_string()),
                    onfocusin: move |_| focused_panel.set("output".to_string()),
                    div {
                        class: "output-panel",
                        style: "flex: 1; display: flex; flex-direction: column; overflow: hidden;",
                        h3 {
                            style: "margin: 0; padding: 12px 16px; background: #2d2d30; font-size: 14px; font-weight: 600; flex: 0 0 auto; border-bottom: 1px solid #3c3c3c;",
                            "输出"
                        }

                        // 输出面板的搜索框
                        if *output_show_find.read() {
                            div {
                                style: "display: flex; flex-direction: column; gap: 0; padding: 0; background: #333333; border-bottom: 1px solid #3c3c3c;",

                                // 查找行
                                div {
                                    style: "display: flex; gap: 0; align-items: center; padding: 6px;",
                                    // 展开/收起替换行按钮
                                    button {
                                        style: "background: transparent; color: #cccccc; border: none; width: 28px; height: 28px; cursor: pointer; display: flex; align-items: center; justify-content: center; font-size: 16px; border-radius: 4px; margin-right: 4px;",
                                        onclick: {
                                            let mut output_show_replace = output_show_replace.clone();
                                            move |_| {
                                                let current = *output_show_replace.read();
                                                output_show_replace.set(!current);
                                            }
                                        },
                                        if *output_show_replace.read() { "▼" } else { "▶" }
                                    }
                                    input {
                                        style: "flex: 1; background: #ffffff; color: #000000; border: 1px solid #454545; padding: 6px 10px; font-family: 'Monaco', 'Consolas', monospace; font-size: 13px; border-radius: 4px; margin-right: 6px;",
                                        value: "{output_find_query}",
                                        placeholder: "查找",
                                        oninput: move |e| output_find_query.set(e.value().clone()),
                                        onkeydown: move |e: KeyboardEvent| {
                                            if e.code() == Code::Enter && e.modifiers().is_empty() {
                                                e.stop_propagation();
                                                output_do_find(());
                                            } else if e.code() == Code::Enter && e.modifiers().contains(Modifiers::SHIFT) {
                                                e.stop_propagation();
                                                output_prev_match(());
                                            }
                                        }
                                    }
                                    button {
                                        style: "background: #3c3c3c; color: #cccccc; border: 1px solid #555; padding: 6px 10px; border-radius: 4px; cursor: pointer; font-size: 12px; margin-right: 4px;",
                                        onclick: move |_| output_prev_match(()),
                                        "↑"
                                    }
                                    button {
                                        style: "background: #3c3c3c; color: #cccccc; border: 1px solid #555; padding: 6px 10px; border-radius: 4px; cursor: pointer; font-size: 12px; margin-right: 4px;",
                                        onclick: move |_| output_next_match(()),
                                        "↓"
                                    }
                                    button {
                                        style: "background: #3c3c3c; color: #cccccc; border: 1px solid #555; padding: 6px 10px; border-radius: 4px; cursor: pointer; font-size: 12px; margin-right: 6px;",
                                        onclick: move |_| output_recompute_matches(),
                                        "🔍"
                                    }
                                    div {
                                        style: "color: #cccccc; font-size: 13px; min-width: 60px; text-align: center; padding: 0 6px;",
                                        {
                                            let total = output_match_positions.read().len();
                                            let idx = *output_current_match_idx.read();
                                            if total > 0 {
                                                format!("{}/{}", idx + 1, total)
                                            } else {
                                                "0/0".to_string()
                                            }
                                        }
                                    }
                                    button {
                                        style: "background: #3c3c3c; color: #cccccc; border: 1px solid #555; padding: 6px 10px; border-radius: 4px; cursor: pointer; font-size: 12px;",
                                        onclick: move |_| output_show_find.set(false),
                                        "✕"
                                    }
                                }

                                // 替换行 (输出面板不支持替换，但保持UI一致性)
                                if *output_show_replace.read() {
                                    div {
                                        style: "display: flex; gap: 6px; align-items: center; padding: 6px; border-top: 1px solid #3c3c3c;",
                                        input {
                                            style: "flex: 1; background: #454545; color: #999999; border: 1px solid #555; padding: 6px 10px; font-family: 'Monaco', 'Consolas', monospace; font-size: 13px; border-radius: 4px;",
                                            value: "{output_replace_query}",
                                            placeholder: "替换为",
                                            oninput: move |e| output_replace_query.set(e.value().clone()),
                                            disabled: true,
                                        }
                                        button {
                                            style: "background: #333; color: #666; border: 1px solid #444; padding: 6px 10px; border-radius: 4px; cursor: not-allowed; font-size: 12px;",
                                            disabled: true,
                                            "替换"
                                        }
                                        button {
                                            style: "background: #005a9e; color: #999; border: none; padding: 6px 12px; border-radius: 4px; cursor: not-allowed; font-size: 12px;",
                                            disabled: true,
                                            "全部替换"
                                        }
                                    }
                                }
                            }
                        }

                        div {
                            style: "flex: 1; background: #1e1e1e; color: #cccccc; margin: 0; padding: 16px; overflow: auto; white-space: pre-wrap; font-family: 'Monaco', 'Consolas', monospace; font-size: 13px; line-height: 1.5;",
                            dangerous_inner_html: "{highlighted_output.read().clone()}",
                            onfocus: move |_| focused_panel.set("output".to_string()),
                            onfocusin: move |_| focused_panel.set("output".to_string()),
                        }
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

/* ===================== 辅助渲染 ===================== */

// 在格式化 JSON 上做语法高亮并叠加"搜索高亮"
fn highlight_json_with_search(json: &str, query: &str) -> String {
    let mut html = highlight_json(json);

    if !query.is_empty() {
        // 当前命中：加强色
        // 注意：这里是"文本级"替换，若 query 含有 HTML 特殊字符，可先 escape，再在 highlight_json 前处理
        let strong = format!(
            "<span style=\"background:#ffcc00;color:#000;\">{}</span>",
            query
        );
        // 先替换一个为"当前命中"
        if let Some(pos) = html.find(query) {
            html = format!("{}{}{}", &html[..pos], strong, &html[pos + query.len()..]);
            // 再把剩余命中替换为浅色
            let weak = format!(
                "<span style=\"background:rgba(255,204,0,0.35);\">{}</span>",
                query
            );
            html = html.replacen(query, &weak, 0); // no-op 防御
            html = html.replace(query, &weak);
        }
    }
    html
}

// 语法高亮（与原先一致，可按需扩展数字等着色）
fn highlight_json(json: &str) -> String {
    let mut result = String::new();
    let mut chars = json.chars().peekable();
    let mut in_string = false;
    let mut is_key = false;
    let mut escape = false;

    while let Some(ch) = chars.next() {
        match ch {
            '"' if !escape => {
                in_string = !in_string;
                if in_string {
                    if is_key {
                        result.push_str("<span style=\"color: #9cdcfe\">\"");
                    } else {
                        result.push_str("<span style=\"color: #ce9178\">\"");
                    }
                } else {
                    result.push_str("\"</span>");
                    if is_key {
                        is_key = false;
                    }
                }
            }
            ':' if !in_string => {
                result.push(':');
                is_key = false;
            }
            '{' | '[' if !in_string => {
                result.push(ch);
                is_key = true;
            }
            '}' | ']' if !in_string => {
                result.push(ch);
            }
            ',' if !in_string => {
                result.push(ch);
                is_key = true;
            }
            't' if !in_string && chars.peek() == Some(&'r') => {
                result.push_str("<span style=\"color: #569cd6\">t");
                chars.next();
                chars.next();
                chars.next();
                result.push_str("rue</span>");
            }
            'f' if !in_string && chars.peek() == Some(&'a') => {
                result.push_str("<span style=\"color: #569cd6\">f");
                chars.next();
                chars.next();
                chars.next();
                chars.next();
                result.push_str("alse</span>");
            }
            'n' if !in_string && chars.peek() == Some(&'u') => {
                result.push_str("<span style=\"color: #569cd6\">n");
                chars.next();
                chars.next();
                chars.next();
                result.push_str("ull</span>");
            }
            '\\' if in_string => {
                escape = true;
                result.push(ch);
                continue;
            }
            _ => {
                result.push(ch);
            }
        }
        escape = false;
    }
    result
}
