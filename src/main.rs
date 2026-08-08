mod component;

use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
use dioxus::LaunchBuilder;

#[cfg(target_os = "macos")]
use dioxus::desktop::tao::platform::macos::WindowBuilderExtMacOS;

use crate::component::App;

fn main() {
    let window = WindowBuilder::new()
        .with_title("ToolBox")
        .with_inner_size(LogicalSize::new(1280.0, 820.0))
        .with_min_inner_size(LogicalSize::new(900.0, 600.0));

    // macOS：去掉系统标题栏的「白框」与「ToolBox」标题文字——
    // 标题栏透明 + 内容全尺寸延伸 + 隐藏标题文字。红黄绿按钮保留，
    // 顶部标题栏区域仍可拖拽窗口；内容区在 mod.rs 里用 padding-top 让出该区域。
    #[cfg(target_os = "macos")]
    let window = window
        .with_titlebar_transparent(true)
        .with_fullsize_content_view(true)
        .with_title_hidden(true);

    LaunchBuilder::new()
        .with_cfg(Config::default().with_window(window))
        .launch(App);
}
