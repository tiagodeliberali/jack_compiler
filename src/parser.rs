use crate::tokenizer::{TokenItem, TokenType, Tokenizer, UNARY_OP_SYMBOLS};

pub struct TokenTreeItem {
    pub name: Option<String>,
    pub item: Option<TokenItem>,
    pub nodes: Vec<TokenTreeItem>,
}

impl TokenTreeItem {
    pub fn new_root(name: &str) -> TokenTreeItem {
        TokenTreeItem {
            name: Some(String::from(name)),
            item: None,
            nodes: Vec::new(),
        }
    }

    pub fn new(token: TokenItem) -> TokenTreeItem {
        TokenTreeItem {
            name: None,
            item: Some(token),
            nodes: Vec::new(),
        }
    }

    pub fn push(&mut self, item: TokenItem) {
        self.nodes.push(TokenTreeItem::new(item));
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
}

pub struct ClassNode {}

impl ClassNode {
    pub fn build(tokenizer: &mut Tokenizer) -> TokenTreeItem {
        let mut root = TokenTreeItem::new_root("class");

        tokenizer.reset();

        root.push(tokenizer.consume("class"));

        root.push(tokenizer.retrieve_identifier());

        root.push(tokenizer.consume("{"));

        for var_dec in VarDec::build_class(tokenizer) {
            root.push_item(var_dec);
        }

        for subroutine in SubroutineDec::build(tokenizer) {
            root.push_item(subroutine);
        }

        root.push(tokenizer.consume("}"));

        root
    }
}

pub struct VarDec {}

impl VarDec {
    pub fn build_class(tokenizer: &mut Tokenizer) -> Vec<TokenTreeItem> {
        let mut result = Vec::new();

        while let Some(current_token) = tokenizer.peek_next() {
            match current_token.get_value().as_str() {
                "field" => result.push(VarDec::build_field(tokenizer, "classVarDec", "field")),
                "static" => result.push(VarDec::build_field(tokenizer, "classVarDec", "static")),
                _ => break,
            }
        }

        result
    }

    pub fn build_var(tokenizer: &mut Tokenizer) -> Vec<TokenTreeItem> {
        let mut result = Vec::new();

        while let Some(current_token) = tokenizer.peek_next() {
            match current_token.get_value().as_str() {
                "var" => result.push(VarDec::build_field(tokenizer, "varDec", "var")),
                _ => break,
            }
        }

        result
    }

    fn build_field(tokenizer: &mut Tokenizer, name: &str, descriptor: &str) -> TokenTreeItem {
        let mut root = TokenTreeItem::new_root(name);

        root.push(tokenizer.consume(descriptor));
        root.push(tokenizer.retrieve_type());
        root.push(tokenizer.retrieve_identifier());

        while let Some(token) = tokenizer.get_next() {
            match token.get_value().as_str() {
                "," => {
                    root.push(token.clone());
                    root.push(tokenizer.retrieve_identifier());
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
    pub fn build(tokenizer: &mut Tokenizer) -> Vec<TokenTreeItem> {
        let mut result = Vec::new();

        while let Some(next_token) = tokenizer.peek_next() {
            if next_token.get_value() == "}" {
                break;
            }

            result.push(SubroutineDec::build_subroutine(tokenizer));
        }

        result
    }

    fn build_subroutine(tokenizer: &mut Tokenizer) -> TokenTreeItem {
        let mut root = TokenTreeItem::new_root("subroutineDec");

        root.push(tokenizer.retrieve_keyword());
        root.push(tokenizer.retrieve_any(Vec::from([TokenType::Keyword, TokenType::Identifier])));
        root.push(tokenizer.retrieve_identifier());
        root.push(tokenizer.consume("("));

        root.push_item(SubroutineDec::build_parameters(tokenizer));

        root.push(tokenizer.consume(")"));

        root.push_item(SubroutineDec::build_body(tokenizer));

        root
    }
    fn build_body(tokenizer: &mut Tokenizer) -> TokenTreeItem {
        let mut root = TokenTreeItem::new_root("subroutineBody");

        root.push(tokenizer.consume("{"));

        for var_dec in VarDec::build_var(tokenizer) {
            root.push_item(var_dec);
        }

        root.push_item(Statement::build_list(tokenizer));

        root.push(tokenizer.consume("}"));

        root
    }

    fn build_parameters(tokenizer: &mut Tokenizer) -> TokenTreeItem {
        let mut root = TokenTreeItem::new_root("parameterList");

        while let Some(next_token) = tokenizer.peek_next() {
            if next_token.get_value() == ")" {
                break;
            }

            if next_token.get_value() == "," {
                root.push(tokenizer.consume(","));
            }

            root.push(tokenizer.retrieve_type());
            root.push(tokenizer.retrieve_identifier());
        }

        root
    }
}

struct Statement {}

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

struct Expression {}

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

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn build_root_node() {
//         let mut tokenizer = Tokenizer::new("class Test {}");

//         let result = ClassNode::build(&mut tokenizer);

//         assert_eq!(result.get_identifier(), "Test");
//         assert_eq!(result.get_class_var_dec().len(), 0);
//         assert_eq!(result.get_subroutine_dec().len(), 0);
//     }

//     #[test]
//     fn build_class_var_dec_list() {
//         let mut tokenizer = Tokenizer::new("field int x, y; static String name;");

//         let result = VarDec::build_class(&mut tokenizer);

//         assert_eq!(result.len(), 2);

//         let var = result.get(0).unwrap();
//         assert_eq!(var.get_descriptor(), "field");
//         assert_eq!(var.get_type(), "int");
//         assert_eq!(var.get_names().len(), 2);
//         assert_eq!(var.get_names().get(0).unwrap(), "x");
//         assert_eq!(var.get_names().get(1).unwrap(), "y");

//         let var = result.get(1).unwrap();
//         assert_eq!(var.get_descriptor(), "static");
//         assert_eq!(var.get_type(), "String");
//         assert_eq!(var.get_names().len(), 1);
//         assert_eq!(var.get_names().get(0).unwrap(), "name");
//     }

//     #[test]
//     fn build_class_var_dec_list_without_data() {
//         let mut tokenizer = Tokenizer::new("method void test(x) {}");

//         let result = VarDec::build_class(&mut tokenizer);

//         assert_eq!(result.len(), 0);
//     }

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
// }
