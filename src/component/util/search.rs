use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct SearchBarProps {
    /// 是否显示查找条
    pub show: Signal<bool>,
    /// 是否展开替换行（右侧可传一个独立的 Signal，或传同一个并禁用替换按钮）
    pub show_replace: Signal<bool>,

    /// 查找关键字
    pub query: Signal<String>,
    /// 替换关键字（右侧若不需要可传入一个临时的 Signal，按钮禁用即可）
    pub replace: Signal<String>,

    /// 上/下一处、关闭
    pub on_prev: EventHandler<()>,
    pub on_next: EventHandler<()>,
    pub on_close: EventHandler<()>,

    /// 查找输入变化时触发（外部重新计算匹配/高亮）
    pub on_query_input: EventHandler<String>,

    /// 点击“替换当前 / 全部替换”
    pub on_replace_one: Option<EventHandler<()>>,
    pub on_replace_all: Option<EventHandler<()>>,

    /// 是否禁用替换（右侧传 true）
    #[props(default = false)]
    pub replace_disabled: bool,
}

#[component]
pub fn SearchBar(props: SearchBarProps) -> Element {
    // 不显示就渲染空节点
    if !*props.show.read() {
        return rsx! { Fragment {} };
    }

    // 动态样式在 rsx! 外构造
    let disabled = props.replace_disabled;

    let replace_input_bg = if disabled { "#454545" } else { "#ffffff" };
    let replace_input_fg = if disabled { "#999999" } else { "#000000" };
    let replace_input_style = format!(
        "flex:1; background:{}; color:{}; border:1px solid #454545; padding:6px 10px; \
         font-family:'Monaco','Consolas',monospace; font-size:13px; border-radius:4px;",
        replace_input_bg, replace_input_fg
    );

    let replace_btn1_bg = if disabled { "#333333" } else { "#3c3c3c" };
    let replace_btn1_fg = if disabled { "#666666" } else { "#cccccc" };
    let replace_btn1_cursor = if disabled { "not-allowed" } else { "pointer" };
    let replace_btn1_style = format!(
        "background:{}; color:{}; border:1px solid #555; padding:6px 10px; \
         border-radius:4px; cursor:{}; font-size:12px;",
        replace_btn1_bg, replace_btn1_fg, replace_btn1_cursor
    );

    let replace_btn2_bg = if disabled { "#005a9e" } else { "#007acc" };
    let replace_btn2_fg = if disabled { "#999999" } else { "#ffffff" };
    let replace_btn2_cursor = if disabled { "not-allowed" } else { "pointer" };
    let replace_btn2_style = format!(
        "background:{}; color:{}; border:none; padding:6px 12px; \
         border-radius:4px; cursor:{}; font-size:12px;",
        replace_btn2_bg, replace_btn2_fg, replace_btn2_cursor
    );

    // 预先克隆必要的信号/事件，避免在闭包内多次 borrow
    let mut show_replace_sig = props.show_replace;
    let mut replace_sig = props.replace;
    let on_replace_one = props.on_replace_one;
    let on_replace_all = props.on_replace_all;

    rsx! {
        div {
            style: "display:flex; flex-direction:column; gap:0; padding:0; background:#333; border-bottom:1px solid #3c3c3c;",

            // 查找行
            div {
                style: "display:flex; gap:0; align-items:center; padding:6px;",

                // 展开/收起替换区
                button {
                    style: "background:transparent; color:#ccc; border:none; width:28px; height:28px; \
                            cursor:pointer; display:flex; align-items:center; justify-content:center; \
                            font-size:16px; border-radius:4px; margin-right:4px;",
                    onclick: move |_| {
                        let cur = *show_replace_sig.read();
                        show_replace_sig.set(!cur);
                    },
                    if *props.show_replace.read() { "▼" } else { "▶" }
                }

                input {
                    style: "flex:1; background:#fff; color:#000; border:1px solid #454545; padding:6px 10px; \
                            font-family:'Monaco','Consolas',monospace; font-size:13px; border-radius:4px; margin-right:6px;",
                    value: "{props.query}",
                    placeholder: "查找",
                    oninput: move |e| props.on_query_input.call(e.value().clone()),
                    onkeydown: move |e: KeyboardEvent| {
                        if e.code() == Code::Enter && e.modifiers().is_empty() {
                            e.stop_propagation();
                            props.on_next.call(());
                        } else if e.code() == Code::Enter && e.modifiers().contains(Modifiers::SHIFT) {
                            e.stop_propagation();
                            props.on_prev.call(());
                        }
                    }
                }

                button {
                    style: "background:#3c3c3c; color:#ccc; border:1px solid #555; padding:6px 10px; \
                            border-radius:4px; cursor:pointer; font-size:12px; margin-right:4px;",
                    onclick: move |_| props.on_prev.call(()),
                    "↑"
                }
                button {
                    style: "background:#3c3c3c; color:#ccc; border:1px solid #555; padding:6px 10px; \
                            border-radius:4px; cursor:pointer; font-size:12px; margin-right:6px;",
                    onclick: move |_| props.on_next.call(()),
                    "↓"
                }
                button {
                    style: "background:#3c3c3c; color:#ccc; border:1px solid #555; padding:6px 10px; \
                            border-radius:4px; cursor:pointer; font-size:12px;",
                    onclick: move |_| props.on_close.call(()),
                    "✕"
                }
            }

            // 替换行
            if *props.show_replace.read() {
                div {
                    style: "display:flex; gap:6px; align-items:center; padding:6px; border-top:1px solid #3c3c3c;",

                    input {
                        style: "{replace_input_style}",
                        value: "{props.replace}",
                        placeholder: "替换为",
                        disabled: "{disabled}",
                        oninput: move |e| {
                            if !disabled {
                                replace_sig.set(e.value().clone());
                            }
                        },
                    }

                    // 替换当前
                    button {
                        style: "{replace_btn1_style}",
                        disabled: "{disabled}",
                        onclick: move |_| {
                            if !disabled {
                                if let Some(h) = on_replace_one.as_ref() {
                                    h.call(());
                                }
                            }
                        },
                        "替换"
                    }

                    // 全部替换
                    button {
                        style: "{replace_btn2_style}",
                        disabled: "{disabled}",
                        onclick: move |_| {
                            if !disabled {
                                if let Some(h) = on_replace_all.as_ref() {
                                    h.call(());
                                }
                            }
                        },
                        "全部替换"
                    }
                }
            }
        }
    }
}
