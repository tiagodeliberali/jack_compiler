use std::fs;
use std::{env, path::Path};

mod builder;
mod debug;
mod parser;
mod tokenizer;

use crate::builder::build_content;
use crate::debug::{debug_parsed_tree, debug_tokenizer};
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
