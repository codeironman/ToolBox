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

    // 在“输出区（格式化后）”上叠加语法高亮 + 搜索高亮
    let mut highlighted_output = use_signal(String::new);

    // 当前活动侧："input" 或 "output"
    let mut active_panel = use_signal(|| "input".to_string());

    // ========== 公用：根据输入刷新输出（仅格式化，高亮由下方 effect 负责） ==========
    let mut repaint = {
        move || {
            let src = input.read().clone();
            if src.trim().is_empty() {
                output.set(String::new());
                error_message.set(String::new());
                return;
            }

            match serde_json::from_str::<serde_json::Value>(&src) {
                Ok(v) => match serde_json::to_string_pretty(&v) {
                    Ok(formatted) => {
                        output.set(formatted);
                        error_message.set(String::new());
                    }
                    Err(e) => {
                        output.set(String::new());
                        error_message.set(format!("格式化错误: {}", e));
                    }
                },
                Err(e) => {
                    output.set(String::new());
                    error_message.set(format!("JSON 解析错误: {}", e));
                }
            }
        }
    };

    // 初次渲染
    {
        use_effect(move || repaint());
    }

    // ========== 输出高亮 effect ==========
    // 依赖 output / output_find_query / output_current_match_idx：
    // 任一变化都重算匹配位置并重新高亮，因此“上一个/下一个”无需手动 repaint。
    // 注意：内部用本地 pos/cur，不回读 output_match_positions，避免循环。
    {
        use_effect(move || {
            let text = output.read().clone();
            let q = output_find_query.read().clone();
            let idx = *output_current_match_idx.read();

            let mut pos = Vec::<usize>::new();
            if !q.is_empty() && !text.is_empty() {
                let mut start = 0usize;
                while let Some(p) = text[start..].find(&q) {
                    pos.push(start + p);
                    start = start + p + q.len();
                }
            }
            let cur = if pos.is_empty() { 0 } else { idx.min(pos.len() - 1) };

            let html = highlight_json_with_search(&text, &q, &pos, cur);
            output_match_positions.set(pos);
            highlighted_output.set(html);
        });
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

    // 输入面板：选中并滚动到当前匹配（textarea 无法高亮，但可选中定位）
    use_effect(move || {
        let positions = input_match_positions.read().clone();
        let idx = *input_current_match_idx.read();
        let q = input_find_query.read().clone();
        if positions.is_empty() || q.is_empty() {
            return;
        }
        let idx = idx.min(positions.len() - 1);
        let start = positions[idx];
        let end = start + q.len();
        // 字节位置 -> 字符索引（JS setSelectionRange 用 UTF-16 索引；
        // 对 BMP 字符二者一致，含 emoji 等非 BMP 字符时略有偏差但不会报错）
        let text = input.read().clone();
        let s = text[..start].chars().count();
        let e = text[..end].chars().count();
        let js = format!(
            "(function(){{var el=document.getElementById('json-input');if(!el)return;el.focus();try{{el.setSelectionRange({s},{e});}}catch(_){{}}var lh=parseFloat(getComputedStyle(el).lineHeight)||20;var lines=el.value.slice(0,{s}).split('\\n').length-1;el.scrollTop=Math.max(0,lines*lh-el.clientHeight/3);}})();",
            s = s, e = e
        );
        dioxus::document::eval(&js);
    });

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

            let idx = (*input_current_match_idx.read()).min(positions.len().saturating_sub(1));
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
    // query 变化时重置到第一个匹配；高亮由输出 effect 自动更新。
    let mut output_recompute_matches = {
        move || {
            output_current_match_idx.set(0);
        }
    };

    // 输出面板下一个/上一个匹配（高亮由 effect 自动跟随）
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
    // Cmd/Ctrl+H：展开替换
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
                    e.stop_propagation();
                    if *active_panel.read() == "input" {
                        input_replace_one(());
                    }
                }
                Code::Enter if meta && alt => {
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
            active_panel.set("input".to_string());
            input.set(e.value().to_string());
            repaint();
            // 若输入面板正在查找，重新计算匹配位置（保持选中定位正确）
            if !input_find_query.read().is_empty() {
                input_recompute_matches();
            }
        }
    };

    // ========== 复制到剪贴板 ==========
    let copy_input = move |_| {
        let text = input.read().clone();
        if text.is_empty() {
            return;
        }
        let escaped = serde_json::to_string(&text).unwrap_or_else(|_| "\"\"".to_string());
        let js = format!(
            "navigator.clipboard.writeText({t}).then(function(){{var b=document.getElementById('copy-in-btn');if(b){{var o=b.textContent;b.textContent='已复制 ✓';setTimeout(function(){{b.textContent=o;}},1200);}}}}).catch(function(){{var b=document.getElementById('copy-in-btn');if(b){{b.textContent='复制失败';setTimeout(function(){{b.textContent='复制';}},1200);}}}});",
            t = escaped
        );
        dioxus::document::eval(&js);
    };
    let copy_output = move |_| {
        let text = output.read().clone();
        if text.is_empty() {
            return;
        }
        let escaped = serde_json::to_string(&text).unwrap_or_else(|_| "\"\"".to_string());
        let js = format!(
            "navigator.clipboard.writeText({t}).then(function(){{var b=document.getElementById('copy-out-btn');if(b){{var o=b.textContent;b.textContent='已复制 ✓';setTimeout(function(){{b.textContent=o;}},1200);}}}}).catch(function(){{var b=document.getElementById('copy-out-btn');if(b){{b.textContent='复制失败';setTimeout(function(){{b.textContent='复制';}},1200);}}}});",
            t = escaped
        );
        dioxus::document::eval(&js);
    };

    // ====== 渲染 ======
    rsx! {
        div {
            class: "tool-container",
            tabindex: "0",
            onkeydown: on_keydown,
            style: "display:flex; flex-direction:column; height:100%; background:var(--bg-app); color:var(--text);",

            div {
                class: "input-output-container",
                style: "display:flex; flex:1; padding:14px; gap:14px; overflow:hidden;",

                // 左侧：输入
                div {
                    class: "input-panel tb-panel",
                    style: "flex:1; display:flex; flex-direction:column;",

                    // -- 查找条
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
                        match_current: *input_current_match_idx.read(),
                        match_total: input_match_positions.read().len(),
                    }

                    // -- 标题栏 + 复制
                    div {
                        class: "tb-panel-header",
                        span { style: "flex:1;", "输入" }
                        button {
                            id: "copy-in-btn",
                            class: "tb-btn-ghost",
                            style: "padding:3px 10px; font-size:11px;",
                            onclick: copy_input,
                            "复制"
                        }
                    }

                    // -- 输入编辑器
                    textarea {
                        id: "json-input",
                        class: "tb-textarea tb-scroll",
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
                    class: "output-panel tb-panel",
                    style: "flex:1; display:flex; flex-direction:column;",

                    // -- 查找条（禁用替换）
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
                        match_current: *output_current_match_idx.read(),
                        match_total: output_match_positions.read().len(),
                    }

                    div {
                        class: "tb-panel-header",
                        span { style: "flex:1;", "输出" }
                        button {
                            id: "copy-out-btn",
                            class: "tb-btn-ghost",
                            style: "padding:3px 10px; font-size:11px;",
                            onclick: copy_output,
                            "复制"
                        }
                    }

                    // -- 高亮输出视图（可选中复制）
                    div {
                        id: "json-output",
                        class: "tb-scroll",
                        tabindex: "0",
                        style: "flex:1; background:var(--bg-input); color:var(--text); margin:0; padding:14px; overflow:auto; white-space:pre-wrap; word-break:break-word; font-family:'Menlo','Monaco','Consolas',monospace; font-size:13px; line-height:1.6;",
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
                    style: "padding:10px 12px; color:var(--danger); background:rgba(244,135,113,.1); border:1px solid var(--danger); border-radius:var(--radius-sm); margin:0 14px 14px; font-size:13px;",
                    "{error_message}"
                }
            }
        }
    }
}
