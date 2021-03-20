use std::fs;
use std::{env, path::Path};

mod builder;
mod debug;
mod parser;
mod tokenizer;

use crate::builder::build_content;
use crate::debug::print_tokens;
use crate::parser::ClassNode;
use crate::tokenizer::Tokenizer;

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

    debug_tokenizer(filename, &mut tokenizer);

    let root = ClassNode::build(&mut tokenizer);

    debug_parsed_tree(&filename, &root);
}

fn debug_tokenizer(filename: &str, tokenizer: &mut Tokenizer) {
    let printable_tokens = print_tokens(tokenizer);

    fs::write(
        filename.replace(".jack", "T2.xml"),
        printable_tokens.join("\r\n"),
    )
    .expect("Something failed on write file to disk");

    tokenizer.reset();
}

fn debug_parsed_tree(filename: &str, root_node: &ClassNode) {
    let mut result: Vec<String> = Vec::new();

    fs::write(filename.replace(".jack", "2.xml"), result.join("\r\n"))
        .expect("Something failed on write file to disk");
}
