use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

// 1) 引入三个子模块
mod base64;
mod common;
mod json;
mod timestamp;
// 2) 把组件 re-export 出来（在本模块内/外都能直接用 ui::json 等）

use crate::component::base64::Base64Tool;
use crate::component::json::JsonFormatterTool;
pub use timestamp::TimestampTool;

// 3) 状态与工具枚举
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

// 4) App（保持和你原来一致）
#[component]
pub fn App() -> Element {
    let app_state = use_context_provider(|| Signal::new(AppState::default()));

    rsx! {
        div {
            class: "app-container",
            style: "display: flex; height: 100vh; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background-color: #1e1e1e; color: #cccccc;",
            Sidebar { app_state }
            div {
                class: "main-content",
                style: "flex: 1; display: flex; flex-direction: column;",
                div {
                    class: "toolbar",
                    style: "height: 40px; background: #252526; border-bottom: 1px solid #3c3c3c; display: flex; align-items: center; padding: 0 10px;",
                    h2 { style: "color: #cccccc; font-size: 14px; font-weight: 400; margin: 0;", "{app_state().current_tool.name()}" }
                }
                div {
                    class: "tool-content",
                    style: "flex: 1; overflow: hidden;",
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

// 5) Sidebar（保持和你原来一致）
#[component]
fn Sidebar(app_state: Signal<AppState>) -> Element {
    let tools = vec![
        Tool::JsonFormatter,
        Tool::Base64Encoder,
        Tool::TimestampConverter,
    ];

    rsx! {
        div {
            class: "sidebar",
            style: "width: 50px; background: #333333; display: flex; flex-direction: column; align-items: center; padding-top: 10px;",
            div {
                class: "tool-icons",
                style: "display: flex; flex-direction: column; gap: 15px;",
                {tools.into_iter().map(|tool| {
                    let is_active = app_state().current_tool == tool;
                    let background_style = if is_active { "background: #007acc; border-radius: 4px;" } else { "background: transparent;" };
                    let style = format!(
                        "border: none; width: 36px; height: 36px; border-radius: 4px; {} \
                         cursor: pointer; display: flex; align-items: center; justify-content: center; \
                         color: white; font-size: 18px;",
                        background_style
                    );
                    rsx! {
                        button {
                            key: "{tool.name()}",
                            class: "tool-button",
                            style: "{style}",
                            onclick: move |_| { app_state.write().current_tool = tool; },
                            title: "{tool.name()}",
                            "{tool.icon()}"
                        }
                    }
                })}
            }
        }
    }
}
