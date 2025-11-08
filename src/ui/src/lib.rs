use dioxus::prelude::*;
use toolbox_core::registry;

#[component]
pub fn App() -> Element {
    // 当前工具 id 和 模式 id 保存在状态
    let tools = registry(); // &[Arc<dyn Tool>]
    let first_tool = tools.tools().first().expect("至少一个工具已注册").clone();

    let current_tool_id = use_signal(|| first_tool.id().to_string());
    let current_mode_id = use_signal(|| first_tool.default_mode().to_string());

    // 双面板状态
    let input = use_signal(|| String::new());
    let output = use_signal(|| String::new());
    let error = use_signal(|| String::new());

    // 工具或模式改变时，重算结果
    let repaint = {
        let input = input.clone();
        let mut output = output.clone();
        let mut error = error.clone();
        let current_tool_id = current_tool_id.clone();
        let current_mode_id = current_mode_id.clone();

        move || {
            let src = input.read().clone();
            if src.trim().is_empty() {
                output.set(String::new());
                error.set(String::new());
                return;
            }
            let reg = registry();
            if let Some(tool) = reg.by_id(&current_tool_id.read()) {
                match tool.process(&current_mode_id.read(), &src) {
                    Ok(s) => {
                        output.set(s);
                        error.set(String::new());
                    }
                    Err(e) => {
                        output.set(String::new());
                        error.set(e.message);
                    }
                }
            }
        }
    };

    let on_input = {
        let mut input = input.clone();
        let mut repaint = repaint.clone();
        move |e: Event<FormData>| {
            input.set(e.value().to_string());
            repaint();
        }
    };

    let mut sidebar_tool_id = current_tool_id.clone();
    let mut sidebar_mode_id = current_mode_id.clone();
    let mut sidebar_repaint = repaint.clone();
    let mut main_mode_id = current_mode_id.clone();
    let mut main_repaint = repaint.clone();

    rsx! {
        div {
            style: "display:flex; height:100vh; background:#1e1e1e; color:#ccc; font-family: system-ui, -apple-system, Segoe UI, Roboto, sans-serif;",

            // 侧边栏
            Sidebar {
                selected_tool_id: current_tool_id(),
                on_select: move |tid: String| {
                    let reg = registry();
                    if let Some(tool) = reg.by_id(&tid) {
                        sidebar_tool_id.set(tid);
                        sidebar_mode_id.set(tool.default_mode().to_string());
                        sidebar_repaint();
                    }
                }
            }

            // 主区域
            MainArea {
                tool_id: current_tool_id(),
                mode_id: current_mode_id(),
                on_select_mode: move |mid: String| {
                    main_mode_id.set(mid);
                    main_repaint();
                },
                input, output, error, on_input
            }
        }
    }
}

#[component]
fn Sidebar(selected_tool_id: String, on_select: EventHandler<String>) -> Element {
    let reg = registry();
    rsx! {
        div {
            style: "width:220px; padding:14px; border-right:1px solid #2a2a2a; background:#181818; display:flex; flex-direction:column; gap:8px; overflow:auto;",
            div { style: "font-size:14px; font-weight:700; color:#eee; margin-bottom:6px;", "ToolBox" }
            div { style: "font-size:12px; opacity:.7; margin-bottom:12px;", "选择一个工具开始" }

            for t in reg.tools() {
                div {
                    style: format!(
                        "cursor:pointer; padding:10px 12px; border-radius:8px; {}",
                        if t.id() == selected_tool_id {
                            "background:#2d2d30; color:#fff; border:1px solid #3c3c3c;"
                        } else {
                            "color:#ccc; border:1px solid transparent;"
                        }
                    ),
                    onclick: {
                        let id = t.id().to_string();
                        let on_select = on_select.clone();
                        move |_| on_select.call(id.clone())
                    },
                    "{t.name()}"
                }
            }
            div { style: "flex:1" }
            div { style: "font-size:11px; opacity:.5;", "Tips: 左右面板可独立滚动" }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct MainAreaProps {
    tool_id: String,
    mode_id: String,
    on_select_mode: EventHandler<String>,
    input: Signal<String>,
    output: Signal<String>,
    error: Signal<String>,
    on_input: EventHandler<Event<FormData>>,
}

#[component]
fn MainArea(props: MainAreaProps) -> Element {
    let reg = registry();
    let tool = reg.by_id(&props.tool_id).expect("工具应存在");
    let modes = tool.modes();
    let error_text = props.error.read().clone();

    rsx! {
        div { style: "flex:1; display:flex; flex-direction:column;",

            // 顶部模式条
            div {
                style: "display:flex; align-items:center; gap:10px; padding:10px 14px; background:#2d2d30; border-bottom:1px solid #2a2a2a;",
                h3 { style: "margin:0; font-size:14px;", "{tool.name()}" }
                div { style: "flex:1" }
                div {
                    style: "display:flex; gap:6px;",
                    for m in modes {
                        button {
                            style: format!(
                                "padding:6px 10px; border-radius:6px; font-size:12px; cursor:pointer; {}",
                                if props.mode_id == m.id {
                                    "background:#3a74d7; color:#fff; border:1px solid #3a74d7;"
                                } else {
                                    "background:#3c3c3c; color:#ccc; border:1px solid #555;"
                                }
                            ),
                            onclick: {
                                let mid = m.id.to_string();
                                let on_select = props.on_select_mode.clone();
                                move |_| on_select.call(mid.clone())
                            },
                            "{m.label}"
                        }
                    }
                }
            }

            // 双面板
            TwoPane {
                input: props.input,
                output: props.output,
                on_input: props.on_input
            }

            if !error_text.is_empty() {
                div {
                    style: "margin: 10px 14px; padding:10px 12px; color:#f48771; background:rgba(244,135,113,.1); border:1px solid #f48771; border-radius:8px; font-size:13px;",
                    "{error_text}"
                }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct TwoPaneProps {
    input: Signal<String>,
    output: Signal<String>,
    on_input: EventHandler<Event<FormData>>,
}

#[component]
fn TwoPane(props: TwoPaneProps) -> Element {
    rsx! {
        div {
            style: "display:flex; gap:12px; padding:12px; flex:1; overflow:hidden;",

            // 左
            div {
                style: "flex:1; display:flex; flex-direction:column; border:1px solid #3c3c3c; border-radius:8px; overflow:hidden;",
                div { style: "padding:8px 10px; background:#2d2d30; font-size:13px; border-bottom:1px solid #3c3c3c;", "输入" }
                textarea {
                    style: "flex:1; background:#1e1e1e; color:#ccc; border:none; padding:12px; font-family: Consolas, Monaco, monospace; font-size:13px; line-height:1.5; resize:none; overflow:auto;",
                    value: "{props.input}",
                    oninput: move |e| props.on_input.call(e),
                    placeholder: "在此输入…"
                }
            }

            // 右
            div {
                style: "flex:1; display:flex; flex-direction:column; border:1px solid #3c3c3c; border-radius:8px; overflow:hidden;",
                div { style: "padding:8px 10px; background:#2d2d30; font-size:13px; border-bottom:1px solid #3c3c3c;", "输出" }
                textarea {
                    style: "flex:1; background:#111; color:#ddd; border:none; padding:12px; font-family: Consolas, Monaco, monospace; font-size:13px; line-height:1.5; resize:none; overflow:auto;",
                    value: "{props.output}",
                    readonly: true
                }
            }
        }
    }
}
