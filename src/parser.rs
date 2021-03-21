use crate::tokenizer::{self, TokenType, Tokenizer, UNARY_OP_SYMBOLS};

pub struct ClassNode {
    identifier: String,
    class_var_dec: Vec<VarDec>,
    subroutine_dec: Vec<SubroutineDec>,
}

impl ClassNode {
    pub fn build(tokenizer: &mut Tokenizer) -> ClassNode {
        tokenizer.reset();

        tokenizer.consume("class");
        let identifier = tokenizer.retrieve_identifier();
        tokenizer.consume("{");
        let class_var_dec = VarDec::build_class(tokenizer);
        let subroutine_dec = SubroutineDec::build(tokenizer);
        tokenizer.consume("}");

        ClassNode {
            identifier,
            class_var_dec,
            subroutine_dec,
        }
    }

    pub fn get_identifier(&self) -> String {
        self.identifier.clone()
    }

    pub fn get_class_var_dec(&self) -> &Vec<VarDec> {
        &self.class_var_dec
    }

    pub fn get_subroutine_dec(&self) -> &Vec<SubroutineDec> {
        &self.subroutine_dec
    }
}

struct VarDec {
    descriptor: String,
    var_type: String,
    names: Vec<String>,
}

impl VarDec {
    pub fn build_class(tokenizer: &mut Tokenizer) -> Vec<VarDec> {
        let mut result = Vec::new();

        while let Some(current_token) = tokenizer.peek_next() {
            match current_token.get_value().as_str() {
                "field" => result.push(VarDec::build_field(tokenizer, "field")),
                "static" => result.push(VarDec::build_field(tokenizer, "static")),
                _ => break,
            }
        }

        result
    }

    pub fn build_var(tokenizer: &mut Tokenizer) -> Vec<VarDec> {
        let mut result = Vec::new();

        while let Some(current_token) = tokenizer.peek_next() {
            match current_token.get_value().as_str() {
                "var" => result.push(VarDec::build_field(tokenizer, "var")),
                _ => break,
            }
        }

        result
    }

    fn build_field(tokenizer: &mut Tokenizer, descriptor: &str) -> VarDec {
        let mut names: Vec<String> = Vec::new();

        tokenizer.consume(descriptor);
        let var_type = tokenizer.retrieve_type();
        let name = tokenizer.retrieve_identifier();
        names.push(name);

        while let Some(token) = tokenizer.get_next() {
            match token.get_value().as_str() {
                "," => names.push(tokenizer.retrieve_identifier()),
                ";" => break,
                value => panic!(format!("Expecting ',' or ';', but retrieved '{}'", value)),
            }
        }

        VarDec {
            descriptor: String::from(descriptor),
            var_type,
            names,
        }
    }

    pub fn get_descriptor(&self) -> &str {
        &self.descriptor
    }

    pub fn get_type(&self) -> &str {
        &self.var_type
    }

    pub fn get_names(&self) -> &Vec<String> {
        &self.names
    }
}

struct SubroutineDec {
    descriptor: String,
    routine_type: String,
    name: String,
    parameters: Vec<String>,
    var_dec: Vec<VarDec>,
    statements: Vec<Box<dyn Statement>>,
}

impl SubroutineDec {
    pub fn build(tokenizer: &mut Tokenizer) -> Vec<SubroutineDec> {
        Vec::new()
    }

    // pub fn build(tokenizer: &mut Tokenizer) -> Vec<SubroutineDec> {
    //     let result = Vec::new();

    //     while let Some(next_token) = tokenizer.peek_next() {
    //         if next_token.get_value() == "}" {
    //             break;
    //         }

    //         result.push(SubroutineDec::build_subroutine(tokenizer));
    //     }

    //     result
    // }

    // fn build_subroutine(tokenizer: &mut Tokenizer) -> SubroutineDec {

    // }
}

fn build_statement_list(tokenizer: &mut Tokenizer) -> Vec<Box<dyn Statement>> {
    Vec::new()
}

trait Statement {}

struct StatementReturn {
    expression: Option<Expression>,
}

impl Statement for StatementReturn {}

impl StatementReturn {
    pub fn build(tokenizer: &mut Tokenizer) {}
}

struct Expression {
    term: Term,
    op: Option<String>,
    other_term: Option<Term>,
}

impl Expression {
    pub fn build(tokenizer: &mut Tokenizer) -> Expression {
        let term = Term::build(tokenizer);

        let next_token = tokenizer.peek_next().unwrap();
        if next_token.is_op() {
            let op = tokenizer.retrieve_op();
            let next_term = Term::build(tokenizer);
            return Expression {
                term,
                op: Option::Some(op),
                other_term: Option::Some(next_term),
            };
        }

        return Expression {
            term,
            op: Option::None,
            other_term: Option::None,
        };
    }

    pub fn get_term(&self) -> &Term {
        &self.term
    }

    pub fn get_op(&self) -> &Option<String> {
        &self.op
    }

    pub fn get_another_term(&self) -> &Option<Term> {
        &self.other_term
    }
}

struct SubroutineCall {
    class_name: Option<String>,
    value: String,
    expressions: Vec<Expression>,
}

impl SubroutineCall {
    pub fn build_from_value(value: String, tokenizer: &mut Tokenizer) -> SubroutineCall {
        let next_token = tokenizer.peek_next().unwrap();

        if next_token.get_type() == TokenType::Symbol && next_token.get_value() == "(" {
            tokenizer.consume("(");
            let expressions = SubroutineCall::build_expression_list(tokenizer);
            tokenizer.consume(")");

            return SubroutineCall {
                class_name: None,
                value,
                expressions,
            };
        }

        if next_token.get_type() == TokenType::Symbol && next_token.get_value() == "." {
            tokenizer.consume(".");
            let var_name = tokenizer.retrieve_identifier();

            tokenizer.consume("(");
            let expressions = SubroutineCall::build_expression_list(tokenizer);
            tokenizer.consume(")");

            return SubroutineCall {
                class_name: Some(value),
                value: var_name,
                expressions,
            };
        }

        panic!("Invalid next token on building subroutine call");
    }

    fn build_expression_list(tokenizer: &mut Tokenizer) -> Vec<Expression> {
        let mut result = Vec::new();

        let next_token = tokenizer.peek_next();

        if next_token.is_none()
            || next_token.unwrap().get_value() == ")"
            || next_token.unwrap().get_value() == "]"
        {
            return result;
        }

        result.push(Expression::build(tokenizer));

        while let Some(next_token) = tokenizer.peek_next() {
            if next_token.get_type() != TokenType::Symbol || next_token.get_value() != "," {
                break;
            }

            tokenizer.consume(",");
            result.push(Expression::build(tokenizer));
        }

        result
    }

    pub fn get_value(&self) -> &String {
        &self.value
    }

    pub fn get_class_name(&self) -> &Option<String> {
        &self.class_name
    }

    pub fn get_expressions(&self) -> &Vec<Expression> {
        &self.expressions
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum TermType {
    Integer,
    String,
    Keyword,
    VarName,
    ArrayCall,
    SubroutineCall,
    Expression,
}

struct Term {
    term_type: TermType,
    value: String,
    expression: Box<Option<Expression>>,
    unary_op: Option<String>,
    another_term: Box<Option<Term>>,
    subroutine_call: Option<SubroutineCall>,
}

impl Term {
    pub fn build(tokenizer: &mut Tokenizer) -> Term {
        let token = tokenizer.get_next().unwrap();

        match token.get_type() {
            TokenType::Integer => Term::new(TermType::Integer, token.get_value()),
            TokenType::String => Term::new(TermType::String, token.get_value()),
            TokenType::Keyword => Term::new(TermType::Keyword, token.get_value()),
            TokenType::Identifier => Term::build_identifier(token.get_value().as_str(), tokenizer),
            TokenType::Symbol => Term::build_symbol(token.get_value().as_str(), tokenizer),
            _ => panic!(format!(
                "Invalid type on expression build: {:?}",
                token.get_type()
            )),
        }
    }

    fn new(term_type: TermType, value: String) -> Term {
        Term {
            term_type,
            value: String::from(value),
            expression: Box::new(Option::None),
            unary_op: None,
            another_term: Box::new(Option::None),
            subroutine_call: None,
        }
    }

    fn new_with_expression(term_type: TermType, value: String, expression: Expression) -> Term {
        Term {
            term_type,
            value: String::from(value),
            expression: Box::new(Some(expression)),
            unary_op: None,
            another_term: Box::new(Option::None),
            subroutine_call: None,
        }
    }

    fn new_with_subroutine(subroutine_call: SubroutineCall) -> Term {
        Term {
            term_type: TermType::SubroutineCall,
            value: String::new(),
            expression: Box::new(Option::None),
            unary_op: None,
            another_term: Box::new(Option::None),
            subroutine_call: Some(subroutine_call),
        }
    }

    fn build_identifier(value: &str, tokenizer: &mut Tokenizer) -> Term {
        let next_token = tokenizer.peek_next();

        if next_token.is_none() {
            return Term::new(TermType::VarName, String::from(value));
        }

        let next_token = next_token.unwrap();

        if next_token.get_type() == TokenType::Symbol && next_token.get_value() == "[" {
            tokenizer.consume("[");
            let expression = Expression::build(tokenizer);
            tokenizer.consume("]");

            return Term::new_with_expression(TermType::ArrayCall, String::from(value), expression);
        }

        if next_token.get_type() == TokenType::Symbol
            && [".", "("].contains(&next_token.get_value().as_str())
        {
            let subroutine = SubroutineCall::build_from_value(String::from(value), tokenizer);
            return Term::new_with_subroutine(subroutine);
        }

        Term::new(TermType::VarName, String::from(value))
    }

    fn build_symbol(value: &str, tokenizer: &mut Tokenizer) -> Term {
        if value == "(" {
            let expression = Expression::build(tokenizer);
            tokenizer.consume(")");

            return Term::new_with_expression(
                TermType::Expression,
                String::from(value),
                expression,
            );
        }

        if UNARY_OP_SYMBOLS.contains(&value) {
            let mut term = Term::build(tokenizer);

            if term.get_unary_op().is_some() {
                panic!("Invalid send try to add an unary op to a term");
            }

            term.set_unary_op(String::from(value));

            return term;
        }

        panic!("Invalid symbol list inside an symbol call");
    }

    pub fn get_type(&self) -> &TermType {
        &self.term_type
    }

    pub fn get_value(&self) -> &str {
        self.value.as_str()
    }

    pub fn get_expression(&self) -> &Option<Expression> {
        &self.expression
    }

    pub fn get_unary_op(&self) -> &Option<String> {
        &self.unary_op
    }

    pub fn set_unary_op(&mut self, value: String) {
        self.unary_op.replace(value);
    }

    pub fn get_subroutine(&self) -> &Option<SubroutineCall> {
        &self.subroutine_call
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_root_node() {
        let mut tokenizer = Tokenizer::new("class Test {}");

        let result = ClassNode::build(&mut tokenizer);

        assert_eq!(result.get_identifier(), "Test");
        assert_eq!(result.get_class_var_dec().len(), 0);
        assert_eq!(result.get_subroutine_dec().len(), 0);
    }

    #[test]
    fn build_class_var_dec_list() {
        let mut tokenizer = Tokenizer::new("field int x, y; static String name;");

        let result = VarDec::build_class(&mut tokenizer);

        assert_eq!(result.len(), 2);

        let var = result.get(0).unwrap();
        assert_eq!(var.get_descriptor(), "field");
        assert_eq!(var.get_type(), "int");
        assert_eq!(var.get_names().len(), 2);
        assert_eq!(var.get_names().get(0).unwrap(), "x");
        assert_eq!(var.get_names().get(1).unwrap(), "y");

        let var = result.get(1).unwrap();
        assert_eq!(var.get_descriptor(), "static");
        assert_eq!(var.get_type(), "String");
        assert_eq!(var.get_names().len(), 1);
        assert_eq!(var.get_names().get(0).unwrap(), "name");
    }

    #[test]
    fn build_class_var_dec_list_without_data() {
        let mut tokenizer = Tokenizer::new("method void test(x) {}");

        let result = VarDec::build_class(&mut tokenizer);

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn build_term_integer() {
        let mut tokenizer = Tokenizer::new("123");

        let result = Term::build(&mut tokenizer);

        assert_eq!(result.get_type(), &TermType::Integer);
        assert_eq!(result.get_value(), "123");
    }

    #[test]
    fn build_term_string() {
        let mut tokenizer = Tokenizer::new("\"test string\"");

        let result = Term::build(&mut tokenizer);

        assert_eq!(result.get_type(), &TermType::String);
        assert_eq!(result.get_value(), "test string");
    }

    #[test]
    fn build_term_keyword() {
        let mut tokenizer = Tokenizer::new("this");

        let result = Term::build(&mut tokenizer);

        assert_eq!(result.get_type(), &TermType::Keyword);
        assert_eq!(result.get_value(), "this");
    }

    #[test]
    fn build_term_var_name() {
        let mut tokenizer = Tokenizer::new("color");

        let result = Term::build(&mut tokenizer);

        assert_eq!(result.get_type(), &TermType::VarName);
        assert_eq!(result.get_value(), "color");
    }

    #[test]
    fn build_term_array() {
        let mut tokenizer = Tokenizer::new("position[10]");

        let result = Term::build(&mut tokenizer);

        assert_eq!(result.get_type(), &TermType::ArrayCall);
        assert_eq!(result.get_value(), "position");

        let expression = result.get_expression().as_ref().unwrap();
        assert_eq!(expression.get_term().get_value(), "10");
    }

    #[test]
    fn build_term_subroutine() {
        let mut tokenizer = Tokenizer::new("print(\"my name\", 10)");

        let result = Term::build(&mut tokenizer);

        assert_eq!(result.get_type(), &TermType::SubroutineCall);
        assert_eq!(result.get_value(), "");

        let subroutine = result.get_subroutine().as_ref().unwrap();
        assert_eq!(subroutine.get_value(), "print");

        let expression = subroutine.get_expressions().get(0).unwrap();
        assert_eq!(expression.get_term().get_value(), "my name");

        let expression = subroutine.get_expressions().get(1).unwrap();
        assert_eq!(expression.get_term().get_value(), "10");
    }

    #[test]
    fn build_term_subroutine_with_class() {
        let mut tokenizer = Tokenizer::new("Console.write()");

        let result = Term::build(&mut tokenizer);

        assert_eq!(result.get_type(), &TermType::SubroutineCall);
        assert_eq!(result.get_value(), "");

        let subroutine = result.get_subroutine().as_ref().unwrap();
        assert_eq!(subroutine.get_value(), "write");

        let class_name = subroutine.get_class_name().clone();
        assert_eq!(class_name.unwrap(), "Console");

        assert_eq!(subroutine.get_expressions().len(), 0);
    }

    #[test]
    fn build_symbol_with_expression() {
        let mut tokenizer = Tokenizer::new("(x + 2)");

        let result = Term::build(&mut tokenizer);

        assert_eq!(result.get_type(), &TermType::Expression);

        let expression = result.get_expression().as_ref().unwrap();

        assert_eq!(expression.get_term().get_value(), "x");

        let op = expression.get_op().clone();
        assert_eq!(op.unwrap(), "+");

        let another_term = expression.get_another_term().as_ref();
        assert_eq!(another_term.unwrap().get_value(), "2");
    }

    #[test]
    fn build_symbol_with_unary() {
        let mut tokenizer = Tokenizer::new("-x");

        let result = Term::build(&mut tokenizer);

        assert_eq!(result.get_type(), &TermType::VarName);
        assert_eq!(result.get_value(), "x");
        assert_eq!(result.get_unary_op().as_ref().unwrap(), "-");
    }

    // #[test]
    // fn build_subroutine_dec_list_void_method() {
    //     let mut tokenizer = Tokenizer::new("method void test(int x, String name) {var int y; let y = x + 1; do print(y, name) return;}");

    //     let result = SubroutineDec::build(&mut tokenizer);

    //     assert_eq!(result.len(), 1);
    // }

    // #[test]
    // fn build_subroutine_dec_list_multiple_items() {
    //     let mut tokenizer = Tokenizer::new("method void test() {} function String print() {}");

    //     let result = SubroutineDec::build(&mut tokenizer);

    //     assert_eq!(result.len(), 2);
    // }

    // #[test]
    // fn build_subroutine_dec_list_string_function() {
    //     let mut tokenizer = Tokenizer::new("function String print() {}");

    //     let result = SubroutineDec::build(&mut tokenizer);

    //     assert_eq!(result.len(), 1);

    //     let var = result.get(0).unwrap();
    //     // assert_eq!(var.get_descriptor(), "function");
    //     // assert_eq!(var.get_type(), "String");
    //     // assert_eq!(var.get_name(), "print");
    //     // assert_eq!(var.get_parameters().len(), 0);
    // }
}
