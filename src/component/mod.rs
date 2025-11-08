use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

mod base64;
mod json;
mod timestamp;
mod util;

use crate::component::base64::Base64Tool;
use crate::component::json::JsonFormatterTool;
pub use timestamp::TimestampTool;

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
}
impl Default for AppState {
    fn default() -> Self {
        Self {
            current_tool: Tool::JsonFormatter,
        }
    }
}

// 4) App
#[component]
pub fn App() -> Element {
    let app_state = use_context_provider(|| Signal::new(AppState::default()));

    rsx! {
        div {
            class: "app-container",
            style: "display:flex; height:100vh; font-family:-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background-color:#1e1e1e; color:#cccccc;",
            Sidebar { app_state }

            div {
                class: "main-content",
                style: "flex:1; display:flex; flex-direction:column;",

                // ===== 顶部 Toolbar：标题绝对居中 =====
                div {
                    class: "toolbar",
                    style: "
                        height:40px;
                        background:#252526;
                        border-bottom:1px solid #3c3c3c;
                        display:flex;
                        align-items:center;
                        padding:0 10px;
                        position:relative;    
                        user-select:none; -webkit-user-select:none; -moz-user-select:none; -ms-user-select:none;
                    ",

                    // 左侧：图标组占位（可放返回、刷新等）
                    div {
                        style: "display:flex; align-items:center; gap:8px;",
                        // 示例图标占位（可按需替换/删除）
                        // span { "⬅︎" }
                        // span { "↻" }
                    }

                    // 中间：始终绝对居中的标题（不受左右影响）
                    h1 {
                        style: "
                            position:absolute;
                            left:50%;
                            transform:translateX(-50%);
                            margin:0;
                            color:#cccccc;
                            font-size:14px;
                            font-weight:500;
                            line-height:40px;
                            pointer-events:none;   /* 不遮挡左右按钮点击 */
                        ",
                        "{app_state().current_tool.name()}"
                    }

                    // 右侧：设置按钮占位
                    div {
                        style: "margin-left:auto; display:flex; align-items:center; gap:8px;",
                        // 示例按钮占位：
                        // button {
                        //     style: "background:#3a3a3a; color:#ddd; border:1px solid #4a4a4a; border-radius:6px; padding:4px 8px; cursor:pointer; font-size:12px;",
                        //     "设置"
                        // }
                    }
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

// ================= Sidebar =================
#[component]
fn Sidebar(app_state: Signal<AppState>) -> Element {
    // --- 状态：宽度、是否收起、是否拖拽中、上一次鼠标x、收起前宽度 ---
    let mut width = use_signal(|| 220.0_f32); // 当前宽度（展开时）
    let collapsed = use_signal(|| false); // 是否收起（仅图标）
    let mut dragging = use_signal(|| false); // 是否正在拖拽
    let mut last_x = use_signal(|| 0.0_f32); // 上一次鼠标 x
    let saved_width = use_signal(|| 220.0_f32); // 收起前记忆的宽度

    // 限制/常量
    let min_w: f32 = 160.0;
    let max_w: f32 = 420.0;
    let collapsed_w: f32 = 56.0;

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
                // 展开，恢复记忆宽度
                let w = (*saved_width.read()).clamp(min_w, max_w);
                width.set(w);
                collapsed.set(false);
            } else {
                // 收起，记忆当前宽度
                saved_width.set(*width.read());
                collapsed.set(true);
            }
        }
    };

    // 按下把手：开始拖拽
    let on_handle_mouse_down = {
        let mut dragging = dragging;
        let mut last_x = last_x;
        move |e: MouseEvent| {
            dragging.set(true);
            last_x.set(e.client_coordinates().x as f32);
        }
    };

    // 侧栏区域监听移动：拖拽时更新宽度（仅在未收起时生效）
    let on_mouse_move = {
        move |e: MouseEvent| {
            if *dragging.read() && !*collapsed.read() {
                let cx = e.client_coordinates().x as f32;
                let delta = cx - *last_x.read();
                last_x.set(cx);
                let new_w = (*width.read() + delta).clamp(min_w, max_w);
                width.set(new_w);
            }
        }
    };

    // 松开：结束拖拽
    let on_mouse_up = {
        move |_e: MouseEvent| {
            if *dragging.read() {
                dragging.set(false);
            }
        }
    };

    // 条目样式
    let item_style = |active: bool, collapsed: bool| -> String {
        let base = if active {
            "cursor:pointer; padding:8px 10px; border-radius:8px; background:#2d2d30; color:#fff; border:1px solid #3c3c3c; font-weight:600;"
        } else {
            "cursor:pointer; padding:8px 10px; border-radius:8px; color:#ccc; border:1px solid transparent;"
        };
        if collapsed {
            // 居中，仅显示图标
            format!("{base} display:flex; align-items:center; justify-content:center; height:36px;")
        } else {
            // 图标 + 文本
            format!("{base} display:flex; align-items:center; gap:10px; height:36px;")
        }
    };

    rsx! {
        // 外层：包含侧栏与把手，侧栏上监听 move/up 便于拖拽
        div {
            style: "display:flex; height:100%; user-select:none; -webkit-user-select:none; -moz-user-select:none; -ms-user-select:none;",

            // 侧栏容器
            div {
                style: "width:{sidebar_w}px; background:#333333; display:flex; flex-direction:column; padding:10px 8px; box-sizing:border-box; user-select:none;",
                ondoubleclick: toggle_collapse,
                onmousemove: on_mouse_move,
                onmouseup: on_mouse_up,

                // 顶部标题（展开显示：图标+文字；收起显示：只有图标）
                div {
                    style: "
                        color:#eee;
                        font-size:15px;
                        font-weight:700;
                        margin-bottom:8px;
                        text-align:center;
                        user-select:none;
                        display:flex;
                        align-items:center;
                        justify-content:center;
                        gap:6px;
                        height:32px;
                    ",

                    // 图标：展开 + 收起都显示
                    span { style: "font-size:20px;", "🔧" }

                    // 文本：只有展开时显示
                    if !*collapsed.read() {
                        span { style: "font-size:14px; color:#ddd;", "ToolBox" }
                    }
                }

                // 工具按钮区
                div {
                    style: "display:flex; flex-direction:column; gap:8px;",
                    {tools.into_iter().map(|tool| {
                        let is_active = app_state().current_tool == tool;
                        let style = item_style(is_active, *collapsed.read());
                        rsx!{
                            div {
                                key: "{tool.name()}",
                                style: "{style}",
                                onclick: move |_| { app_state.write().current_tool = tool; },
                                title: "{tool.name()}",
                                // 图标
                                span { style: "font-size:18px;", "{tool.icon()}" }
                                // 展开时显示文字
                                if !*collapsed.read() {
                                    span { style: "font-size:13px; color:#ddd;", "{tool.name()}" }
                                }
                            }
                        }
                    })}
                }

                // 占位撑满
                div { style: "flex:1;" }
            }

            // 右侧拖拽把手（独立 6px）
            div {
                style: "width:6px; background:linear-gradient(90deg,#2b2b2b,#2f2f2f); cursor:col-resize;",
                onmousedown: on_handle_mouse_down,
                ondoubleclick: toggle_collapse,
            }
        }
    }
}
