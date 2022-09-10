use std::collections::HashSet;

use once_cell::sync::Lazy;
use unicode_normalization::UnicodeNormalization;
use unicode_xid::UnicodeXID;

static KEYWORDS: Lazy<HashSet<&str>> = Lazy::new(|| {
    vec![
        "_", "abstract", "as", "async", "await", "become", "box", "break", "const", "continue",
        "crate", "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "if",
        "impl", "in", "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv",
        "pub", "ref", "return", "Self", "self", "static", "struct", "super", "trait", "true",
        "try", "type", "typeof", "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
    ]
    .into_iter()
    .collect()
});

pub(crate) fn sanitize_component(raw: &str) -> String {
    let mut s = String::with_capacity(raw.len());
    let mut new_word = false;
    for ch in raw.chars() {
        if ch.is_xid_continue() && ch != '_' {
            if new_word || (s.is_empty() && !ch.is_xid_start()) {
                s.push('_');
                new_word = false;
            }
            s.push(ch);
        } else if !s.is_empty() {
            new_word = true
        }
    }
    if s.is_empty() {
        s.push_str("empty");
    }
    if KEYWORDS.contains(&&s[..]) {
        s.push('_');
    }
    if s.is_ascii() {
        s
    } else {
        s.nfc().collect::<String>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize() {
        assert_eq!(sanitize_component("foo"), "foo");
        assert_eq!(sanitize_component("foo_bar"), "foo_bar");
        assert_eq!(sanitize_component("FOO_BAR"), "FOO_BAR");
        assert_eq!(sanitize_component("foo-bar"), "foo_bar");
        assert_eq!(sanitize_component("abc#:def"), "abc_def");
        assert_eq!(sanitize_component("123abc"), "_123abc");
        assert_eq!(sanitize_component("abc123"), "abc123");
        assert_eq!(sanitize_component("123_abc"), "_123_abc");
        assert_eq!(sanitize_component("abc_123"), "abc_123");
        assert_eq!(sanitize_component("-123"), "_123");
        assert_eq!(sanitize_component("-abc"), "abc");
        assert_eq!(sanitize_component("abc-def-"), "abc_def");

        assert_eq!(sanitize_component("あいう"), "あいう");
        assert_eq!(sanitize_component("あいう×いろは"), "あいう_いろは");
        assert_eq!(sanitize_component("A\u{30A}"), "\u{C5}");

        assert_eq!(sanitize_component(""), "empty");
        assert_eq!(sanitize_component("_"), "empty");
        assert_eq!(sanitize_component("-"), "empty");
        assert_eq!(sanitize_component("abstract"), "abstract_");
        assert_eq!(sanitize_component("as"), "as_");
        assert_eq!(sanitize_component("async"), "async_");
        assert_eq!(sanitize_component("await"), "await_");
        assert_eq!(sanitize_component("become"), "become_");
        assert_eq!(sanitize_component("box"), "box_");
        assert_eq!(sanitize_component("break"), "break_");
        assert_eq!(sanitize_component("const"), "const_");
        assert_eq!(sanitize_component("continue"), "continue_");
        assert_eq!(sanitize_component("crate"), "crate_");
        assert_eq!(sanitize_component("do"), "do_");
        assert_eq!(sanitize_component("dyn"), "dyn_");
        assert_eq!(sanitize_component("else"), "else_");
        assert_eq!(sanitize_component("enum"), "enum_");
        assert_eq!(sanitize_component("extern"), "extern_");
        assert_eq!(sanitize_component("false"), "false_");
        assert_eq!(sanitize_component("final"), "final_");
        assert_eq!(sanitize_component("fn"), "fn_");
        assert_eq!(sanitize_component("for"), "for_");
        assert_eq!(sanitize_component("if"), "if_");
        assert_eq!(sanitize_component("impl"), "impl_");
        assert_eq!(sanitize_component("in"), "in_");
        assert_eq!(sanitize_component("let"), "let_");
        assert_eq!(sanitize_component("loop"), "loop_");
        assert_eq!(sanitize_component("macro"), "macro_");
        assert_eq!(sanitize_component("match"), "match_");
        assert_eq!(sanitize_component("mod"), "mod_");
        assert_eq!(sanitize_component("move"), "move_");
        assert_eq!(sanitize_component("mut"), "mut_");
        assert_eq!(sanitize_component("override"), "override_");
        assert_eq!(sanitize_component("priv"), "priv_");
        assert_eq!(sanitize_component("pub"), "pub_");
        assert_eq!(sanitize_component("ref"), "ref_");
        assert_eq!(sanitize_component("return"), "return_");
        assert_eq!(sanitize_component("Self"), "Self_");
        assert_eq!(sanitize_component("self"), "self_");
        assert_eq!(sanitize_component("static"), "static_");
        assert_eq!(sanitize_component("struct"), "struct_");
        assert_eq!(sanitize_component("super"), "super_");
        assert_eq!(sanitize_component("trait"), "trait_");
        assert_eq!(sanitize_component("true"), "true_");
        assert_eq!(sanitize_component("try"), "try_");
        assert_eq!(sanitize_component("type"), "type_");
        assert_eq!(sanitize_component("typeof"), "typeof_");
        assert_eq!(sanitize_component("unsafe"), "unsafe_");
        assert_eq!(sanitize_component("unsized"), "unsized_");
        assert_eq!(sanitize_component("use"), "use_");
        assert_eq!(sanitize_component("virtual"), "virtual_");
        assert_eq!(sanitize_component("where"), "where_");
        assert_eq!(sanitize_component("while"), "while_");
        assert_eq!(sanitize_component("yield"), "yield_");
        assert_eq!(sanitize_component("if_"), "if_");
        assert_eq!(sanitize_component("if-"), "if_");
        assert_eq!(sanitize_component("#if"), "if_");
        assert_eq!(sanitize_component("IF"), "IF");
    }
}
