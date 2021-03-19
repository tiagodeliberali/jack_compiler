use std::fs;
use std::{env, path::Path};

mod builder;
mod tokenizer;
mod parser;

use crate::builder::build_content;
use crate::tokenizer::{TokenType, Tokenizer};
use crate::parser::ClassNode;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = args.get(1).expect("Please supply a folder or file name");

    if path.ends_with(".jack") {
        parse_file(&path);
    } else {
        let file_list = fs::read_dir(path).unwrap();

        for file in file_list {
            let file_path_buff = file.unwrap().path();
            let file_path = file_path_buff.to_str().unwrap();
            let file_name = Path::new(file_path).file_name().unwrap().to_str().unwrap();

            if file_name.ends_with(".jack") {
                parse_file(&file_path);
            }
        }
    }
}

fn parse_file(filename: &str) {
    let content = fs::read_to_string(filename).expect("Something went wrong reading the file");

    let clean_code = build_content(content);

    let mut tokenizer = Tokenizer::new(&clean_code);

    print_tokens(&mut tokenizer, &filename);

    let root = ClassNode::build(&mut tokenizer);

    let mut result: Vec<String> = Vec::new();

    fs::write(filename.replace(".jack", "2.xml"), result.join("\r\n"))
        .expect("Something failed on write file to disk");
}

fn print_tokens(tokenizer: &mut Tokenizer, filename: &str) {
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

    fs::write(filename.replace(".jack", "T2.xml"), result.join("\r\n"))
        .expect("Something failed on write file to disk");
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
