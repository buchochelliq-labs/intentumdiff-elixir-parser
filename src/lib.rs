//! Elixir parser plugin — full-parse mode.
//!
//! Handles `.ex` and `.exs` files.
//! The plugin parses source with Tree-sitter inside Rust/Wasm.
//!
//! In tree-sitter-elixir, `def`/`defp`/`defmodule`/`defmacro` etc. are all
//! represented as `call` nodes whose first identifier child is the macro name.
//! This plugin lifts those into recognisable semantic constructs.

use intentdiff_plugin_sdk::{
    cst::CstNode,
    hash::structural_hash_with_memo,
    tree::{SemanticNode, SemanticNodeBuilder},
};

wit_bindgen::generate!({
    path: "wit/plugin.wit",
    world: "parser-plugin",
});

use crate::exports::intentdiff::plugin::parser::ExamplePair;
use crate::exports::intentdiff::plugin::parser::Guest;
use crate::exports::intentdiff::plugin::parser::LanguageInfoRecord;
use crate::exports::intentdiff::plugin::parser::ParserMode;

const PLUGIN_METADATA: &str = include_str!("../plugin_metadata.info");

fn language_info_for(ids: Vec<String>) -> Vec<LanguageInfoRecord> {
    let metadata = intentdiff_plugin_sdk::metadata::parse_plugin_metadata(PLUGIN_METADATA);
    ids.into_iter()
        .map(|language_id| {
            let info = metadata.language_or_default(&language_id);
            LanguageInfoRecord {
                language_id: info.language_id,
                language_name: info.language_name,
                language_short_name: info.language_short_name,
                monaco_language: info.monaco_language,
                default_filename: info.default_filename,
                language_file_extensions: info.language_file_extensions,
                author: metadata.author().to_string(),
                plugin_version: metadata.plugin_version().to_string(),
                last_updated: metadata.last_updated().to_string(),
            }
        })
        .collect()
}
struct ElixirParser;

const TRIVIA: &[&str] = &["comment", "whitespace"];

/// Elixir macro names that introduce named definitions.
const DEFINITION_MACROS: &[&str] = &[
    "def",
    "defp",
    "defmodule",
    "defmacro",
    "defmacrop",
    "defprotocol",
    "defimpl",
    "defstruct",
    "defenum",
    "defdelegate",
    "defguard",
    "defguardp",
    "defoverridable",
];

const SEMANTIC_TYPES: &[&str] = &[
    "source_file",
    // All def-like constructs are `call` nodes in tree-sitter-elixir
    "call",
    "anonymous_function",
    "do_block",
    "body",
    "stab_clause",
    // Control flow
    "if_expression",
    "unless_expression",
    "case_expression",
    "cond_expression",
    "with_expression",
    "for_expression",
    "receive_expression",
    "try_expression",
    "rescue_clause",
    "catch_clause",
    "after_clause",
    "else_clause",
    // Pipes and binary ops
    "binary_operator",
    // Literals
    "string",
    "charlist",
    "atom",
    "quoted_atom",
    "integer",
    "float",
    "boolean",
    "nil",
    "tuple",
    "list",
    "map",
    "identifier",
    "alias",
];

fn is_semantic(node_type: &str) -> bool {
    SEMANTIC_TYPES.contains(&node_type)
}

/// Extract the macro keyword (first identifier child of a call node).
fn call_macro_name(node: &CstNode) -> Option<&str> {
    for child in &node.children {
        if child.node_type == "identifier" {
            let text = child.text_or_empty();
            if DEFINITION_MACROS.contains(&text) {
                return Some(text);
            }
        }
        // In some grammar versions the macro name may be nested under "dot"
        break; // only first child matters
    }
    None
}

/// For `def foo(...)` / `defmodule Foo do` — extract the defined name.
fn definition_name(node: &CstNode) -> Option<String> {
    // Arguments node holds the function/module name as its first child
    for child in &node.children {
        if child.node_type == "arguments" {
            if let Some(first) = child.children.first() {
                match first.node_type.as_str() {
                    "identifier" | "alias" | "atom" => {
                        return Some(first.text_or_empty().to_string());
                    }
                    "call" => {
                        // def foo(x) — inner call node holds function name
                        return Some(label_for(first));
                    }
                    _ => {}
                }
            }
            break;
        }
    }
    None
}

fn label_for(node: &CstNode) -> String {
    // A do_block is a body CONTAINER, identified structurally — never by its source text.
    // An EMPTY `do ... end` has no named children so tree-sitter yields a leaf, and labelling
    // it with its text (`do\n  end`) made a trivial-body -> real-body edit flip the label
    // (leaf-text -> structural), which kept a redundant parent MODIFICATION on top of the
    // real body ADDITION and blocked routing elixir (issue #62 / #57). Always structural,
    // matching how go/java label their (never-text-labelled) blocks.
    if node.node_type == "do_block" {
        return "do_block".to_string();
    }
    if node.is_leaf() {
        return node.text_or_empty().to_string();
    }
    // Literal containers label with their captured source text (SDK-shared, issue #47).
    if let Some(label) = intentdiff_plugin_sdk::ts_convert::literal_label(node) {
        return label;
    }
    match node.node_type.as_str() {
        "call" => {
            if call_macro_name(node).is_some() {
                if let Some(name) = definition_name(node) {
                    return name;
                }
            }
            // Non-definition call: use first identifier as label
            for child in &node.children {
                if child.node_type == "identifier" || child.node_type == "alias" {
                    return child.text_or_empty().to_string();
                }
            }
        }
        "anonymous_function" => return "fn".to_string(),
        "alias" | "identifier" | "atom" | "quoted_atom" => {
            return node.text_or_empty().to_string();
        }
        _ => {}
    }
    node.node_type.clone()
}

fn is_class_like(node: &CstNode) -> bool {
    if node.node_type == "call" {
        if let Some(m) = call_macro_name(node) {
            return matches!(m, "defmodule" | "defprotocol" | "defimpl");
        }
    }
    false
}

fn is_method_like(node: &CstNode) -> bool {
    if node.node_type == "call" {
        if let Some(m) = call_macro_name(node) {
            return matches!(
                m,
                "def"
                    | "defp"
                    | "defmacro"
                    | "defmacrop"
                    | "defdelegate"
                    | "defguard"
                    | "defguardp"
            );
        }
    }
    false
}

fn convert(
    node: &CstNode,
    id_prefix: &str,
    parent_class: Option<&str>,
    memo: &mut std::collections::HashMap<usize, String>,
) -> Option<SemanticNode> {
    if TRIVIA.contains(&node.node_type.as_str()) {
        return None;
    }

    let owned_class: Option<String> = if is_class_like(node) {
        Some(label_for(node))
    } else {
        parent_class.map(|s| s.to_string())
    };

    let children: Vec<SemanticNode> = node
        .children
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            convert(
                c,
                &format!("{}.{}", id_prefix, i),
                owned_class.as_deref(),
                memo,
            )
        })
        .collect();
    if !is_semantic(&node.node_type) && children.is_empty() {
        return None;
    }

    let hash = structural_hash_with_memo(node, memo);
    let mut builder = SemanticNodeBuilder::new(
        id_prefix,
        &node.node_type,
        label_for(node),
        node.start_line,
        node.start_col,
        node.end_line,
        node.end_col,
        hash,
    )
    .children(children);

    if is_method_like(node) {
        if let Some(class_name) = parent_class {
            builder = builder.parent_type(class_name);
        }
    }

    Some(builder.build())
}



use intentdiff_plugin_sdk::ts_convert::node_to_cst;

fn parse_source(source: &str) -> Result<CstNode, String> {
    let mut parser = tree_sitter::Parser::new();
    let lang = tree_sitter_elixir::LANGUAGE.into();
    parser
        .set_language(&lang)
        .map_err(|_| "Failed to load elixir grammar".to_string())?;
    let tree = parser
        .parse(source, None)
        .ok_or_else(|| "Parse failed".to_string())?;
    Ok(node_to_cst(tree.root_node(), source.as_bytes()))
}

fn process_impl(source: &str) -> String {
    let root: CstNode = match parse_source(source) {
        Ok(n) => n,
        Err(e) => return format!(r#"{{\"error\":\"{}\"}}"#, e),
    };
    let mut memo: std::collections::HashMap<usize, String> = std::collections::HashMap::new();
    let sem = match convert(&root, "0", None, &mut memo) {
        Some(n) => n,
        None => return r#"{"error":"Empty semantic tree"}"#.to_string(),
    };
    match serde_json::to_string(&sem) {
        Ok(s) => s,
        Err(e) => format!(r#"{{"error":"Serialisation error: {}"}}"#, e),
    }
}

impl Guest for ElixirParser {
    fn get_parser_mode() -> ParserMode {
        ParserMode::FullParse
    }
    fn grammar_id() -> String {
        "elixir".to_string()
    }
    fn detect_language(filename: String, _content: String) -> String {
        let lower = filename.to_lowercase();
        if lower.ends_with(".ex") || lower.ends_with(".exs") {
            return "elixir".to_string();
        }
        String::new()
    }
    fn preprocess_source(source: String) -> String {
        source
    }
    fn example(_language: String) -> ExamplePair {
        ExamplePair {
            old: "defmodule Greeter do\n  def greet(name) do\n    IO.puts(\"Hello, \" <> name)\n  end\n\n  def add(a, b) do\n    a + b\n  end\nend\n".to_string(),
            new: "defmodule Greeter do\n  def greet(name) do\n    IO.puts(\"Hello, #{name}!\")\n  end\n\n  def add(x, y), do: x + y\n\n  def multiply(x, y), do: x * y\nend\n".to_string(),
        }
    }
    fn process(input: String, _language: String, _filename: String) -> String {
        process_impl(&input)
    }
    fn trivia_node_types() -> Vec<String> {
        TRIVIA.iter().map(|s| s.to_string()).collect()
    }
    fn language_ids() -> Vec<String> {
        vec!["elixir".to_string()]
    }
    fn language_info() -> Vec<LanguageInfoRecord> {
        language_info_for(Self::language_ids())
    }
    fn priority() -> i32 {
        0
    }
}

export!(ElixirParser);

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exports::intentdiff::plugin::parser::Guest;
    use intentdiff_plugin_sdk::testing as t;

    #[test]
    fn grammar_id_nonempty() {
        assert!(!ElixirParser::grammar_id().is_empty());
    }

    #[test]
    fn language_ids_contain_grammar_id() {
        let gid = ElixirParser::grammar_id();
        let ids = ElixirParser::language_ids();
        assert!(
            ids.contains(&gid),
            "language_ids {:?} must contain {:?}",
            ids,
            gid
        );
    }

    #[test]
    fn detect_language_known_ext() {
        let r = ElixirParser::detect_language("test.ex".to_string(), "".to_string());
        assert_eq!(r.as_str(), "elixir");
    }

    #[test]
    fn detect_language_unknown_ext() {
        let r =
            ElixirParser::detect_language("test.xyz_notareal_ext_9z8y".to_string(), "".to_string());
        assert_eq!(r.as_str(), "");
    }

    #[test]
    fn parser_mode_is_full_parse() {
        assert!(matches!(
            ElixirParser::get_parser_mode(),
            ParserMode::FullParse
        ));
    }

    #[test]
    fn process_impl_accepts_raw_example_source() {
        let example = ElixirParser::example(ElixirParser::grammar_id());
        let out = process_impl(&example.old);
        t::assert_valid_json(&out, "process(raw example)");
        assert!(!out.contains("\"error\""), "{out}");
    }
    #[test]
    fn process_impl_empty_returns_valid_json() {
        let out = process_impl("");
        t::assert_valid_json(&out, "process(empty)");
    }

    #[test]
    fn process_impl_whitespace_returns_valid_json() {
        let out = process_impl("   \n  ");
        t::assert_valid_json(&out, "process(whitespace)");
    }

    #[test]
    fn empty_and_filled_do_blocks_share_a_structural_label() {
        // Issue #62: an empty `do ... end` (a leaf) must not be labelled with its text, or a
        // trivial-body -> real-body edit flips the label and keeps a redundant parent
        // modification. Both empty and filled do_blocks label as "do_block".
        fn do_block_labels(source: &str) -> Vec<String> {
            let root: SemanticNode = serde_json::from_str(&process_impl(source)).unwrap();
            fn walk(node: &SemanticNode, out: &mut Vec<String>) {
                if node.node_type == "do_block" {
                    out.push(node.label.clone());
                }
                for child in &node.children {
                    walk(child, out);
                }
            }
            let mut out = Vec::new();
            walk(&root, &mut out);
            out
        }
        for label in do_block_labels("defmodule M do\n  def f do\n  end\nend\n") {
            assert_eq!(label, "do_block", "empty do_block must be structural");
        }
        for label in do_block_labels("defmodule M do\n  def f do\n    IO.puts(\"x\")\n  end\nend\n") {
            assert_eq!(label, "do_block", "filled do_block must be structural");
        }
    }
}
