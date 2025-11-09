mod component;

use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
use dioxus::LaunchBuilder;

use crate::component::App;

fn main() {
    LaunchBuilder::new()
        .with_cfg(
            Config::default().with_window(
                WindowBuilder::new()
                    .with_title("ToolBox")
                    .with_inner_size(LogicalSize::new(1280.0, 820.0))
                    .with_min_inner_size(LogicalSize::new(900.0, 600.0)), // .with_position(LogicalPosition::new(300.0, 150.0)), // ✅ 近似居中
            ),
        )
        .launch(App);
}
