// src/component/util/highlight.rs
use html_escape;

/// 在格式化后的 JSON 文本上叠加“搜索高亮”
/// 注意：这是非常轻量的文本级高亮（不做词法定位），可能会在 HTML 标签里命中；
/// 若需更稳健可改用词法切分后再包裹 span。
pub fn highlight_json_with_search(json: &str, query: &str) -> String {
    let mut html = highlight_json(json);

    if !query.is_empty() {
        // 当前第一处命中：加强色
        let strong = format!(
            "<span style=\"background:#ffcc00;color:#000;\">{}</span>",
            html_escape::encode_text(query)
        );

        if let Some(pos) = html.find(query) {
            html = format!("{}{}{}", &html[..pos], strong, &html[pos + query.len()..]);

            // 其他命中：弱化色（再次替换）
            let weak = format!(
                "<span style=\"background:rgba(255,204,0,0.35);\">{}</span>",
                html_escape::encode_text(query)
            );
            html = html.replace(query, &weak);
        }
    }

    html
}

/// 简单 JSON 语法高亮（纯文本处理，适合展示已 pretty 的 JSON）
pub fn highlight_json(json: &str) -> String {
    let mut result = String::new();
    let mut chars = json.chars().peekable();
    let mut in_string = false;
    let mut is_key = false;
    let mut escape = false;

    while let Some(ch) = chars.next() {
        match ch {
            '"' if !escape => {
                in_string = !in_string;
                if in_string {
                    if is_key {
                        result.push_str("<span style=\"color: #9cdcfe\">\"");
                    } else {
                        result.push_str("<span style=\"color: #ce9178\">\"");
                    }
                } else {
                    result.push_str("\"</span>");
                    if is_key {
                        is_key = false;
                    }
                }
            }
            ':' if !in_string => {
                result.push(':');
                is_key = false;
            }
            '{' | '[' if !in_string => {
                result.push(ch);
                is_key = true;
            }
            '}' | ']' if !in_string => {
                result.push(ch);
            }
            ',' if !in_string => {
                result.push(ch);
                is_key = true;
            }
            't' if !in_string && chars.peek() == Some(&'r') => {
                result.push_str("<span style=\"color: #569cd6\">t");
                chars.next();
                chars.next();
                chars.next();
                result.push_str("rue</span>");
            }
            'f' if !in_string && chars.peek() == Some(&'a') => {
                result.push_str("<span style=\"color: #569cd6\">f");
                chars.next();
                chars.next();
                chars.next();
                chars.next();
                result.push_str("alse</span>");
            }
            'n' if !in_string && chars.peek() == Some(&'u') => {
                result.push_str("<span style=\"color: #569cd6\">n");
                chars.next();
                chars.next();
                chars.next();
                result.push_str("ull</span>");
            }
            '\\' if in_string => {
                escape = true;
                result.push(ch);
                continue;
            }
            _ => {
                result.push(ch);
            }
        }
        escape = false;
    }

    result
}
