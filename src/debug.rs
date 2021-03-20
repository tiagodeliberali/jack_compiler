use crate::tokenizer::{TokenType, Tokenizer};

pub fn print_tokens(tokenizer: &mut Tokenizer) -> Vec<String> {
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
