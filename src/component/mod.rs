use dioxus::desktop::use_window;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

mod base64;
mod json;
mod timestamp;
mod util;

use crate::component::base64::Base64Tool;
use crate::component::json::JsonFormatterTool;
pub use timestamp::TimestampTool;

/// 全局样式：统一设计 token + 通用组件 class（含 :hover/:active/:focus 等伪类，
/// 这些用内联 style 无法实现，故集中注入一次）。
const GLOBAL_CSS: &str = r#"
:root{
  --bg-app:#1e1e1e; --bg-panel:#1b1b1b; --bg-header:#252526; --bg-input:#141414;
  --bg-sidebar:#1a1a1a; --bg-card:#232323; --border:#3c3c3c; --border-soft:#2a2a2a;
  --text:#cccccc; --text-dim:#8a8a8a; --text-bright:#ececec;
  --accent:#0e84d8; --accent-hover:#1196e8; --accent-soft:rgba(14,132,216,.16);
  --danger:#f48771; --danger-soft:rgba(127,58,50,.18); --ok:#73c991;
  --bg-btn:#2d2d30; --bg-btn-hover:#3a3a3e; --border-btn:#454545; --border-btn-hover:#5a5a5a;
  --bg-hover:#262626; --bg-input-2:#1b1b1b;
  --text-on-accent:#fff; --placeholder:#5a5a5a;
  --scrollbar:#3a3a3a; --scrollbar-hover:#4a4a4a; --handle-1:#232323; --handle-2:#2b2b2b;
  --shadow:0 2px 10px rgba(0,0,0,.18); --shadow-card:0 4px 14px rgba(0,0,0,.22);
  --radius:8px; --radius-sm:6px;
}
[data-theme="light"]{
  --bg-app:#f4f4f4; --bg-panel:#ffffff; --bg-header:#ececec; --bg-input:#ffffff;
  --bg-sidebar:#f0f0f0; --bg-card:#ffffff; --border:#d4d4d4; --border-soft:#e4e4e4;
  --text:#242424; --text-dim:#6a6a6a; --text-bright:#0a0a0a;
  --accent:#0e84d8; --accent-hover:#0a6db5; --accent-soft:rgba(14,132,216,.12);
  --danger:#d04030; --danger-soft:rgba(208,64,48,.10); --ok:#2a8a4a;
  --bg-btn:#ffffff; --bg-btn-hover:#eaeaea; --border-btn:#d0d0d0; --border-btn-hover:#b0b0b0;
  --bg-hover:#e6e6e6; --bg-input-2:#ffffff;
  --text-on-accent:#fff; --placeholder:#9a9a9a;
  --scrollbar:#c4c4c4; --scrollbar-hover:#a0a0a0; --handle-1:#e0e0e0; --handle-2:#d4d4d4;
  --shadow:0 2px 10px rgba(0,0,0,.08); --shadow-card:0 4px 14px rgba(0,0,0,.08);
}
*{box-sizing:border-box;}
body{margin:0;background:var(--bg-app);}
.tb-btn{background:var(--bg-btn);color:var(--text);border:1px solid var(--border-btn);padding:6px 12px;border-radius:var(--radius-sm);cursor:pointer;font-size:12px;font-family:inherit;transition:background .15s,border-color .15s,color .15s;user-select:none;}
.tb-btn:hover{background:var(--bg-btn-hover);border-color:var(--border-btn-hover);color:var(--text-bright);}
.tb-btn:active{transform:translateY(1px);}
.tb-btn:disabled{background:var(--bg-panel);color:var(--text-dim);border-color:var(--border-soft);cursor:not-allowed;}
.tb-btn-primary{background:var(--accent);color:var(--text-on-accent);border:1px solid var(--accent);padding:6px 14px;border-radius:var(--radius-sm);cursor:pointer;font-size:12px;font-family:inherit;transition:background .15s,transform .05s;user-select:none;}
.tb-btn-primary:hover{background:var(--accent-hover);}
.tb-btn-primary:active{transform:translateY(1px);}
.tb-btn-primary:disabled{background:var(--bg-btn);color:var(--text-dim);border-color:var(--border-btn);cursor:not-allowed;}
.tb-btn-ghost{background:transparent;color:var(--text-dim);border:1px solid transparent;padding:6px 12px;border-radius:var(--radius-sm);cursor:pointer;font-size:12px;font-family:inherit;transition:background .15s,color .15s;user-select:none;}
.tb-btn-ghost:hover{background:var(--bg-btn);color:var(--text-bright);}
.tb-icon-btn{background:var(--bg-btn);color:var(--text);border:1px solid var(--border-btn);width:28px;height:28px;display:inline-flex;align-items:center;justify-content:center;border-radius:var(--radius-sm);cursor:pointer;font-size:13px;font-family:inherit;transition:background .15s,border-color .15s,color .15s;user-select:none;}
.tb-icon-btn:hover{background:var(--bg-btn-hover);border-color:var(--border-btn-hover);color:var(--text-bright);}
.tb-icon-btn:active{transform:translateY(1px);}
.tb-panel{background:var(--bg-panel);border:1px solid var(--border);border-radius:var(--radius);overflow:hidden;box-shadow:var(--shadow);}
.tb-panel-header{height:38px;background:var(--bg-header);border-bottom:1px solid var(--border);padding:0 12px;display:flex;align-items:center;gap:8px;font-size:13px;font-weight:600;color:var(--text-bright);user-select:none;}
.tb-textarea{flex:1;background:var(--bg-input);color:var(--text);border:none;padding:14px;font-family:'Menlo','Monaco','Consolas',monospace;font-size:13px;line-height:1.6;resize:none;overflow:auto;outline:none;}
.tb-textarea::placeholder{color:var(--placeholder);}
.tb-input{background:var(--bg-input);color:var(--text);border:1px solid var(--border);border-radius:var(--radius-sm);padding:8px 10px;font-family:inherit;font-size:13px;outline:none;transition:border-color .15s,box-shadow .15s;}
.tb-input:focus{border-color:var(--accent);box-shadow:0 0 0 2px var(--accent-soft);}
.tb-search-input{flex:1;background:var(--bg-input-2);color:var(--text-bright);border:1px solid var(--border-btn);padding:6px 10px;font-family:'Menlo','Monaco','Consolas',monospace;font-size:13px;border-radius:var(--radius-sm);outline:none;transition:border-color .15s,box-shadow .15s;}
.tb-search-input:focus{border-color:var(--accent);box-shadow:0 0 0 2px var(--accent-soft);}
.tb-count{font-size:11px;color:var(--text-dim);min-width:36px;text-align:center;user-select:none;font-variant-numeric:tabular-nums;}
.tb-sidebar-item{cursor:pointer;padding:9px 10px;border-radius:var(--radius-sm);border:1px solid transparent;display:flex;align-items:center;gap:10px;color:var(--text);transition:background .15s,border-color .15s,color .15s;user-select:none;}
.tb-sidebar-item:hover{background:var(--bg-hover);color:var(--text-bright);}
.tb-mode-btn{padding:6px 12px;border-radius:var(--radius-sm);font-size:12px;cursor:pointer;border:1px solid transparent;transition:background .15s,color .15s,border-color .15s;user-select:none;}
.tb-mode-btn:hover{color:var(--text-bright);}
.tb-scroll::-webkit-scrollbar{width:10px;height:10px;}
.tb-scroll::-webkit-scrollbar-thumb{background:var(--scrollbar);border-radius:5px;border:2px solid var(--bg-app);}
.tb-scroll::-webkit-scrollbar-thumb:hover{background:var(--scrollbar-hover);}
.tb-scroll::-webkit-scrollbar-track{background:transparent;}
"#;

/// macOS 透明标题栏下，内容顶部需让出红黄绿按钮 / 拖拽区的高度；其他平台为 0。
#[cfg(target_os = "macos")]
const TITLEBAR_PAD: &str = "28px";
#[cfg(not(target_os = "macos"))]
const TITLEBAR_PAD: &str = "0px";

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
enum Tool {
    JsonFormatter,
    Base64Encoder,
    TimestampConverter,
}

impl Tool {
    fn name(&self) -> &'static str {
        match self {
            Tool::JsonFormatter => "JSON 格式化",
            Tool::Base64Encoder => "Base64 编解码",
            Tool::TimestampConverter => "时间戳转换",
        }
    }
    fn icon(&self) -> &'static str {
        match self {
            Tool::JsonFormatter => "📄",
            Tool::Base64Encoder => "🔒",
            Tool::TimestampConverter => "⏰",
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct AppState {
    current_tool: Tool,
    dark: bool,
}
impl Default for AppState {
    fn default() -> Self {
        Self {
            current_tool: Tool::JsonFormatter,
            dark: true,
        }
    }
}

// 4) App
#[component]
pub fn App() -> Element {
    let app_state = use_context_provider(|| Signal::new(AppState::default()));
    let desktop = use_window();
    let desktop_max = desktop.clone();

    // 仅注入一次全局 CSS（desktop webview 不读取项目根 index.html，故用 eval 注入 <style>）
    use_hook(|| {
        let css = serde_json::to_string(GLOBAL_CSS).unwrap_or_else(|_| "\"\"" .to_string());
        dioxus::document::eval(&format!(
            "if(!document.querySelector('style[data-tb]')){{var s=document.createElement('style');s.setAttribute('data-tb','1');s.textContent={};document.head.appendChild(s);}}",
            css
        ));
    });

    // 主题切换：监听 dark 字段，设置 <html data-theme="dark|light">
    use_effect(move || {
        let theme = if app_state.read().dark { "dark" } else { "light" };
        dioxus::document::eval(&format!(
            "document.documentElement.setAttribute('data-theme','{theme}');"
        ));
    });

    rsx! {
        div {
            class: "app-container",
            style: "display:flex; flex-direction:column; height:100vh; font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif; background:var(--bg-app); color:var(--text);",

            // macOS 透明标题栏拖拽区：按住此处可拖动整个窗口；双击最大化/还原；
            // 红黄绿按钮浮在此区域左侧，点按钮不受影响。
            div {
                style: "height:{TITLEBAR_PAD}; flex-shrink:0; width:100%;",
                onmousedown: move |_| { desktop.drag(); },
                ondoubleclick: move |_| { desktop_max.toggle_maximized(); },
            }

            div {
                style: "display:flex; flex:1; min-height:0;",
                Sidebar { app_state }

                div {
                    class: "main-content",
                    style: "flex:1; display:flex; flex-direction:column; min-width:0;",

                    // ===== 顶部 Toolbar：标题绝对居中 =====
                    div {
                        class: "toolbar",
                        style: "height:42px; background:var(--bg-header); border-bottom:1px solid var(--border); display:flex; align-items:center; padding:0 12px; position:relative; user-select:none; -webkit-user-select:none;",

                        // 左侧占位（可放返回/刷新等图标）
                        div { style: "display:flex; align-items:center; gap:8px;" }

                        // 中间：始终绝对居中的标题
                        h1 {
                            style: "position:absolute; left:50%; transform:translateX(-50%); margin:0; color:var(--text-bright); font-size:14px; font-weight:700; line-height:42px; pointer-events:none; white-space:nowrap;",
                            "{app_state().current_tool.name()}"
                        }

                        // 右侧占位
                        div { style: "margin-left:auto; display:flex; align-items:center; gap:8px;" }
                    }

                    // ===== 工具内容区 =====
                    div {
                        class: "tool-content",
                        style: "flex:1; overflow:hidden;",
                        match app_state().current_tool {
                            Tool::JsonFormatter => rsx! { JsonFormatterTool {} },
                            Tool::Base64Encoder => rsx! { Base64Tool {} },
                            Tool::TimestampConverter => rsx! { TimestampTool {} },
                        }
                    }
                }
            }
        }
    }
}
#[component]
fn Sidebar(app_state: Signal<AppState>) -> Element {
    // --- 状态：宽度、是否收起、是否拖拽中、上一次鼠标x、收起前宽度 ---
    let mut width = use_signal(|| 180.0_f32); // 当前宽度（展开时）
    let mut collapsed = use_signal(|| false); // 是否收起（仅图标）
    let mut dragging = use_signal(|| false); // 是否正在拖拽
    let mut last_x = use_signal(|| 0.0_f32); // 上一次鼠标 x
    let saved_width = use_signal(|| 180.0_f32); // 收起前记忆的宽度

    // 限制/常量
    let min_w: f32 = 160.0;
    let max_w: f32 = 420.0;
    let collapsed_w: f32 = 60.0;

    // 阈值：小于等于该值就触发收起，避免来回抖动
    let collapse_threshold: f32 = min_w + 2.0;

    // 工具列表
    let tools = vec![
        Tool::JsonFormatter,
        Tool::Base64Encoder,
        Tool::TimestampConverter,
    ];

    // 当前显示宽度
    let sidebar_w = if *collapsed.read() {
        collapsed_w
    } else {
        *width.read()
    };

    // 双击：收起/展开
    let toggle_collapse = {
        let mut collapsed = collapsed;
        let mut saved_width = saved_width;
        let mut width = width;
        move |_| {
            if *collapsed.read() {
                let w = (*saved_width.read()).clamp(min_w, max_w);
                width.set(w);
                collapsed.set(false);
            } else {
                saved_width.set((*width.read()).clamp(min_w, max_w));
                collapsed.set(true);
            }
        }
    };

    // 按下把手：开始拖拽（收起状态下立刻展开再拖）
    let on_handle_mouse_down = {
        move |e: MouseEvent| {
            if *collapsed.read() {
                let w = (*saved_width.read()).clamp(min_w, max_w);
                width.set(w);
                collapsed.set(false);
            }
            dragging.set(true);
            last_x.set(e.client_coordinates().x as f32);
        }
    };

    // 侧栏区域监听移动：拖拽时更新宽度（展开模式下）
    let on_mouse_move = {
        let mut dragging = dragging;
        let mut last_x = last_x;
        let mut width = width;
        let mut collapsed = collapsed;
        let mut saved_width = saved_width;

        move |e: MouseEvent| {
            if *dragging.read() && !*collapsed.read() {
                let cx = e.client_coordinates().x as f32;
                let delta = cx - *last_x.read();
                last_x.set(cx);

                let current_w = *width.read();
                let proposed_w = current_w + delta;

                if proposed_w <= collapse_threshold {
                    saved_width.set(current_w.clamp(min_w, max_w));
                    collapsed.set(true);
                    dragging.set(false);
                    return;
                }

                let new_w = proposed_w.clamp(min_w, max_w);
                width.set(new_w);
            }
        }
    };

    // 松开：结束拖拽
    let on_mouse_up = {
        let mut dragging = dragging;
        move |_e: MouseEvent| {
            if *dragging.read() {
                dragging.set(false);
            }
        }
    };

    // 主题切换按钮：当前是否暗色、对齐方式（收起时居中）、图标与文字
    let is_dark = app_state().dark;
    let theme_justify = if *collapsed.read() { "justify-content:center;" } else { "justify-content:flex-start;" };
    let theme_icon = if is_dark { "🌙" } else { "☀️" };
    let theme_label = if is_dark { "暗色" } else { "亮色" };

    rsx! {
        // 外层：包含侧栏与把手，侧栏上监听 move/up 便于拖拽
        div {
            style: "display:flex; height:100%; user-select:none; -webkit-user-select:none;",

            // 侧栏容器
            div {
                style: "width:{sidebar_w}px; background:var(--bg-sidebar); border-right:1px solid var(--border-soft); display:flex; flex-direction:column; padding:12px 8px; box-sizing:border-box; user-select:none; transition:width .12s ease;",
                ondoubleclick: toggle_collapse,
                onmousemove: on_mouse_move,
                onmouseup: on_mouse_up,

                // 顶部标题（展开显示：图标+文字；收起显示：只有图标）
                div {
                    style: "color:var(--text-bright); font-size:15px; font-weight:700; margin-bottom:12px; text-align:center; user-select:none; display:flex; align-items:center; justify-content:center; gap:6px; height:32px;",
                    span { style: "font-size:26px;", "🔧" }
                    if !*collapsed.read() {
                        span { style: "font-size:14px; color:var(--text-bright);", "ToolBox" }
                    }
                }

                // 工具按钮区
                div {
                    style: "display:flex; flex-direction:column; gap:6px;",
                    {tools.into_iter().map(|tool| {
                        let is_active = app_state().current_tool == tool;
                        let justify = if *collapsed.read() { "justify-content:center;" } else { "justify-content:flex-start;" };
                        let active_style = if is_active {
                            "background:var(--accent-soft); color:var(--text-bright); border:1px solid var(--accent);"
                        } else {
                            ""
                        };
                        rsx!{
                            div {
                                key: "{tool.name()}",
                                class: "tb-sidebar-item",
                                style: "{justify} {active_style}",
                                onclick: move |_| { app_state.write().current_tool = tool; },
                                title: "{tool.name()}",
                                span { style: "font-size:18px;", "{tool.icon()}" }
                                if !*collapsed.read() {
                                    span { style: "font-size:13px;", "{tool.name()}" }
                                }
                            }
                        }
                    })}
                }

                // 占位撑满
                div { style: "flex:1;" }

                // 底部：主题切换（暗色🌙 / 亮色☀️）
                div {
                    class: "tb-sidebar-item",
                    style: "{theme_justify}",
                    onclick: move |_| {
                        let new_dark = !app_state.read().dark;
                        app_state.write().dark = new_dark;
                    },
                    title: "切换主题",
                    span { style: "font-size:18px;", "{theme_icon}" }
                    if !*collapsed.read() {
                        span { style: "font-size:13px;", "{theme_label}" }
                    }
                }
            }

            // 右侧拖拽把手（独立 6px）
            div {
                style: "width:5px; background:linear-gradient(90deg,var(--handle-1),var(--handle-2)); cursor:col-resize; transition:background .15s;",
                onmousedown: on_handle_mouse_down,
                ondoubleclick: toggle_collapse,
            }
        }
    }
}
