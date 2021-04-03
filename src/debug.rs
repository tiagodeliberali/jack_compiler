use crate::parser::*;
use crate::tokenizer::{TokenType, Tokenizer};
use std::fs;

pub fn debug_tokenizer(filename: &str, tokenizer: &Tokenizer) {
    let printable_tokens = print_tokens(tokenizer);

    fs::write(
        filename.replace(".jack", "T.xml"),
        printable_tokens.join("\r\n"),
    )
    .expect("Something failed on write file to disk");
}

pub fn debug_parsed_tree(filename: &str, root: &TokenTreeItem) {
    let mut result: Vec<String> = Vec::new();

    result.extend(debug_token_item(root));
    result.push(String::new());

    fs::write(filename.replace(".jack", ".xml"), result.join("\r\n"))
        .expect("Something failed on write file to disk");
}

fn debug_token_item(item: &TokenTreeItem) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();

    if let Some(name) = &item.get_name() {
        result.push(format!("<{}>", name));
    }

    if let Some(item) = &item.get_item() {
        result.push(format!(
            "<{}> {} </{}>",
            enum_to_str(item.get_type()),
            parse_symbol(&item.get_value().as_str()),
            enum_to_str(item.get_type())
        ));
    }

    for node in item.get_nodes() {
        result.extend(debug_token_item(&node));
    }

    if let Some(name) = &item.get_name() {
        result.push(format!("</{}>", name));
    }

    result
}

fn enum_to_str(value: TokenType) -> String {
    let result = match value {
        TokenType::Identifier => "identifier",
        TokenType::Integer => "integerConstant",
        TokenType::Keyword => "keyword",
        TokenType::None => "ERROR!",
        TokenType::String => "stringConstant",
        TokenType::Symbol => "symbol",
    };

    String::from(result)
}

fn print_tokens(tokenizer: &Tokenizer) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    result.push(String::from("<tokens>"));

    while tokenizer.has_next() {
        let token = tokenizer.get_next();
        let token = token.unwrap();

        let token_type = match token.get_type() {
            TokenType::Identifier => "identifier",
            TokenType::Integer => "integerConstant",
            TokenType::Keyword => "keyword",
            TokenType::String => "stringConstant",
            TokenType::Symbol => "symbol",
            _ => "SOMETHING GOES WRONG HERE",
        };

        result.push(format!(
            "<{}> {} </{}>",
            token_type,
            parse_symbol(token.get_value().trim()),
            token_type
        ));
    }
    result.push(String::from("</tokens>"));

    result
}

fn parse_symbol(value: &str) -> String {
    if value == ">" {
        return String::from("&gt;");
    }

    if value == "<" {
        return String::from("&lt;");
    }

    if value == "&" {
        return String::from("&amp;");
    }

    if value == "\"" {
        return String::from("&quot;");
    }

    String::from(value)
}
