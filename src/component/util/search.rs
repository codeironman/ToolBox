use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct SearchBarProps {
    /// 是否显示查找条
    pub show: Signal<bool>,
    /// 是否展开替换行
    pub show_replace: Signal<bool>,

    /// 查找关键字
    pub query: Signal<String>,
    /// 替换关键字
    pub replace: Signal<String>,

    /// 上/下一处、关闭
    pub on_prev: EventHandler<()>,
    pub on_next: EventHandler<()>,
    pub on_close: EventHandler<()>,

    /// 查找输入变化时触发
    pub on_query_input: EventHandler<String>,

    /// 替换当前 / 全部替换
    pub on_replace_one: Option<EventHandler<()>>,
    pub on_replace_all: Option<EventHandler<()>>,

    /// 是否禁用替换（右侧输出面板传 true）
    #[props(default = false)]
    pub replace_disabled: bool,

    /// 当前匹配索引（从 0 开始）
    #[props(default = 0)]
    pub match_current: usize,
    /// 匹配总数
    #[props(default = 0)]
    pub match_total: usize,
}

#[component]
pub fn SearchBar(props: SearchBarProps) -> Element {
    // 不显示就渲染空节点
    if !*props.show.read() {
        return rsx! { Fragment {} };
    }

    let disabled = props.replace_disabled;
    let total = props.match_total;
    let cur = props.match_current;
    // total==0 显示 "0/0"，否则 "{cur+1}/{total}"
    let count_text = if total == 0 {
        "0/0".to_string()
    } else {
        format!("{}/{}", cur + 1, total)
    };
    let count_color = if total == 0 { "#6a6a6a" } else { "var(--text-dim)" };

    // 预先克隆必要的信号/事件，避免在闭包内多次 borrow
    let mut show_replace_sig = props.show_replace;
    let mut replace_sig = props.replace;
    let on_replace_one = props.on_replace_one;
    let on_replace_all = props.on_replace_all;

    rsx! {
        div {
            style: "display:flex; flex-direction:column; gap:0; padding:0; background:var(--bg-header); border-bottom:1px solid var(--border);",

            // 查找行
            div {
                style: "display:flex; gap:6px; align-items:center; padding:6px 8px;",

                // 展开/收起替换区
                button {
                    class: "tb-icon-btn",
                    style: "width:24px; height:24px; font-size:11px; margin-right:2px;",
                    onclick: move |_| {
                        let c = *show_replace_sig.read();
                        show_replace_sig.set(!c);
                    },
                    if *props.show_replace.read() { "▼" } else { "▶" }
                }

                input {
                    class: "tb-search-input",
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

                span {
                    class: "tb-count",
                    style: "color:{count_color};",
                    "{count_text}"
                }

                button {
                    class: "tb-icon-btn",
                    style: "width:26px; height:26px; font-size:14px;",
                    onclick: move |_| props.on_prev.call(()),
                    "↑"
                }
                button {
                    class: "tb-icon-btn",
                    style: "width:26px; height:26px; font-size:14px;",
                    onclick: move |_| props.on_next.call(()),
                    "↓"
                }
                button {
                    class: "tb-icon-btn",
                    style: "width:26px; height:26px; font-size:14px;",
                    onclick: move |_| props.on_close.call(()),
                    "✕"
                }
            }

            // 替换行
            if *props.show_replace.read() {
                div {
                    style: "display:flex; gap:6px; align-items:center; padding:6px 8px; border-top:1px solid var(--border-soft);",

                    input {
                        class: "tb-search-input",
                        style: if disabled { "opacity:.5;" } else { "" },
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
                        class: "tb-btn",
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
                        class: "tb-btn-primary",
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
