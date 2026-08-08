// src/component/util/hignlight.rs
//
// JSON 语法高亮 + 搜索高亮。
//
// 设计要点（修复旧版在已高亮 HTML 上做 find/replace 导致结构被破坏的 bug）：
// 1. 先把“搜索匹配”转成**字节级标记数组**（0=无 / 1=弱 / 2=强=当前匹配）。
//    匹配区间来自纯文本上的字节位置，必定落在 UTF-8 字符边界上，
//    因此一个字符的所有字节标记一致，取首字节标记即可代表整字符。
// 2. 用状态机**逐字符**遍历 JSON，确定每个字符的语法色
//    （key / string / keyword / number / default），并在转义后追加到当前段。
// 3. 用 SegmentBuilder 把相邻“同(语法色, 搜索标记)”的字符合并为一段，
//    最后输出：搜索高亮 span 包在语法 span 外层，两者不再互相破坏。

// ---- 语法色 ----
const COLOR_KEY: &str = "#9cdcfe";
const COLOR_STRING: &str = "#ce9178";
const COLOR_KEYWORD: &str = "#569cd6";
const COLOR_NUMBER: &str = "#b5cea8";

// ---- 搜索高亮 ----
const STRONG_STYLE: &str = "background:#ffcc00;color:#000;";
const WEAK_STYLE: &str = "background:rgba(255,204,0,.35);";

/// 语法高亮 + 搜索高亮。
///
/// - `json`: pretty 后的纯文本
/// - `query`: 搜索词（保留参数以兼容旧调用，内部以 `positions` 为准）
/// - `positions`: 每处匹配的**起始字节位置**
/// - `current_idx`: 当前匹配在 `positions` 中的索引（用加强色高亮）
pub fn highlight_json_with_search(
    json: &str,
    query: &str,
    positions: &[usize],
    current_idx: usize,
) -> String {
    // 1. 字节级匹配标记
    let n = json.len();
    let q_len = query.len();
    let mut mark = vec![0u8; n];
    if q_len > 0 {
        for (i, &start) in positions.iter().enumerate() {
            let end = start.saturating_add(q_len);
            if start < n && end <= n {
                let val = if i == current_idx { 2 } else { 1 };
                for b in &mut mark[start..end] {
                    *b = val;
                }
            }
        }
    }

    // 2. 状态机逐字符 -> 段
    let mut builder = SegmentBuilder::new();
    let mut chars = json.char_indices().peekable();
    let mut in_string = false;
    let mut is_key_context = false; // 下一个字符串是否为 key
    let mut string_is_key = false; // 当前字符串是否为 key
    let mut escape = false;
    // 容器类型栈：true=对象，false=数组。用于判断字符串是 key 还是 value。
    let mut container_stack: Vec<bool> = Vec::new();

    while let Some((byte_pos, ch)) = chars.next() {
        if in_string {
            let color = string_color(string_is_key);
            let m = mark_at(&mark, byte_pos);
            if escape {
                escape = false;
                builder.push(color, m, ch);
            } else if ch == '\\' {
                escape = true;
                builder.push(color, m, ch);
            } else if ch == '"' {
                builder.push(color, m, ch);
                in_string = false;
                if string_is_key {
                    is_key_context = false;
                }
            } else {
                builder.push(color, m, ch);
            }
            continue;
        }

        // 非字符串
        match ch {
            '"' => {
                in_string = true;
                string_is_key = is_key_context;
                builder.push(string_color(string_is_key), mark_at(&mark, byte_pos), ch);
            }
            '{' => {
                container_stack.push(true);
                is_key_context = true;
                builder.push(None, mark_at(&mark, byte_pos), ch);
            }
            '[' => {
                container_stack.push(false);
                is_key_context = false;
                builder.push(None, mark_at(&mark, byte_pos), ch);
            }
            ',' => {
                // 对象中逗号后是下一个 key；数组中逗号后是下一个 value
                is_key_context = container_stack.last().copied().unwrap_or(true);
                builder.push(None, mark_at(&mark, byte_pos), ch);
            }
            ':' => {
                is_key_context = false;
                builder.push(None, mark_at(&mark, byte_pos), ch);
            }
            '}' | ']' => {
                container_stack.pop();
                builder.push(None, mark_at(&mark, byte_pos), ch);
            }
            ' ' | '\t' | '\n' | '\r' => {
                builder.push(None, mark_at(&mark, byte_pos), ch);
            }
            't' if json[byte_pos..].starts_with("true") => {
                push_keyword(&mut builder, &mut chars, &mark, byte_pos, 't', "true");
            }
            'f' if json[byte_pos..].starts_with("false") => {
                push_keyword(&mut builder, &mut chars, &mark, byte_pos, 'f', "false");
            }
            'n' if json[byte_pos..].starts_with("null") => {
                push_keyword(&mut builder, &mut chars, &mark, byte_pos, 'n', "null");
            }
            c if c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | 'e' | 'E') => {
                builder.push(Some(COLOR_NUMBER), mark_at(&mark, byte_pos), ch);
            }
            _ => {
                builder.push(None, mark_at(&mark, byte_pos), ch);
            }
        }
    }

    builder.finish()
}

// ============== 内部辅助 ==============

fn string_color(is_key: bool) -> Option<&'static str> {
    Some(if is_key { COLOR_KEY } else { COLOR_STRING })
}

fn mark_at(mark: &[u8], byte_pos: usize) -> u8 {
    if byte_pos < mark.len() {
        mark[byte_pos]
    } else {
        0
    }
}

/// 把一个关键字（true/false/null）整词染成 keyword 色。
/// `first_ch` 已经由调用方从迭代器取出，这里负责 push 它，再消费剩余字符。
fn push_keyword(
    builder: &mut SegmentBuilder,
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    mark: &[u8],
    first_pos: usize,
    first_ch: char,
    word: &str,
) {
    builder.push(Some(COLOR_KEYWORD), mark_at(mark, first_pos), first_ch);
    // 消费 word 剩下的字符（ASCII，1 字节 = 1 字符）
    for _ in 1..word.len() {
        if let Some((bp, c)) = chars.next() {
            builder.push(Some(COLOR_KEYWORD), mark_at(mark, bp), c);
        }
    }
}

/// 一个连续的、同(语法色, 搜索标记)的文本段（已 HTML 转义）。
struct Segment {
    color: Option<&'static str>,
    search: u8, // 0 / 1 / 2
    text: String,
}

struct SegmentBuilder {
    segments: Vec<Segment>,
    cur_color: Option<&'static str>,
    cur_search: u8,
    cur_text: String,
}

impl SegmentBuilder {
    fn new() -> Self {
        Self {
            segments: Vec::new(),
            cur_color: None,
            cur_search: 0,
            cur_text: String::new(),
        }
    }

    fn push(&mut self, color: Option<&'static str>, search: u8, ch: char) {
        if color != self.cur_color || search != self.cur_search {
            self.flush();
            self.cur_color = color;
            self.cur_search = search;
        }
        match ch {
            '<' => self.cur_text.push_str("&lt;"),
            '>' => self.cur_text.push_str("&gt;"),
            '&' => self.cur_text.push_str("&amp;"),
            '"' => self.cur_text.push_str("&quot;"),
            _ => self.cur_text.push(ch),
        }
    }

    fn flush(&mut self) {
        if !self.cur_text.is_empty() {
            self.segments.push(Segment {
                color: self.cur_color,
                search: self.cur_search,
                text: std::mem::take(&mut self.cur_text),
            });
        }
    }

    fn finish(mut self) -> String {
        self.flush();
        let mut html = String::with_capacity(self.segments.len() * 24);
        for seg in &self.segments {
            match seg.search {
                // 当前匹配：黑字黄底，最醒目（覆盖语法色）
                2 => {
                    html.push_str("<span style=\"");
                    html.push_str(STRONG_STYLE);
                    html.push_str("\">");
                    html.push_str(&seg.text);
                    html.push_str("</span>");
                }
                // 其它匹配：半透明黄底，保留语法色
                1 => {
                    html.push_str("<span style=\"");
                    html.push_str(WEAK_STYLE);
                    html.push_str("\">");
                    if let Some(c) = seg.color {
                        html.push_str("<span style=\"color:");
                        html.push_str(c);
                        html.push_str(";\">");
                        html.push_str(&seg.text);
                        html.push_str("</span>");
                    } else {
                        html.push_str(&seg.text);
                    }
                    html.push_str("</span>");
                }
                // 无搜索：仅语法色
                _ => {
                    if let Some(c) = seg.color {
                        html.push_str("<span style=\"color:");
                        html.push_str(c);
                        html.push_str(";\">");
                        html.push_str(&seg.text);
                        html.push_str("</span>");
                    } else {
                        html.push_str(&seg.text);
                    }
                }
            }
        }
        html
    }
}
