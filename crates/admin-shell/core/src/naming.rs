use convert_case::{Case, Casing};
use deunicode::deunicode;
use pinyin::ToPinyin;

/// 根据显示标题生成稳定的 snake_case 标识。
#[must_use]
pub fn identifier_from_title(title: &str) -> String {
    let transliterated = title
        .chars()
        .map(|character| {
            character.to_pinyin().map_or_else(
                || character.to_string(),
                |pinyin| format!(" {} ", pinyin.plain()),
            )
        })
        .collect::<String>();
    let normalized = deunicode(&transliterated).to_case(Case::Snake);
    let normalized = normalized
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || *character == '_')
        .collect::<String>();
    let normalized = normalized.trim_matches('_');
    if normalized.is_empty() {
        return String::new();
    }
    if normalized.starts_with(|character: char| character.is_ascii_digit())
        || rust_keyword(normalized)
    {
        return format!("item_{normalized}");
    }
    normalized.to_owned()
}

fn rust_keyword(value: &str) -> bool {
    matches!(
        value,
        "as" | "async"
            | "await"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "union"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
            | "try"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chinese_title_becomes_full_pinyin() {
        assert_eq!(identifier_from_title("资产管理"), "zi_chan_guan_li");
    }

    #[test]
    fn mixed_title_keeps_ascii_words() {
        assert_eq!(
            identifier_from_title("AIO 资产中心"),
            "aio_zi_chan_zhong_xin"
        );
        assert_eq!(identifier_from_title("API Keys"), "api_keys");
    }

    #[test]
    fn unsafe_module_names_receive_a_stable_prefix() {
        assert_eq!(identifier_from_title("123 服务"), "item_123_fu_wu");
        assert_eq!(identifier_from_title("type"), "item_type");
        assert!(identifier_from_title("……").is_empty());
    }
}
