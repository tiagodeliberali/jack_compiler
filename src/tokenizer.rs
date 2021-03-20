pub struct Tokenizer {
    tokens: Vec<TokenItem>,
    cursor: usize,
}

impl Tokenizer {
    pub fn new(code: &str) -> Tokenizer {
        let tokens = process_code(&code);
        Tokenizer { tokens, cursor: 0 }
    }

    pub fn reset(&mut self) {
        self.cursor = 0;
    }

    pub fn has_next(&self) -> bool {
        self.tokens.len() > self.cursor
    }

    pub fn get_next(&mut self) -> Option<&TokenItem> {
        if self.has_next() {
            let cursor = self.cursor;

            self.cursor = cursor + 1;

            return self.tokens.get(cursor);
        }
        None
    }

    pub fn peek_next(&self) -> Option<&TokenItem> {
        if self.has_next() {
            return self.tokens.get(self.cursor);
        }
        None
    }

    pub fn consume(&mut self, value: &str) {
        let token = self.get_next().unwrap();

        if token.get_value() != value {
            panic!(
                "Invalid token found. Expected {} and received {}",
                value,
                token.get_value()
            )
        }
    }

    pub fn retrieve_identifier(&mut self) -> String {
        self.retrieve(TokenType::Identifier)
    }

    fn retrieve(&mut self, expected_type: TokenType) -> String {
        let token = self.get_next().unwrap();

        if token.get_type() != expected_type {
            panic!(
                "Invalid token type found. Expected {:?} and received {:?}",
                expected_type,
                token.get_type()
            )
        }

        token.get_value()
    }

    pub fn retrieve_type(&mut self) -> String {
        let type_keywords: [&str; 3] = ["int", "char", "boolean"];
        let token = self.retrieve_any(Vec::from([TokenType::Identifier, TokenType::Keyword]));

        if token.get_type() == TokenType::Keyword {
            if !type_keywords.contains(&token.get_value().as_str()) {
                panic!(format!(
                    "Invalid keywork. Expected {:?}, but found {}",
                    type_keywords,
                    token.get_value()
                ));
            }
        }

        token.get_value()
    }

    fn retrieve_any(&mut self, expected_type: Vec<TokenType>) -> &TokenItem {
        let token = self.get_next().unwrap();

        if !expected_type.contains(&token.get_type()) {
            panic!(
                "Invalid token type found. Expected {:?} and received {:?}",
                expected_type,
                token.get_type()
            )
        }

        token
    }
}

pub struct TokenItem {
    token_type: TokenType,
    value: String,
}

impl TokenItem {
    pub fn new(value: &str, token_type: TokenType) -> TokenItem {
        TokenItem {
            value: String::from(value),
            token_type,
        }
    }

    pub fn get_type(&self) -> TokenType {
        self.token_type
    }

    pub fn get_value(&self) -> String {
        self.value.clone()
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TokenType {
    String,
    Integer,
    Symbol,
    Identifier,
    Keyword,
    None,
}

fn process_code(code: &str) -> Vec<TokenItem> {
    let mut start_token_position: usize = 0;
    let mut current_type = TokenType::None;
    let mut result: Vec<TokenItem> = Vec::new();

    for (i, c) in code.chars().enumerate() {
        if c == '"' {
            match current_type {
                TokenType::None => {
                    start_token_position = i;
                    current_type = TokenType::String;
                }
                TokenType::String => {
                    result.push(build_token(&code[start_token_position..(i + 1)]));
                    start_token_position = i + 1;
                    current_type = TokenType::None;
                    continue;
                }
                _ => panic!(format!(
                    "Invalid presence of \" inside a {:?}",
                    current_type
                )),
            }
        }

        if current_type == TokenType::String {
            continue;
        }

        if c == ' ' {
            if i - start_token_position > 0 {
                result.push(build_token(&code[start_token_position..i]));
            }

            start_token_position = i + 1;
            current_type = TokenType::None;

            continue;
        }

        if is_symbol(c) {
            if i - start_token_position > 0 {
                result.push(build_token(&code[start_token_position..i]));
            }

            result.push(build_token(&c.to_string()));
            start_token_position = i + 1;
            current_type = TokenType::None;

            continue;
        }

        if c.is_numeric() && current_type == TokenType::None {
            start_token_position = i;
            current_type = TokenType::Integer;
        }

        if current_type == TokenType::Integer && !c.is_numeric() {
            panic!("Non numeric char mixed inside a Integer token");
        }

        if current_type == TokenType::None {
            start_token_position = i;
            current_type = TokenType::Identifier;
        }
    }

    if code.len() - start_token_position > 0 {
        result.push(build_token(&code[start_token_position..]));
    }

    result
}

fn build_token(value: &str) -> TokenItem {
    if value.len() == 1 && is_symbol(value.chars().nth(0).unwrap()) {
        return TokenItem::new(value, TokenType::Symbol);
    }

    if is_keyword(value) {
        return TokenItem::new(value, TokenType::Keyword);
    }

    if is_string(value) {
        return TokenItem::new(&value.replace("\"", ""), TokenType::String);
    }

    if is_integer(value) {
        return TokenItem::new(&value.replace("\"", ""), TokenType::Integer);
    }

    TokenItem::new(value, TokenType::Identifier)
}

fn is_symbol(c: char) -> bool {
    let symbols: [char; 19] = [
        '{', '}', '(', ')', '[', ']', '.', ',', ';', '+', '-', '*', '/', '&', '|', '>', '<', '=',
        '~',
    ];

    symbols.contains(&c)
}

fn is_keyword(value: &str) -> bool {
    let keywords: [&str; 21] = [
        "class",
        "constructor",
        "function",
        "method",
        "field",
        "static",
        "var",
        "int",
        "char",
        "boolean",
        "void",
        "true",
        "false",
        "null",
        "this",
        "let",
        "do",
        "if",
        "else",
        "while",
        "return",
    ];

    keywords.contains(&value)
}

fn is_string(value: &str) -> bool {
    if value.starts_with('"') {
        if value.ends_with('"') {
            return true;
        }
        panic!(format!(
            "Incomplete string: '{}' starts with \" but not ends with \"",
            value
        ));
    }
    false
}

fn is_integer(value: &str) -> bool {
    for c in value.chars() {
        if !c.is_numeric() {
            return false;
        }
    }

    // panics on long numeric sequences that are out of i16 range
    let parsed = value.parse::<i16>();
    if parsed.is_err() {
        panic!(format!(
            "Invalid numeric value: {}. Failed to parse to i16",
            value
        ));
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_token_symbol() {
        let token = build_token("(");

        assert_eq!(token.get_type(), TokenType::Symbol);
        assert_eq!(token.get_value(), "(");
    }

    #[test]
    fn test_process_code_symbol() {
        let result = process_code("(");

        assert_eq!(result.len(), 1);

        let token = result.get(0).unwrap();
        assert_eq!(token.get_type(), TokenType::Symbol);
        assert_eq!(token.get_value(), "(");
    }

    #[test]
    fn test_process_code_identifier_and_symbol() {
        let result = process_code("test(");

        assert_eq!(result.len(), 2);

        let token = result.get(0).unwrap();
        assert_eq!(token.get_type(), TokenType::Identifier);
        assert_eq!(token.get_value(), "test");

        let token = result.get(1).unwrap();
        assert_eq!(token.get_type(), TokenType::Symbol);
        assert_eq!(token.get_value(), "(");
    }

    #[test]
    fn test_process_code_keyword() {
        let result = process_code("class");

        assert_eq!(result.len(), 1);

        let token = result.get(0).unwrap();
        assert_eq!(token.get_type(), TokenType::Keyword);
        assert_eq!(token.get_value(), "class");
    }

    #[test]
    fn test_retrieve_type() {
        let mut tokenizer = Tokenizer::new("int x");

        let token = tokenizer.retrieve_type();

        assert_eq!(token, "int");
    }

    #[test]
    #[should_panic(expected = "Invalid presence of \" inside a Identifier")]
    fn test_process_code_invalid_quote() {
        let _ = process_code("test\"");
    }

    #[test]
    #[should_panic(expected = "Incomplete string: '\"test)' starts with \" but not ends with \"")]
    fn test_process_code_with_invalid_string() {
        let _ = process_code("print(\"test)");
    }

    #[test]
    #[should_panic(expected = "Non numeric char mixed inside a Integer token")]
    fn test_process_code_number_with_invalid_char() {
        let _ = process_code("x = 23a");
    }

    #[test]
    #[should_panic(expected = "Invalid numeric value: 32768. Failed to parse to i16")]
    fn test_process_code_number_too_big() {
        let _ = process_code("x = 32768");
    }

    #[test]
    #[should_panic(
        expected = "Invalid keywork. Expected [\"int\", \"char\", \"boolean\"], but found void"
    )]
    fn test_retrieve_invalid_type() {
        let mut tokenizer = Tokenizer::new("void x");

        let _ = tokenizer.retrieve_type();
    }

    #[test]
    fn test_process_code_call_method_with_string() {
        let result = process_code("print(\"big string\")");

        assert_eq!(result.len(), 4);

        let token = result.get(0).unwrap();
        assert_eq!(token.get_type(), TokenType::Identifier);
        assert_eq!(token.get_value(), "print");

        let token = result.get(1).unwrap();
        assert_eq!(token.get_type(), TokenType::Symbol);
        assert_eq!(token.get_value(), "(");

        let token = result.get(2).unwrap();
        assert_eq!(token.get_type(), TokenType::String);
        assert_eq!(token.get_value(), "big string");

        let token = result.get(3).unwrap();
        assert_eq!(token.get_type(), TokenType::Symbol);
        assert_eq!(token.get_value(), ")");
    }

    #[test]
    fn test_process_code_sum_two_numbers() {
        let result = process_code("5 +   7");

        assert_eq!(result.len(), 3);

        let token = result.get(0).unwrap();
        assert_eq!(token.get_type(), TokenType::Integer);
        assert_eq!(token.get_value(), "5");

        let token = result.get(1).unwrap();
        assert_eq!(token.get_type(), TokenType::Symbol);
        assert_eq!(token.get_value(), "+");

        let token = result.get(2).unwrap();
        assert_eq!(token.get_type(), TokenType::Integer);
        assert_eq!(token.get_value(), "7");
    }
    #[test]
    fn test_process_code_long_command() {
        let result = process_code("do    Output.printInt(   sum / length  );");

        assert_eq!(result.len(), 10);

        let token = result.get(0).unwrap();
        assert_eq!(token.get_type(), TokenType::Keyword);
        assert_eq!(token.get_value(), "do");

        let token = result.get(1).unwrap();
        assert_eq!(token.get_type(), TokenType::Identifier);
        assert_eq!(token.get_value(), "Output");

        let token = result.get(2).unwrap();
        assert_eq!(token.get_type(), TokenType::Symbol);
        assert_eq!(token.get_value(), ".");

        let token = result.get(3).unwrap();
        assert_eq!(token.get_type(), TokenType::Identifier);
        assert_eq!(token.get_value(), "printInt");

        let token = result.get(4).unwrap();
        assert_eq!(token.get_type(), TokenType::Symbol);
        assert_eq!(token.get_value(), "(");

        let token = result.get(5).unwrap();
        assert_eq!(token.get_type(), TokenType::Identifier);
        assert_eq!(token.get_value(), "sum");

        let token = result.get(6).unwrap();
        assert_eq!(token.get_type(), TokenType::Symbol);
        assert_eq!(token.get_value(), "/");

        let token = result.get(7).unwrap();
        assert_eq!(token.get_type(), TokenType::Identifier);
        assert_eq!(token.get_value(), "length");

        let token = result.get(8).unwrap();
        assert_eq!(token.get_type(), TokenType::Symbol);
        assert_eq!(token.get_value(), ")");

        let token = result.get(9).unwrap();
        assert_eq!(token.get_type(), TokenType::Symbol);
        assert_eq!(token.get_value(), ";");
    }
}
