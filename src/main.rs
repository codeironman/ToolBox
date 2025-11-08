use base64::{engine::general_purpose, Engine};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use dioxus::prelude::*;
use serde_json;

// ===== 业务模型 =====
#[derive(Clone, Copy, PartialEq, Eq)]
enum ToolType {
    Json,
    Base64,
    Timestamp,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ToolMode {
    // JSON
    JsonPretty,
    JsonMinify,
    // Base64
    B64Encode,
    B64Decode,
    // Timestamp
    TsToHuman, // unix -> human
    TsToUnix,  // human -> unix
}

// 每个工具对应的默认模式
fn default_mode(tool: ToolType) -> ToolMode {
    match tool {
        ToolType::Json => ToolMode::JsonPretty,
        ToolType::Base64 => ToolMode::B64Encode,
        ToolType::Timestamp => ToolMode::TsToHuman,
    }
}

// 每个工具的模式清单（用于右侧顶部“模式选择”）
fn modes_for(tool: ToolType) -> &'static [ToolMode] {
    match tool {
        ToolType::Json => &[ToolMode::JsonPretty, ToolMode::JsonMinify],
        ToolType::Base64 => &[ToolMode::B64Encode, ToolMode::B64Decode],
        ToolType::Timestamp => &[ToolMode::TsToHuman, ToolMode::TsToUnix],
    }
}

fn mode_label(m: ToolMode) -> &'static str {
    match m {
        ToolMode::JsonPretty => "Pretty（美化）",
        ToolMode::JsonMinify => "Minify（压缩）",
        ToolMode::B64Encode => "Encode（编码）",
        ToolMode::B64Decode => "Decode（解码）",
        ToolMode::TsToHuman => "Unix → 人类可读",
        ToolMode::TsToUnix => "人类可读 → Unix",
    }
}

fn tool_label(t: ToolType) -> &'static str {
    match t {
        ToolType::Json => "JSON 工具",
        ToolType::Base64 => "Base64 工具",
        ToolType::Timestamp => "时间戳 工具",
    }
}

// ===== 顶层 App：左侧侧边栏 + 右侧工作区 =====
#[component]
fn App() -> Element {
    // 当前选中的工具和模式
    let mut tool = use_signal(|| ToolType::Json);
    let mode = use_signal(|| default_mode(*tool.read()));

    // 双面板文本状态
    let input = use_signal(|| String::new());
    let output = use_signal(|| String::new());
    let error = use_signal(|| String::new());

    // 当切换工具时，重置模式为默认，并立刻重算输出
    use_effect({
        let tool = tool.clone();
        let mut mode = mode.clone();
        let input = input.clone();
        let mut output = output.clone();
        let mut error = error.clone();

        move || {
            let m = default_mode(*tool.read());
            mode.set(m);

            let src = input.read().clone();
            if src.trim().is_empty() {
                output.set(String::new());
                error.set(String::new());
            } else {
                match process_input(*tool.read(), *mode.read(), &src) {
                    Ok(res) => {
                        output.set(res);
                        error.set(String::new());
                    }
                    Err(e) => {
                        output.set(String::new());
                        error.set(e);
                    }
                }
            }
        }
    });

    // 输入变更即刻处理
    let repaint = {
        let tool = tool.clone();
        let mode = mode.clone();
        let input = input.clone();
        let mut output = output.clone();
        let mut error = error.clone();
        move || {
            let src = input.read().clone();
            if src.trim().is_empty() {
                output.set(String::new());
                error.set(String::new());
                return;
            }
            match process_input(*tool.read(), *mode.read(), &src) {
                Ok(res) => {
                    output.set(res);
                    error.set(String::new());
                }
                Err(e) => {
                    output.set(String::new());
                    error.set(e);
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

    let on_select_mode = {
        let mut mode = mode.clone();
        let mut repaint = repaint.clone();
        move |new_mode: ToolMode| {
            mode.set(new_mode);
            repaint();
        }
    };

    // ===== 布局（左侧：侧边栏；右侧：工具工作区）=====
    rsx! {
        div {
            style: "display:flex; height:100vh; background:#1e1e1e; color:#ccc; font-family: system-ui, -apple-system, Segoe UI, Roboto, sans-serif;",

            // 左侧侧边栏
            Sidebar {
                selected: *tool.read(),
                on_select: move |t| {
                    tool.set(t);
                }
            }

            // 右侧主工作区（包含模式切换条 + 双面板）
            MainWorkspace {
                tool: *tool.read(),
                mode: *mode.read(),
                on_select_mode,
                input,
                output,
                error,
                on_input
            }
        }
    }
}

// ===== 侧边栏组件 =====
#[component]
fn Sidebar(selected: ToolType, on_select: EventHandler<ToolType>) -> Element {
    let item_style = |active: bool| -> String {
        if active {
            "cursor:pointer; padding:10px 12px; border-radius:8px; background:#2d2d30; color:#fff; border:1px solid #3c3c3c; font-weight:600;".into()
        } else {
            "cursor:pointer; padding:10px 12px; border-radius:8px; color:#ccc; border:1px solid transparent;".into()
        }
    };

    rsx! {
        div {
            style: "width:220px; padding:14px; border-right:1px solid #2a2a2a; background:#181818; display:flex; flex-direction:column; gap:8px; overflow:auto;",

            div { style: "font-size:14px; font-weight:700; color:#eee; margin-bottom:6px;", "ToolBox" }
            div { style: "font-size:12px; opacity:.7; margin-bottom:12px;", "选择一个工具开始" }

            div {
                style: "{item_style(matches!(selected, ToolType::Json))}",
                onclick: move |_| on_select.call(ToolType::Json),
                "🧩  ", strong { "{tool_label(ToolType::Json)}" }
            }
            div {
                style: "{item_style(matches!(selected, ToolType::Base64))}",
                onclick: move |_| on_select.call(ToolType::Base64),
                "🔐  ", strong { "{tool_label(ToolType::Base64)}" }
            }
            div {
                style: "{item_style(matches!(selected, ToolType::Timestamp))}",
                onclick: move |_| on_select.call(ToolType::Timestamp),
                "⏱️  ", strong { "{tool_label(ToolType::Timestamp)}" }
            }

            div { style: "flex:1" }
            div { style: "font-size:11px; opacity:.5;", "Tips: 左右面板可独立滚动" }
        }
    }
}

// ===== 主工作区（右侧）=====
#[derive(Props, PartialEq, Clone)]
struct MainWorkspaceProps {
    tool: ToolType,
    mode: ToolMode,
    on_select_mode: EventHandler<ToolMode>,
    input: Signal<String>,
    output: Signal<String>,
    error: Signal<String>,
    on_input: EventHandler<Event<FormData>>,
}

#[component]
fn MainWorkspace(props: MainWorkspaceProps) -> Element {
    // 顶部模式切换条
    let error_text = props.error.read().clone();
    let modes = modes_for(props.tool);

    rsx! {
        div {
            style: "flex:1; display:flex; flex-direction:column;",

            // 顶部条：当前工具名 + 模式选项
            div {
                style: "display:flex; align-items:center; gap:10px; padding:10px 14px; background:#2d2d30; border-bottom:1px solid #2a2a2a;",
                h3 { style: "margin:0; font-size:14px;", "{tool_label(props.tool)}" }

                div { style: "flex:1" }
                // 模式按钮组
                div {
                    style: "display:flex; gap:6px;",
                    for &mode_value in modes {
                        button {
                            style: format!(
                                "padding:6px 10px; border-radius:6px; font-size:12px; cursor:pointer; {}",
                                if mode_value == props.mode {
                                    "background:#3a74d7; color:#fff; border:1px solid #3a74d7;"
                                } else {
                                    "background:#3c3c3c; color:#ccc; border:1px solid #555;"
                                }
                            ),
                            onclick: {
                                let on_select_mode = props.on_select_mode.clone();
                                move |_| on_select_mode.call(mode_value)
                            },
                            "{mode_label(mode_value)}"
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

            // 错误条
            if !error_text.is_empty() {
                div {
                    style: "margin: 10px 14px; padding:10px 12px; color:#f48771; background:rgba(244,135,113,.1); border:1px solid #f48771; border-radius:8px; font-size:13px;",
                    "{error_text}"
                }
            }
        }
    }
}

// ===== 统一双面板组件（左右独立滚动）=====
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

            // 左：输入（独立滚动）
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

            // 右：输出（独立滚动）
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

// ===== 通用处理逻辑（新增工具只改这里）=====
fn process_input(tool: ToolType, mode: ToolMode, input: &str) -> Result<String, String> {
    match tool {
        ToolType::Json => match mode {
            ToolMode::JsonPretty => json_pretty(input),
            ToolMode::JsonMinify => json_minify(input),
            _ => Err("当前工具不支持该模式".into()),
        },
        ToolType::Base64 => match mode {
            ToolMode::B64Encode => Ok(general_purpose::STANDARD.encode(input.as_bytes())),
            ToolMode::B64Decode => {
                let bytes = general_purpose::STANDARD
                    .decode(input.trim().as_bytes())
                    .map_err(|e| format!("Base64 解码失败: {e}"))?;
                String::from_utf8(bytes).map_err(|e| format!("UTF-8 解析失败: {e}"))
            }
            _ => Err("当前工具不支持该模式".into()),
        },
        ToolType::Timestamp => match mode {
            ToolMode::TsToHuman => ts_to_human(input),
            ToolMode::TsToUnix => human_to_unix(input),
            _ => Err("当前工具不支持该模式".into()),
        },
    }
}

// ===== JSON 工具 =====
fn json_pretty(input: &str) -> Result<String, String> {
    let v: serde_json::Value =
        serde_json::from_str(input).map_err(|e| format!("JSON 解析错误: {e}"))?;
    serde_json::to_string_pretty(&v).map_err(|e| format!("JSON 格式化错误: {e}"))
}

fn json_minify(input: &str) -> Result<String, String> {
    let v: serde_json::Value =
        serde_json::from_str(input).map_err(|e| format!("JSON 解析错误: {e}"))?;
    serde_json::to_string(&v).map_err(|e| format!("JSON 压缩错误: {e}"))
}

// ===== 时间戳工具 =====
fn ts_to_human(input: &str) -> Result<String, String> {
    let raw = input.trim().replace('_', "").replace(',', "");
    let n: i128 = raw
        .parse()
        .map_err(|_| "请输入整数型 Unix 时间戳（秒或毫秒）".to_string())?;

    // 秒 or 毫秒 粗略判断
    let (secs, nsecs) = if n.abs() > 10_000_000_000_000i128 {
        (n / 1000, (n % 1000) as i64 * 1_000_000)
    } else {
        (n, 0)
    };

    let dt_utc = Utc
        .timestamp_opt(secs as i64, nsecs as u32)
        .single()
        .ok_or_else(|| "无法构造时间，请检查数值范围".to_string())?;
    let dt_local: DateTime<Local> = DateTime::from(dt_utc);

    Ok(format!(
        "UTC : {}\nLocal: {}",
        dt_utc.to_rfc3339(),
        dt_local.format("%Y-%m-%d %H:%M:%S%.3f %z")
    ))
}

fn human_to_unix(input: &str) -> Result<String, String> {
    let s = input.trim();

    // RFC3339
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.timestamp().to_string());
    }

    // 常见本地格式
    const FMT_SEC: &str = "%Y-%m-%d %H:%M:%S";
    const FMT_MSEC: &str = "%Y-%m-%d %H:%M:%S%.3f";

    if let Ok(naive) = NaiveDateTime::parse_from_str(s, FMT_MSEC) {
        let dt = Local
            .from_local_datetime(&naive)
            .single()
            .ok_or_else(|| "本地时间不唯一/无效（涉夏令时）".to_string())?;
        return Ok(dt.timestamp().to_string());
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(s, FMT_SEC) {
        let dt = Local
            .from_local_datetime(&naive)
            .single()
            .ok_or_else(|| "本地时间不唯一/无效（涉夏令时）".to_string())?;
        return Ok(dt.timestamp().to_string());
    }

    Err("不支持的时间格式：请用 RFC3339（如 2025-09-13T18:00:00+08:00）或 \"YYYY-MM-DD HH:MM:SS[.sss]\"".into())
}

// ===== 入口 =====
fn main() {
    launch(App);
}
