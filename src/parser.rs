use std::collections::HashMap;

use crate::tokenizer::{TokenItem, TokenType, Tokenizer, UNARY_OP_SYMBOLS};

pub struct TokenTreeItem {
    name: Option<String>,
    item: Option<TokenItem>,
    nodes: Vec<TokenTreeItem>,
    symbol_table: Option<SymbolTable>,
}

impl TokenTreeItem {
    pub fn new_root(name: &str) -> TokenTreeItem {
        TokenTreeItem {
            name: Some(String::from(name)),
            item: None,
            nodes: Vec::new(),
            symbol_table: None,
        }
    }

    pub fn new(token: TokenItem) -> TokenTreeItem {
        TokenTreeItem {
            name: None,
            item: Some(token),
            nodes: Vec::new(),
            symbol_table: None,
        }
    }

    pub fn push(&mut self, item: TokenItem) {
        self.nodes.push(TokenTreeItem::new(item));
    }

    pub fn set_symbol_table(&mut self, symbol_table: SymbolTable) {
        self.symbol_table.replace(symbol_table);
    }

    pub fn push_item(&mut self, item: TokenTreeItem) {
        self.nodes.push(item);
    }

    pub fn get_name(&self) -> &Option<String> {
        &self.name
    }

    pub fn get_item(&self) -> &Option<TokenItem> {
        &self.item
    }

    pub fn get_nodes(&self) -> &Vec<TokenTreeItem> {
        &self.nodes
    }

    pub fn get_symbol_table(&self) -> &Option<SymbolTable> {
        &self.symbol_table
    }
}

#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
enum SymbolType {
    Field,
    StaticType,
    Local,
    Argument,
}

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
struct SymbolItem {
    id: usize,
    name: String,
    symbol_type: SymbolType,
    kind: String,
    position: usize,
}

impl SymbolItem {
    fn new(
        id: usize,
        name: String,
        symbol_type: SymbolType,
        kind: String,
        position: usize,
    ) -> SymbolItem {
        SymbolItem {
            id,
            name,
            symbol_type,
            kind,
            position,
        }
    }

    pub fn get_type_as_str(&self) -> String {
        let result = match self.symbol_type {
            SymbolType::Argument => "argument",
            SymbolType::Field => "field",
            SymbolType::Local => "local",
            SymbolType::StaticType => "static",
        };

        String::from(result)
    }

    pub fn get_position(&self) -> usize {
        self.position
    }
}

pub struct SymbolTable {
    symbols: Vec<SymbolItem>,
    indexes: HashMap<String, usize>,
    types: HashMap<SymbolType, usize>,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        let mut types = HashMap::new();

        types.insert(SymbolType::Field, 0 as usize);
        types.insert(SymbolType::StaticType, 0 as usize);
        types.insert(SymbolType::Local, 0 as usize);
        types.insert(SymbolType::Argument, 0 as usize);

        SymbolTable {
            symbols: Vec::new(),
            indexes: HashMap::new(),
            types,
        }
    }

    fn clone(&self) -> SymbolTable {
        SymbolTable {
            symbols: Vec::from(self.symbols.clone()),
            indexes: HashMap::from(self.indexes.clone()),
            types: HashMap::from(self.types.clone()),
        }
    }

    pub fn add(&mut self, symbol_type: &str, kind: &str, name: &str) {
        let symbol_type = match symbol_type {
            "field" => SymbolType::Field,
            "static" => SymbolType::StaticType,
            "var" => SymbolType::Local,
            "argument" => SymbolType::Argument,
            v => panic!(format!("Invalid symbol type: {}", v)),
        };

        if self.indexes.contains_key(name) {
            panic!(format!("Symbol already exists on symbol table: {}", name));
        }

        let position = self.types.get(&symbol_type).unwrap().clone();
        *self.types.entry(symbol_type).or_insert(1) += 1;

        let id = self.symbols.len();
        self.symbols.push(SymbolItem::new(
            id,
            String::from(name),
            symbol_type,
            String::from(kind),
            position,
        ));

        self.indexes.insert(String::from(name), id);
    }

    pub fn get(&self, name: &str) -> &SymbolItem {
        let index = self
            .indexes
            .get(name)
            .expect("Name nof found on indexes")
            .clone();
        self.symbols.get(index).unwrap()
    }

    pub fn get_pop(&self, name: &str) -> String {
        let symbol = self.get(name);
        format!("pop {} {}", symbol.get_type_as_str(), symbol.get_position())
    }

    pub fn get_push(&self, name: &str) -> String {
        let symbol = self.get(name);
        format!(
            "push {} {}",
            symbol.get_type_as_str(),
            symbol.get_position()
        )
    }
}

pub struct ClassNode {}

impl ClassNode {
    pub fn build(tokenizer: &mut Tokenizer) -> TokenTreeItem {
        let mut root = TokenTreeItem::new_root("class");
        let mut symbol_table = SymbolTable::new();

        tokenizer.reset();

        root.push(tokenizer.consume("class"));

        root.push(tokenizer.retrieve_identifier());

        root.push(tokenizer.consume("{"));

        for var_dec in VarDec::build_class(tokenizer, &mut symbol_table) {
            root.push_item(var_dec);
        }

        for subroutine in SubroutineDec::build(tokenizer, &symbol_table) {
            root.push_item(subroutine);
        }

        root.push(tokenizer.consume("}"));

        root
    }
}

pub struct VarDec {}

impl VarDec {
    pub fn build_class(
        tokenizer: &mut Tokenizer,
        symbol_table: &mut SymbolTable,
    ) -> Vec<TokenTreeItem> {
        let mut result = Vec::new();

        while let Some(current_token) = tokenizer.peek_next() {
            match current_token.get_value().as_str() {
                "field" => result.push(VarDec::build_field(
                    tokenizer,
                    "classVarDec",
                    "field",
                    symbol_table,
                )),
                "static" => result.push(VarDec::build_field(
                    tokenizer,
                    "classVarDec",
                    "static",
                    symbol_table,
                )),
                _ => break,
            }
        }

        result
    }

    pub fn build_var(
        tokenizer: &mut Tokenizer,
        symbol_table: &mut SymbolTable,
    ) -> Vec<TokenTreeItem> {
        let mut result = Vec::new();

        while let Some(current_token) = tokenizer.peek_next() {
            match current_token.get_value().as_str() {
                "var" => result.push(VarDec::build_field(
                    tokenizer,
                    "varDec",
                    "var",
                    symbol_table,
                )),
                _ => break,
            }
        }

        result
    }

    fn build_field(
        tokenizer: &mut Tokenizer,
        name: &str,
        descriptor: &str,
        symbol_table: &mut SymbolTable,
    ) -> TokenTreeItem {
        let mut root = TokenTreeItem::new_root(name);

        root.push(tokenizer.consume(descriptor));

        let field_type = tokenizer.retrieve_type();
        let kind = field_type.get_value();

        let identifier = tokenizer.retrieve_identifier();

        symbol_table.add(descriptor, kind.as_str(), identifier.get_value().as_str());

        root.push(field_type);
        root.push(identifier);

        while let Some(token) = tokenizer.get_next() {
            match token.get_value().as_str() {
                "," => {
                    root.push(token.clone());

                    let identifier = tokenizer.retrieve_identifier();

                    symbol_table.add(descriptor, kind.as_str(), identifier.get_value().as_str());

                    root.push(identifier);
                }
                ";" => {
                    root.push(token.clone());
                    break;
                }
                value => panic!(format!("Expecting ',' or ';', but retrieved '{}'", value)),
            }
        }

        root
    }
}

struct SubroutineDec {}

impl SubroutineDec {
    pub fn build(tokenizer: &mut Tokenizer, symbol_table: &SymbolTable) -> Vec<TokenTreeItem> {
        let mut result = Vec::new();

        while let Some(next_token) = tokenizer.peek_next() {
            if next_token.get_value() == "}" {
                break;
            }

            result.push(SubroutineDec::build_subroutine(tokenizer, &symbol_table));
        }

        result
    }

    pub fn build_subroutine(
        tokenizer: &mut Tokenizer,
        symbol_table: &SymbolTable,
    ) -> TokenTreeItem {
        let mut root = TokenTreeItem::new_root("subroutineDec");
        let mut symbol_table = symbol_table.clone();

        root.push(tokenizer.retrieve_keyword());
        root.push(tokenizer.retrieve_any(Vec::from([TokenType::Keyword, TokenType::Identifier])));
        root.push(tokenizer.retrieve_identifier());
        root.push(tokenizer.consume("("));

        root.push_item(SubroutineDec::build_parameters(
            tokenizer,
            &mut symbol_table,
        ));

        root.push(tokenizer.consume(")"));

        root.push_item(SubroutineDec::build_body(tokenizer, &mut symbol_table));

        root.set_symbol_table(symbol_table);

        root
    }

    fn build_body(tokenizer: &mut Tokenizer, symbol_table: &mut SymbolTable) -> TokenTreeItem {
        let mut root = TokenTreeItem::new_root("subroutineBody");

        root.push(tokenizer.consume("{"));

        for var_dec in VarDec::build_var(tokenizer, symbol_table) {
            root.push_item(var_dec);
        }

        root.push_item(Statement::build_list(tokenizer));

        root.push(tokenizer.consume("}"));

        root
    }

    fn build_parameters(
        tokenizer: &mut Tokenizer,
        symbol_table: &mut SymbolTable,
    ) -> TokenTreeItem {
        let mut root = TokenTreeItem::new_root("parameterList");

        while let Some(next_token) = tokenizer.peek_next() {
            if next_token.get_value() == ")" {
                break;
            }

            if next_token.get_value() == "," {
                root.push(tokenizer.consume(","));
            }

            let parameter_type = tokenizer.retrieve_type();
            let identifier = tokenizer.retrieve_identifier();

            symbol_table.add(
                "argument",
                parameter_type.get_value().as_str(),
                identifier.get_value().as_str(),
            );

            root.push(parameter_type);
            root.push(identifier);
        }

        root
    }
}

pub struct Statement {}

impl Statement {
    pub fn build_list(tokenizer: &mut Tokenizer) -> TokenTreeItem {
        let mut root = TokenTreeItem::new_root("statements");

        while let Some(next_token) = tokenizer.peek_next() {
            if next_token.get_value() == "}" {
                break;
            }

            root.push_item(Statement::build(tokenizer));
        }

        root
    }

    pub fn build(tokenizer: &mut Tokenizer) -> TokenTreeItem {
        let next_token = tokenizer.peek_next().unwrap();

        if next_token.get_type() != TokenType::Keyword {
            panic!(format!(
                "Invalid token type on build of statement: {:?} ({})",
                next_token.get_type(),
                next_token.get_value()
            ));
        }

        match next_token.get_value().as_str() {
            "return" => Statement::build_return(tokenizer),
            "do" => Statement::build_do(tokenizer),
            "while" => Statement::build_while(tokenizer),
            "if" => Statement::build_if(tokenizer),
            "let" => Statement::build_let(tokenizer),
            value => panic!(format!("Invalid statement value: {}", value)),
        }
    }

    pub fn build_return(tokenizer: &mut Tokenizer) -> TokenTreeItem {
        let mut root = TokenTreeItem::new_root("returnStatement");

        root.push(tokenizer.consume("return"));

        let next_token = tokenizer.peek_next().unwrap();

        if next_token.get_value() == ";" {
            root.push(tokenizer.consume(";"));
            return root;
        }

        root.push_item(Expression::build(tokenizer));
        root.push(tokenizer.consume(";"));

        root
    }

    pub fn build_do(tokenizer: &mut Tokenizer) -> TokenTreeItem {
        let mut root = TokenTreeItem::new_root("doStatement");

        root.push(tokenizer.consume("do"));

        root.push(tokenizer.retrieve_identifier());
        SubroutineCall::build(&mut root, tokenizer);

        root.push(tokenizer.consume(";"));

        root
    }

    pub fn build_while(tokenizer: &mut Tokenizer) -> TokenTreeItem {
        let mut root = TokenTreeItem::new_root("whileStatement");

        root.push(tokenizer.consume("while"));
        root.push(tokenizer.consume("("));
        root.push_item(Expression::build(tokenizer));
        root.push(tokenizer.consume(")"));
        root.push(tokenizer.consume("{"));
        root.push_item(Statement::build_list(tokenizer));
        root.push(tokenizer.consume("}"));

        root
    }

    pub fn build_if(tokenizer: &mut Tokenizer) -> TokenTreeItem {
        let mut root = TokenTreeItem::new_root("ifStatement");

        root.push(tokenizer.consume("if"));
        root.push(tokenizer.consume("("));
        root.push_item(Expression::build(tokenizer));
        root.push(tokenizer.consume(")"));
        root.push(tokenizer.consume("{"));
        root.push_item(Statement::build_list(tokenizer));
        root.push(tokenizer.consume("}"));

        let next_token = tokenizer.peek_next();

        if next_token.is_none() {
            return root;
        }

        let next_token = next_token.unwrap();

        if next_token.get_value() == "else" {
            root.push(tokenizer.consume("else"));
            root.push(tokenizer.consume("{"));
            root.push_item(Statement::build_list(tokenizer));
            root.push(tokenizer.consume("}"));

            return root;
        }

        root
    }

    pub fn build_let(tokenizer: &mut Tokenizer) -> TokenTreeItem {
        let mut root = TokenTreeItem::new_root("letStatement");

        root.push(tokenizer.consume("let"));
        root.push(tokenizer.retrieve_identifier());

        let next_token = tokenizer.peek_next().unwrap();

        if next_token.get_value() == "[" {
            root.push(tokenizer.consume("["));
            root.push_item(Expression::build(tokenizer));
            root.push(tokenizer.consume("]"));
        }

        root.push(tokenizer.consume("="));
        root.push_item(Expression::build(tokenizer));
        root.push(tokenizer.consume(";"));

        root
    }
}

pub struct Expression {}

impl Expression {
    pub fn build(tokenizer: &mut Tokenizer) -> TokenTreeItem {
        let mut root = TokenTreeItem::new_root("expression");

        root.push_item(Term::build(tokenizer));

        while let Some(next_token) = tokenizer.peek_next() {
            if !next_token.is_op() {
                break;
            }

            root.push(tokenizer.retrieve_op());
            root.push_item(Term::build(tokenizer));
        }

        root
    }
}

struct SubroutineCall {}

impl SubroutineCall {
    pub fn build(root: &mut TokenTreeItem, tokenizer: &mut Tokenizer) {
        let next_token = tokenizer.peek_next().unwrap();

        if next_token.get_type() == TokenType::Symbol && next_token.get_value() == "(" {
            root.push(tokenizer.consume("("));
            root.push_item(SubroutineCall::build_expression_list(tokenizer));
            root.push(tokenizer.consume(")"));

            return;
        }

        if next_token.get_type() == TokenType::Symbol && next_token.get_value() == "." {
            root.push(tokenizer.consume("."));
            root.push(tokenizer.retrieve_identifier());

            root.push(tokenizer.consume("("));
            root.push_item(SubroutineCall::build_expression_list(tokenizer));
            root.push(tokenizer.consume(")"));

            return;
        }

        panic!("Invalid next token on building subroutine call");
    }

    fn build_expression_list(tokenizer: &mut Tokenizer) -> TokenTreeItem {
        let mut root = TokenTreeItem::new_root("expressionList");

        let next_token = tokenizer.peek_next();

        if next_token.is_none()
            || next_token.unwrap().get_value() == ")"
            || next_token.unwrap().get_value() == "]"
        {
            return root;
        }

        root.push_item(Expression::build(tokenizer));

        while let Some(next_token) = tokenizer.peek_next() {
            if next_token.get_type() != TokenType::Symbol || next_token.get_value() != "," {
                break;
            }

            root.push(tokenizer.consume(","));
            root.push_item(Expression::build(tokenizer));
        }

        root
    }
}

struct Term {}

impl Term {
    pub fn build(tokenizer: &mut Tokenizer) -> TokenTreeItem {
        let mut root = TokenTreeItem::new_root("term");

        let token = tokenizer.get_next().unwrap();
        root.push(token.clone());

        match token.get_type() {
            TokenType::Identifier => Term::build_identifier(&mut root, tokenizer),
            TokenType::Symbol => {
                Term::build_symbol(token.get_value().as_str(), &mut root, tokenizer)
            }
            _ => (),
        };

        root
    }

    fn build_identifier(root: &mut TokenTreeItem, tokenizer: &mut Tokenizer) {
        let next_token = tokenizer.peek_next();

        if next_token.is_none() {
            return;
        }

        let next_token = next_token.unwrap();

        if next_token.get_value() == "[" {
            root.push(tokenizer.consume("["));
            root.push_item(Expression::build(tokenizer));
            root.push(tokenizer.consume("]"));

            return;
        }

        if [".", "("].contains(&next_token.get_value().as_str()) {
            SubroutineCall::build(root, tokenizer);
        }
    }

    fn build_symbol(value: &str, root: &mut TokenTreeItem, tokenizer: &mut Tokenizer) {
        if value == "(" {
            root.push_item(Expression::build(tokenizer));
            root.push(tokenizer.consume(")"));

            return;
        }

        if UNARY_OP_SYMBOLS.contains(&value) {
            root.push_item(Term::build(tokenizer));

            return;
        }

        panic!("Invalid symbol list inside an symbol call");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_root_node() {
        let mut tokenizer = Tokenizer::new("class Test {}");

        let result = ClassNode::build(&mut tokenizer);

        let name = result.get_name().as_ref();
        assert!(&name.is_some());
        assert_eq!(name.unwrap().as_str(), "class");
    }

    #[test]
    fn build_class_var_dec_list() {
        let mut tokenizer = Tokenizer::new("field int x, y; static String name;");
        let mut symbol_table = SymbolTable::new();

        let result = VarDec::build_class(&mut tokenizer, &mut symbol_table);

        assert_eq!(symbol_table.symbols.len(), 3);

        let symbol = symbol_table.symbols.get(0).unwrap();
        assert_eq!(symbol.name, "x");
        assert_eq!(symbol.symbol_type, SymbolType::Field);
        assert_eq!(symbol.kind, "int");
        assert_eq!(symbol.position, 0);

        let symbol = symbol_table.symbols.get(1).unwrap();
        assert_eq!(symbol.name, "y");
        assert_eq!(symbol.symbol_type, SymbolType::Field);
        assert_eq!(symbol.kind, "int");
        assert_eq!(symbol.position, 1);

        let symbol = symbol_table.symbols.get(2).unwrap();
        assert_eq!(symbol.name, "name");
        assert_eq!(symbol.symbol_type, SymbolType::StaticType);
        assert_eq!(symbol.kind, "String");
        assert_eq!(symbol.position, 0);

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn build_subroutine_with_argumants_and_vars() {
        let mut tokenizer =
            Tokenizer::new("method void test(int x, String name) {var boolean a, b;}");
        let symbol_table = SymbolTable::new();

        let result = SubroutineDec::build_subroutine(&mut tokenizer, &symbol_table);
        let symbol_table = result.get_symbol_table().as_ref().unwrap();

        assert_eq!(symbol_table.symbols.len(), 4);

        let symbol = symbol_table.symbols.get(0).unwrap();
        assert_eq!(symbol.name, "x");
        assert_eq!(symbol.symbol_type, SymbolType::Argument);
        assert_eq!(symbol.kind, "int");
        assert_eq!(symbol.position, 0);

        let symbol = symbol_table.symbols.get(1).unwrap();
        assert_eq!(symbol.name, "name");
        assert_eq!(symbol.symbol_type, SymbolType::Argument);
        assert_eq!(symbol.kind, "String");
        assert_eq!(symbol.position, 1);

        let symbol = symbol_table.symbols.get(2).unwrap();
        assert_eq!(symbol.name, "a");
        assert_eq!(symbol.symbol_type, SymbolType::Local);
        assert_eq!(symbol.kind, "boolean");
        assert_eq!(symbol.position, 0);

        let symbol = symbol_table.symbols.get(3).unwrap();
        assert_eq!(symbol.name, "b");
        assert_eq!(symbol.symbol_type, SymbolType::Local);
        assert_eq!(symbol.kind, "boolean");
        assert_eq!(symbol.position, 1);

        assert_eq!(result.nodes.len(), 7);
        let identifier = result.nodes.get(2).unwrap();
        assert_eq!(identifier.get_item().as_ref().unwrap().get_value(), "test");
    }

    #[test]
    fn build_list_of_subroutines() {
        let mut tokenizer =
            Tokenizer::new("method void print(int x) {} function int count(String name) {}");
        let symbol_table = SymbolTable::new();

        let result = SubroutineDec::build(&mut tokenizer, &symbol_table);

        assert_eq!(result.len(), 2);

        let subroutine = result.get(0).unwrap();
        let identifier = subroutine.nodes.get(2).unwrap();
        assert_eq!(identifier.get_item().as_ref().unwrap().get_value(), "print");

        let subroutine = result.get(1).unwrap();
        let identifier = subroutine.nodes.get(2).unwrap();
        assert_eq!(identifier.get_item().as_ref().unwrap().get_value(), "count");
    }

    //     #[test]
    //     fn build_term_integer() {
    //         let mut tokenizer = Tokenizer::new("123");

    //         let result = Term::build(&mut tokenizer);

    //         assert_eq!(result.get_type(), &TermType::Integer);
    //         assert_eq!(result.get_value(), "123");
    //     }

    //     #[test]
    //     fn build_term_string() {
    //         let mut tokenizer = Tokenizer::new("\"test string\"");

    //         let result = Term::build(&mut tokenizer);

    //         assert_eq!(result.get_type(), &TermType::String);
    //         assert_eq!(result.get_value(), "test string");
    //     }

    //     #[test]
    //     fn build_term_keyword() {
    //         let mut tokenizer = Tokenizer::new("this");

    //         let result = Term::build(&mut tokenizer);

    //         assert_eq!(result.get_type(), &TermType::Keyword);
    //         assert_eq!(result.get_value(), "this");
    //     }

    //     #[test]
    //     fn build_term_var_name() {
    //         let mut tokenizer = Tokenizer::new("color");

    //         let result = Term::build(&mut tokenizer);

    //         assert_eq!(result.get_type(), &TermType::VarName);
    //         assert_eq!(result.get_value(), "color");
    //     }

    //     #[test]
    //     fn build_term_array() {
    //         let mut tokenizer = Tokenizer::new("position[10]");

    //         let result = Term::build(&mut tokenizer);

    //         assert_eq!(result.get_type(), &TermType::ArrayCall);
    //         assert_eq!(result.get_value(), "position");

    //         let expression = result.get_expression().as_ref().unwrap();
    //         assert_eq!(expression.get_term().get_value(), "10");
    //     }

    //     #[test]
    //     fn build_term_subroutine() {
    //         let mut tokenizer = Tokenizer::new("print(\"my name\", 10)");

    //         let result = Term::build(&mut tokenizer);

    //         assert_eq!(result.get_type(), &TermType::SubroutineCall);
    //         assert_eq!(result.get_value(), "");

    //         let subroutine = result.get_subroutine().as_ref().unwrap();
    //         assert_eq!(subroutine.get_value(), "print");

    //         let expression = subroutine.get_expressions().get(0).unwrap();
    //         assert_eq!(expression.get_term().get_value(), "my name");

    //         let expression = subroutine.get_expressions().get(1).unwrap();
    //         assert_eq!(expression.get_term().get_value(), "10");
    //     }

    //     #[test]
    //     fn build_term_subroutine_with_class() {
    //         let mut tokenizer = Tokenizer::new("Console.write()");

    //         let result = Term::build(&mut tokenizer);

    //         assert_eq!(result.get_type(), &TermType::SubroutineCall);
    //         assert_eq!(result.get_value(), "");

    //         let subroutine = result.get_subroutine().as_ref().unwrap();
    //         assert_eq!(subroutine.get_value(), "write");

    //         let class_name = subroutine.get_class_name().clone();
    //         assert_eq!(class_name.unwrap(), "Console");

    //         assert_eq!(subroutine.get_expressions().len(), 0);
    //     }

    //     #[test]
    //     fn build_symbol_with_expression() {
    //         let mut tokenizer = Tokenizer::new("(x + 2)");

    //         let result = Term::build(&mut tokenizer);

    //         assert_eq!(result.get_type(), &TermType::Expression);

    //         let expression = result.get_expression().as_ref().unwrap();

    //         assert_eq!(expression.get_term().get_value(), "x");

    //         let op = expression.get_op().clone();
    //         assert_eq!(op.unwrap(), "+");

    //         let another_term = expression.get_another_term().as_ref();
    //         assert_eq!(another_term.unwrap().get_value(), "2");
    //     }

    //     #[test]
    //     fn build_symbol_with_unary() {
    //         let mut tokenizer = Tokenizer::new("-x");

    //         let result = Term::build(&mut tokenizer);

    //         assert_eq!(result.get_type(), &TermType::VarName);
    //         assert_eq!(result.get_value(), "x");
    //         assert_eq!(result.get_unary_op().as_ref().unwrap(), "-");
    //     }

    //     #[test]
    //     fn build_statement_list_return_expression() {
    //         let mut tokenizer = Tokenizer::new("return name;");

    //         let statements = Statement::build_list(&mut tokenizer);

    //         assert_eq!(statements.len(), 1);

    //         let return_statement = statements.get(0).unwrap();

    //         assert_eq!(return_statement.get_type(), &StatementType::Return);

    //         let expression = return_statement
    //             .get_return()
    //             .as_ref()
    //             .unwrap()
    //             .get_expression()
    //             .as_ref()
    //             .unwrap();
    //         assert_eq!(expression.get_term().get_value(), "name");
    //     }

    //     #[test]
    //     fn build_statement_list_return() {
    //         let mut tokenizer = Tokenizer::new("return;");

    //         let statements = Statement::build_list(&mut tokenizer);

    //         assert_eq!(statements.len(), 1);

    //         let return_statement = statements.get(0).unwrap();

    //         assert_eq!(return_statement.get_type(), &StatementType::Return);

    //         let expression = return_statement
    //             .get_return()
    //             .as_ref()
    //             .unwrap()
    //             .get_expression()
    //             .as_ref();
    //         assert!(expression.is_none());
    //     }

    //     #[test]
    //     fn build_statement_list_do() {
    //         let mut tokenizer = Tokenizer::new("do Console.print(test);");

    //         let statements = Statement::build_list(&mut tokenizer);

    //         assert_eq!(statements.len(), 1);

    //         let do_statement = statements.get(0).unwrap();

    //         assert_eq!(do_statement.get_type(), &StatementType::Do);

    //         let subroutine = do_statement.get_do().as_ref().unwrap().get_subroutine();
    //         assert_eq!(subroutine.get_class_name().as_ref().unwrap(), "Console");
    //         assert_eq!(subroutine.get_value(), "print");
    //     }

    //     #[test]
    //     fn build_statement_list_while() {
    //         let mut tokenizer = Tokenizer::new("while (x < 5) { do Console.print(test); }");

    //         let statements = Statement::build_list(&mut tokenizer);

    //         assert_eq!(statements.len(), 1);

    //         let statement = statements.get(0).unwrap();

    //         assert_eq!(statement.get_type(), &StatementType::While);

    //         let while_statement = statement.get_while().as_ref().unwrap();

    //         let expression = while_statement.get_expression();
    //         assert_eq!(expression.get_term().get_value(), "x");

    //         let statements = while_statement.get_statements();
    //         assert_eq!(statements.len(), 1);
    //     }

    //     #[test]
    //     fn build_statement_list_if() {
    //         let mut tokenizer = Tokenizer::new("if (x < 5) { return 10; }");

    //         let statements = Statement::build_list(&mut tokenizer);

    //         assert_eq!(statements.len(), 1);

    //         let statement = statements.get(0).unwrap();

    //         assert_eq!(statement.get_type(), &StatementType::If);

    //         let if_statement = statement.get_if().as_ref().unwrap();
    //         assert!(if_statement.get_else_statements().is_none());

    //         let expression = if_statement.get_expression();
    //         assert_eq!(expression.get_term().get_value(), "x");

    //         let statements = if_statement.get_statements();
    //         assert_eq!(statements.len(), 1);
    //     }

    //     #[test]
    //     fn build_statement_list_if_else() {
    //         let mut tokenizer = Tokenizer::new("if (x < 5) { return 10; } else { return 20; }");

    //         let statements = Statement::build_list(&mut tokenizer);

    //         assert_eq!(statements.len(), 1);

    //         let statement = statements.get(0).unwrap();

    //         assert_eq!(statement.get_type(), &StatementType::If);

    //         let if_statement = statement.get_if().as_ref().unwrap();
    //         assert!(if_statement.get_else_statements().is_some());

    //         let expression = if_statement.get_expression();
    //         assert_eq!(expression.get_term().get_value(), "x");

    //         let statements = if_statement.get_statements();
    //         assert_eq!(statements.len(), 1);

    //         let statements = if_statement.get_else_statements().as_ref().unwrap();
    //         assert_eq!(statements.len(), 1);
    //     }

    //     #[test]
    //     fn build_statement_list_let() {
    //         let mut tokenizer = Tokenizer::new("let x = 25;");

    //         let statements = Statement::build_list(&mut tokenizer);

    //         assert_eq!(statements.len(), 1);

    //         let statement = statements.get(0).unwrap();

    //         assert_eq!(statement.get_type(), &StatementType::Let);

    //         let let_statement = statement.get_let().as_ref().unwrap();
    //         assert_eq!(let_statement.get_var_name(), "x");
    //         assert!(let_statement.get_array_expression().is_none());

    //         let expression = let_statement.get_expression();
    //         assert_eq!(expression.get_term().get_value(), "25");
    //     }

    //     #[test]
    //     fn build_statement_list_let_array() {
    //         let mut tokenizer = Tokenizer::new("let names[10] = \"test\";");

    //         let statements = Statement::build_list(&mut tokenizer);

    //         assert_eq!(statements.len(), 1);

    //         let statement = statements.get(0).unwrap();

    //         assert_eq!(statement.get_type(), &StatementType::Let);

    //         let let_statement = statement.get_let().as_ref().unwrap();
    //         assert_eq!(let_statement.get_var_name(), "names");
    //         assert!(let_statement.get_array_expression().is_some());

    //         let array_expression = let_statement.get_array_expression().as_ref().unwrap();
    //         assert_eq!(array_expression.get_term().get_value(), "10");

    //         let expression = let_statement.get_expression();
    //         assert_eq!(expression.get_term().get_value(), "test");
    //     }

    //     #[test]
    //     fn build_subroutine_dec_list_string_function() {
    //         let mut tokenizer = Tokenizer::new("function String print() {}");

    //         let result = SubroutineDec::build(&mut tokenizer);

    //         assert_eq!(result.len(), 1);

    //         let subroutine = result.get(0).unwrap();
    //         assert_eq!(subroutine.get_descriptor(), "function");
    //         assert_eq!(subroutine.get_type(), "String");
    //         assert_eq!(subroutine.get_name(), "print");
    //         assert_eq!(subroutine.get_parameters().len(), 0);
    //     }

    //     #[test]
    //     fn build_subroutine_dec_list_multiple_items() {
    //         let mut tokenizer = Tokenizer::new("method void test() {} function String print() {}");

    //         let result = SubroutineDec::build(&mut tokenizer);

    //         assert_eq!(result.len(), 2);
    //     }

    //     #[test]
    //     fn build_subroutine_dec_list_void_method() {
    //         let mut tokenizer = Tokenizer::new("method void test(int x, String name) {var int y; let y = x + 1; do print(y, name); return;}");

    //         let result = SubroutineDec::build(&mut tokenizer);

    //         assert_eq!(result.len(), 1);

    //         let subroutine = result.get(0).unwrap();
    //         assert_eq!(subroutine.get_descriptor(), "method");
    //         assert_eq!(subroutine.get_type(), "void");
    //         assert_eq!(subroutine.get_name(), "test");
    //         assert_eq!(subroutine.get_parameters().len(), 2);
    //     }
}
