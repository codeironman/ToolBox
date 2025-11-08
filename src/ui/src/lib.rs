use dioxus::events::MouseData;
use dioxus::prelude::*;
use std::time::Instant;
use toolbox_core::registry;

const SIDEBAR_DEFAULT_WIDTH: f32 = 240.0;
const SIDEBAR_MIN_WIDTH: f32 = 180.0;
const SIDEBAR_MAX_WIDTH: f32 = 360.0;
const SIDEBAR_ICON_WIDTH: f32 = 68.0;
const SIDEBAR_AUTO_COLLAPSE_THRESHOLD: f32 = 150.0;
const SIDEBAR_DOUBLE_CLICK_THRESHOLD_MS: u128 = 350;

#[component]
pub fn App() -> Element {
    // 当前工具 id 和 模式 id 保存在状态
    let tools = registry(); // &[Arc<dyn Tool>]
    let first_tool = tools.tools().first().expect("至少一个工具已注册").clone();

    let current_tool_id = use_signal(|| first_tool.id().to_string());
    let current_mode_id = use_signal(|| first_tool.default_mode().to_string());

    // 双面板状态
    let input = use_signal(String::new);
    let output = use_signal(String::new);
    let error = use_signal(String::new);

    // 侧边栏状态
    let sidebar_width = use_signal(|| SIDEBAR_DEFAULT_WIDTH);
    let sidebar_last_width = use_signal(|| SIDEBAR_DEFAULT_WIDTH);
    let sidebar_collapsed = use_signal(|| false);
    let sidebar_dragging = use_signal(|| false);

    // 工具或模式改变时，重算结果
    let repaint = {
        let mut output = output;
        let mut error = error;

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
        let mut input = input;
        let mut repaint = repaint;
        move |e: Event<FormData>| {
            input.set(e.value().to_string());
            repaint();
        }
    };

    let mut sidebar_tool_id = current_tool_id;
    let mut sidebar_mode_id = current_mode_id;
    let mut sidebar_repaint = repaint;
    let mut main_mode_id = current_mode_id;
    let mut main_repaint = repaint;

    let start_drag = {
        let mut sidebar_dragging = sidebar_dragging;
        move |_evt: Event<MouseData>| {
            sidebar_dragging.set(true);
        }
    };

    let drag_move = {
        let mut sidebar_width = sidebar_width;
        let mut sidebar_last_width = sidebar_last_width;
        let mut sidebar_collapsed = sidebar_collapsed;
        move |evt: Event<MouseData>| {
            let x = evt.data.page_coordinates().x as f32;
            let clamped = x.clamp(SIDEBAR_ICON_WIDTH, SIDEBAR_MAX_WIDTH);
            if clamped < SIDEBAR_AUTO_COLLAPSE_THRESHOLD {
                sidebar_collapsed.set(true);
            } else {
                sidebar_collapsed.set(false);
                sidebar_width.set(clamped);
                sidebar_last_width.set(clamped);
            }
        }
    };

    let drag_end = {
        let mut sidebar_width = sidebar_width;
        let mut sidebar_last_width = sidebar_last_width;
        let mut sidebar_collapsed = sidebar_collapsed;
        let mut sidebar_dragging = sidebar_dragging;
        move |evt: Event<MouseData>| {
            let x = evt.data.page_coordinates().x as f32;
            let clamped = x.clamp(SIDEBAR_ICON_WIDTH, SIDEBAR_MAX_WIDTH);
            if clamped < SIDEBAR_AUTO_COLLAPSE_THRESHOLD {
                sidebar_collapsed.set(true);
            } else {
                sidebar_collapsed.set(false);
                sidebar_width.set(clamped);
                sidebar_last_width.set(clamped);
            }
            sidebar_dragging.set(false);
        }
    };

    let current_sidebar_width = if sidebar_collapsed() {
        SIDEBAR_ICON_WIDTH
    } else {
        sidebar_width()
    };

    rsx! {
        div {
            style: "display:flex; height:100vh; background:#1e1e1e; color:#ccc; font-family: system-ui, -apple-system, Segoe UI, Roboto, sans-serif; position:relative;",

            // 侧边栏
            Sidebar {
                selected_tool_id: current_tool_id(),
                collapsed: sidebar_collapsed(),
                width: current_sidebar_width,
                on_toggle_collapse: {
                    let mut collapsed = sidebar_collapsed;
                    let mut width = sidebar_width;
                    let last_width = sidebar_last_width;
                    move |_| {
                        let next = !collapsed();
                        collapsed.set(next);
                        if !next {
                            let restore = last_width().max(SIDEBAR_MIN_WIDTH);
                            width.set(restore);
                        }
                    }
                },
                on_select: move |tid: String| {
                    let reg = registry();
                    if let Some(tool) = reg.by_id(&tid) {
                        sidebar_tool_id.set(tid);
                        sidebar_mode_id.set(tool.default_mode().to_string());
                        sidebar_repaint();
                    }
                }
            }

            // 拖拽手柄
            div {
                style: "width:4px; cursor:col-resize; background:#1f1f1f; border-right:1px solid #111;",
                onmousedown: start_drag,
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

            if sidebar_dragging() {
                div {
                    style: "position:fixed; inset:0; cursor:col-resize; z-index:999;",
                    onmousemove: drag_move,
                    onmouseup: drag_end,
                    onmouseleave: {
                        let mut drag_flag = sidebar_dragging;
                        move |_| drag_flag.set(false)
                    },
                }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct SidebarProps {
    selected_tool_id: String,
    collapsed: bool,
    width: f32,
    on_select: EventHandler<String>,
    on_toggle_collapse: EventHandler<()>,
}

#[component]
fn Sidebar(props: SidebarProps) -> Element {
    let double_click_tracker = use_signal(|| None::<(String, Instant)>);
    let reg = registry();
    let show_labels = !props.collapsed;
    let horizontal_padding = if props.collapsed { "10px" } else { "14px" };
    let container_style = format!(
        "width:{0}px; min-width:{0}px; max-width:{0}px; padding:{1} 10px; border-right:1px solid #2a2a2a; background:#181818; display:flex; flex-direction:column; gap:8px; overflow:auto;",
        props.width,
        horizontal_padding
    );
    rsx! {
        div {
            style: "{container_style}",
            if show_labels {
                div { style: "font-size:14px; font-weight:700; color:#eee; margin-bottom:6px;", "ToolBox" }
                div { style: "font-size:12px; opacity:.7; margin-bottom:12px;", "选择一个工具开始" }
            }

            for t in reg.tools() {
                div {
                    style: format!(
                        "cursor:pointer; padding:10px; border-radius:8px; border:1px solid transparent; display:flex; align-items:center; {} {}",
                        if t.id() == props.selected_tool_id {
                            "background:#2d2d30; color:#fff; border:1px solid #3c3c3c;"
                        } else {
                            "color:#ccc; border:1px solid transparent;"
                        },
                        if show_labels {
                            "gap:8px; justify-content:flex-start;"
                        } else {
                            "gap:0; justify-content:center;"
                        }
                    ),
                    title: "{t.name()}",
                    onclick: {
                        let id = t.id().to_string();
                        let on_select = props.on_select;
                        let on_toggle = props.on_toggle_collapse;
                        let mut tracker = double_click_tracker;
                        move |_| {
                            let now = Instant::now();
                            let mut trigger_toggle = false;
                            if let Some((last_id, last_time)) = tracker() {
                                if last_id == id && now.duration_since(last_time).as_millis() <= SIDEBAR_DOUBLE_CLICK_THRESHOLD_MS {
                                    trigger_toggle = true;
                                    tracker.set(None);
                                }
                            }
                            if !trigger_toggle {
                                tracker.set(Some((id.clone(), now)));
                            }
                            on_select.call(id.clone());
                            if trigger_toggle {
                                on_toggle.call(());
                            }
                        }
                    },
                    span { style: "font-size:14px; font-weight:600;", "{tool_icon(t.id())}" }
                    span { style: if show_labels { "display:block;" } else { "display:none;" }, "{t.name()}" }
                }
            }
            div { style: "flex:1" }
            if show_labels {
                div { style: "font-size:11px; opacity:.5;", "Tips: 左右面板可独立滚动 · 双击图标可收起" }
            }
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
                                let on_select = props.on_select_mode;
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

fn tool_icon(tool_id: &str) -> &'static str {
    match tool_id {
        "json" => "{}",
        "b64" => "B64",
        "ts" => "TS",
        _ => "...",
    }
}
