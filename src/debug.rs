use crate::parser::ClassNode;
use crate::tokenizer::{TokenType, Tokenizer};
use std::fs;

pub fn debug_tokenizer(filename: &str, tokenizer: &mut Tokenizer) {
    let printable_tokens = print_tokens(tokenizer);

    fs::write(
        filename.replace(".jack", "T2.xml"),
        printable_tokens.join("\r\n"),
    )
    .expect("Something failed on write file to disk");
}

pub fn debug_parsed_tree(filename: &str, root_node: &ClassNode) {
    let mut result: Vec<String> = Vec::new();

    result.push(String::from("<class>"));
    result.push(String::from("<keyword> class </keyword>"));
    result.push(format!(
        "<identifier> {} </identifier>",
        root_node.get_identifier()
    ));
    result.push(String::from("<symbol> { </symbol>"));
    result.push(String::from("</class>"));

    fs::write(filename.replace(".jack", "2.xml"), result.join("\r\n"))
        .expect("Something failed on write file to disk");
}

fn print_tokens(tokenizer: &mut Tokenizer) -> Vec<String> {
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
