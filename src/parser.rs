use crate::tokenizer::{TokenType, Tokenizer, UNARY_OP_SYMBOLS};

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

struct Parameter {
    parameter_type: String,
    name: String,
}

impl Parameter {
    fn build(tokenizer: &mut Tokenizer) -> Parameter {
        let parameter_type = tokenizer.retrieve_type();
        let name = tokenizer.retrieve_identifier();

        Parameter {
            parameter_type,
            name,
        }
    }

    pub fn get_type(&self) -> &String {
        &self.parameter_type
    }

    pub fn get_name(&self) -> &String {
        &self.name
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
    parameters: Vec<Parameter>,
    var_dec: Vec<VarDec>,
    statements: Vec<Statement>,
}

impl SubroutineDec {
    pub fn build(tokenizer: &mut Tokenizer) -> Vec<SubroutineDec> {
        let mut result = Vec::new();

        while let Some(next_token) = tokenizer.peek_next() {
            if next_token.get_value() == "}" {
                break;
            }

            result.push(SubroutineDec::build_subroutine(tokenizer));
        }

        result
    }

    fn build_subroutine(tokenizer: &mut Tokenizer) -> SubroutineDec {
        let descriptor = tokenizer.retrieve_keyword();
        let routine_type = tokenizer
            .retrieve_any(Vec::from([TokenType::Keyword, TokenType::Identifier]))
            .get_value();
        let name = tokenizer.retrieve_identifier();
        tokenizer.consume("(");
        let parameters = SubroutineDec::build_parameters(tokenizer);
        tokenizer.consume(")");
        tokenizer.consume("{");
        let var_dec = VarDec::build_var(tokenizer);
        let statements = Statement::build_list(tokenizer);
        tokenizer.consume("}");

        SubroutineDec {
            descriptor,
            routine_type,
            name,
            parameters,
            var_dec,
            statements,
        }
    }

    fn build_parameters(tokenizer: &mut Tokenizer) -> Vec<Parameter> {
        let mut result = Vec::new();

        while let Some(next_token) = tokenizer.peek_next() {
            if next_token.get_value() == ")" {
                break;
            }

            if next_token.get_value() == "," {
                tokenizer.consume(",");
            }

            result.push(Parameter::build(tokenizer));
        }

        result
    }

    pub fn get_descriptor(&self) -> &String {
        &self.descriptor
    }

    pub fn get_type(&self) -> &String {
        &self.routine_type
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_parameters(&self) -> &Vec<Parameter> {
        &self.parameters
    }

    pub fn get_var_dec(&self) -> &Vec<VarDec> {
        &self.var_dec
    }

    pub fn get_statements(&self) -> &Vec<Statement> {
        &self.statements
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum StatementType {
    Return,
    Do,
    While,
    If,
    Let,
}
struct Statement {
    statement_type: StatementType,
    statement_return: Option<StatementReturn>,
    statement_do: Option<StatementDo>,
    statement_while: Option<StatementWhile>,
    statement_if: Option<StatementIf>,
    statement_let: Option<StatementLet>,
}

impl Statement {
    pub fn build_list(tokenizer: &mut Tokenizer) -> Vec<Statement> {
        let mut result = Vec::new();

        while let Some(next_token) = tokenizer.peek_next() {
            if next_token.get_value() == "}" {
                break;
            }

            result.push(Statement::build(tokenizer));
        }

        result
    }

    pub fn build(tokenizer: &mut Tokenizer) -> Statement {
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

    fn build_return(tokenizer: &mut Tokenizer) -> Statement {
        Statement {
            statement_type: StatementType::Return,
            statement_return: Some(StatementReturn::build(tokenizer)),
            statement_do: None,
            statement_while: None,
            statement_if: None,
            statement_let: None,
        }
    }

    fn build_do(tokenizer: &mut Tokenizer) -> Statement {
        Statement {
            statement_type: StatementType::Do,
            statement_return: None,
            statement_do: Some(StatementDo::build(tokenizer)),
            statement_while: None,
            statement_if: None,
            statement_let: None,
        }
    }

    fn build_while(tokenizer: &mut Tokenizer) -> Statement {
        Statement {
            statement_type: StatementType::While,
            statement_return: None,
            statement_do: None,
            statement_while: Some(StatementWhile::build(tokenizer)),
            statement_if: None,
            statement_let: None,
        }
    }

    fn build_if(tokenizer: &mut Tokenizer) -> Statement {
        Statement {
            statement_type: StatementType::If,
            statement_return: None,
            statement_do: None,
            statement_while: None,
            statement_if: Some(StatementIf::build(tokenizer)),
            statement_let: None,
        }
    }

    fn build_let(tokenizer: &mut Tokenizer) -> Statement {
        Statement {
            statement_type: StatementType::Let,
            statement_return: None,
            statement_do: None,
            statement_while: None,
            statement_if: None,
            statement_let: Some(StatementLet::build(tokenizer)),
        }
    }

    pub fn get_type(&self) -> &StatementType {
        &self.statement_type
    }

    pub fn get_return(&self) -> &Option<StatementReturn> {
        &self.statement_return
    }

    pub fn get_do(&self) -> &Option<StatementDo> {
        &self.statement_do
    }

    pub fn get_while(&self) -> &Option<StatementWhile> {
        &self.statement_while
    }

    pub fn get_if(&self) -> &Option<StatementIf> {
        &self.statement_if
    }

    pub fn get_let(&self) -> &Option<StatementLet> {
        &self.statement_let
    }
}

struct StatementReturn {
    expression: Option<Expression>,
}

impl StatementReturn {
    pub fn build(tokenizer: &mut Tokenizer) -> StatementReturn {
        tokenizer.consume("return");

        let next_token = tokenizer.peek_next().unwrap();

        if next_token.get_value() == ";" {
            tokenizer.consume(";");
            return StatementReturn { expression: None };
        }

        let expression = Expression::build(tokenizer);
        tokenizer.consume(";");

        StatementReturn {
            expression: Some(expression),
        }
    }

    pub fn get_expression(&self) -> &Option<Expression> {
        &self.expression
    }
}

struct StatementDo {
    subroutine: SubroutineCall,
}

impl StatementDo {
    pub fn build(tokenizer: &mut Tokenizer) -> StatementDo {
        tokenizer.consume("do");
        let subroutine = SubroutineCall::build(tokenizer);
        tokenizer.consume(";");

        StatementDo { subroutine }
    }

    pub fn get_subroutine(&self) -> &SubroutineCall {
        &self.subroutine
    }
}

struct StatementWhile {
    expression: Expression,
    statements: Vec<Statement>,
}

impl StatementWhile {
    pub fn build(tokenizer: &mut Tokenizer) -> StatementWhile {
        tokenizer.consume("while");
        tokenizer.consume("(");
        let expression = Expression::build(tokenizer);
        tokenizer.consume(")");
        tokenizer.consume("{");
        let statements = Statement::build_list(tokenizer);
        tokenizer.consume("}");

        StatementWhile {
            expression,
            statements,
        }
    }

    pub fn get_expression(&self) -> &Expression {
        &self.expression
    }

    pub fn get_statements(&self) -> &Vec<Statement> {
        &self.statements
    }
}

struct StatementIf {
    expression: Expression,
    statements: Vec<Statement>,
    else_statements: Option<Vec<Statement>>,
}

impl StatementIf {
    pub fn build(tokenizer: &mut Tokenizer) -> StatementIf {
        tokenizer.consume("if");
        tokenizer.consume("(");
        let expression = Expression::build(tokenizer);
        tokenizer.consume(")");
        tokenizer.consume("{");
        let statements = Statement::build_list(tokenizer);
        tokenizer.consume("}");

        let next_token = tokenizer.peek_next();

        if next_token.is_none() {
            return StatementIf {
                expression,
                statements,
                else_statements: None,
            };
        }

        let next_token = next_token.unwrap();

        if next_token.get_value() == "else" {
            tokenizer.consume("else");
            tokenizer.consume("{");
            let else_statements = Statement::build_list(tokenizer);
            tokenizer.consume("}");

            return StatementIf {
                expression,
                statements,
                else_statements: Some(else_statements),
            };
        }

        StatementIf {
            expression,
            statements,
            else_statements: None,
        }
    }

    pub fn get_expression(&self) -> &Expression {
        &self.expression
    }

    pub fn get_statements(&self) -> &Vec<Statement> {
        &self.statements
    }

    pub fn get_else_statements(&self) -> &Option<Vec<Statement>> {
        &self.else_statements
    }
}

struct StatementLet {
    var_name: String,
    expression: Expression,
    array_expression: Option<Expression>,
}

impl StatementLet {
    pub fn build(tokenizer: &mut Tokenizer) -> StatementLet {
        tokenizer.consume("let");
        let var_name = tokenizer.retrieve_identifier();
        let mut array_expression: Option<Expression> = None;

        let next_token = tokenizer.peek_next().unwrap();

        if next_token.get_value() == "[" {
            tokenizer.consume("[");
            array_expression.replace(Expression::build(tokenizer));
            tokenizer.consume("]");
        }

        tokenizer.consume("=");
        let expression = Expression::build(tokenizer);
        tokenizer.consume(";");

        StatementLet {
            var_name,
            expression,
            array_expression,
        }
    }

    pub fn get_expression(&self) -> &Expression {
        &self.expression
    }

    pub fn get_var_name(&self) -> &String {
        &self.var_name
    }

    pub fn get_array_expression(&self) -> &Option<Expression> {
        &self.array_expression
    }
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
    pub fn build(tokenizer: &mut Tokenizer) -> SubroutineCall {
        let value = tokenizer.retrieve_identifier();
        SubroutineCall::build_from_value(value, tokenizer)
    }

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

    #[test]
    fn build_statement_list_return_expression() {
        let mut tokenizer = Tokenizer::new("return name;");

        let statements = Statement::build_list(&mut tokenizer);

        assert_eq!(statements.len(), 1);

        let return_statement = statements.get(0).unwrap();

        assert_eq!(return_statement.get_type(), &StatementType::Return);

        let expression = return_statement
            .get_return()
            .as_ref()
            .unwrap()
            .get_expression()
            .as_ref()
            .unwrap();
        assert_eq!(expression.get_term().get_value(), "name");
    }

    #[test]
    fn build_statement_list_return() {
        let mut tokenizer = Tokenizer::new("return;");

        let statements = Statement::build_list(&mut tokenizer);

        assert_eq!(statements.len(), 1);

        let return_statement = statements.get(0).unwrap();

        assert_eq!(return_statement.get_type(), &StatementType::Return);

        let expression = return_statement
            .get_return()
            .as_ref()
            .unwrap()
            .get_expression()
            .as_ref();
        assert!(expression.is_none());
    }

    #[test]
    fn build_statement_list_do() {
        let mut tokenizer = Tokenizer::new("do Console.print(test);");

        let statements = Statement::build_list(&mut tokenizer);

        assert_eq!(statements.len(), 1);

        let do_statement = statements.get(0).unwrap();

        assert_eq!(do_statement.get_type(), &StatementType::Do);

        let subroutine = do_statement.get_do().as_ref().unwrap().get_subroutine();
        assert_eq!(subroutine.get_class_name().as_ref().unwrap(), "Console");
        assert_eq!(subroutine.get_value(), "print");
    }

    #[test]
    fn build_statement_list_while() {
        let mut tokenizer = Tokenizer::new("while (x < 5) { do Console.print(test); }");

        let statements = Statement::build_list(&mut tokenizer);

        assert_eq!(statements.len(), 1);

        let statement = statements.get(0).unwrap();

        assert_eq!(statement.get_type(), &StatementType::While);

        let while_statement = statement.get_while().as_ref().unwrap();

        let expression = while_statement.get_expression();
        assert_eq!(expression.get_term().get_value(), "x");

        let statements = while_statement.get_statements();
        assert_eq!(statements.len(), 1);
    }

    #[test]
    fn build_statement_list_if() {
        let mut tokenizer = Tokenizer::new("if (x < 5) { return 10; }");

        let statements = Statement::build_list(&mut tokenizer);

        assert_eq!(statements.len(), 1);

        let statement = statements.get(0).unwrap();

        assert_eq!(statement.get_type(), &StatementType::If);

        let if_statement = statement.get_if().as_ref().unwrap();
        assert!(if_statement.get_else_statements().is_none());

        let expression = if_statement.get_expression();
        assert_eq!(expression.get_term().get_value(), "x");

        let statements = if_statement.get_statements();
        assert_eq!(statements.len(), 1);
    }

    #[test]
    fn build_statement_list_if_else() {
        let mut tokenizer = Tokenizer::new("if (x < 5) { return 10; } else { return 20; }");

        let statements = Statement::build_list(&mut tokenizer);

        assert_eq!(statements.len(), 1);

        let statement = statements.get(0).unwrap();

        assert_eq!(statement.get_type(), &StatementType::If);

        let if_statement = statement.get_if().as_ref().unwrap();
        assert!(if_statement.get_else_statements().is_some());

        let expression = if_statement.get_expression();
        assert_eq!(expression.get_term().get_value(), "x");

        let statements = if_statement.get_statements();
        assert_eq!(statements.len(), 1);

        let statements = if_statement.get_else_statements().as_ref().unwrap();
        assert_eq!(statements.len(), 1);
    }

    #[test]
    fn build_statement_list_let() {
        let mut tokenizer = Tokenizer::new("let x = 25;");

        let statements = Statement::build_list(&mut tokenizer);

        assert_eq!(statements.len(), 1);

        let statement = statements.get(0).unwrap();

        assert_eq!(statement.get_type(), &StatementType::Let);

        let let_statement = statement.get_let().as_ref().unwrap();
        assert_eq!(let_statement.get_var_name(), "x");
        assert!(let_statement.get_array_expression().is_none());

        let expression = let_statement.get_expression();
        assert_eq!(expression.get_term().get_value(), "25");
    }

    #[test]
    fn build_statement_list_let_array() {
        let mut tokenizer = Tokenizer::new("let names[10] = \"test\";");

        let statements = Statement::build_list(&mut tokenizer);

        assert_eq!(statements.len(), 1);

        let statement = statements.get(0).unwrap();

        assert_eq!(statement.get_type(), &StatementType::Let);

        let let_statement = statement.get_let().as_ref().unwrap();
        assert_eq!(let_statement.get_var_name(), "names");
        assert!(let_statement.get_array_expression().is_some());

        let array_expression = let_statement.get_array_expression().as_ref().unwrap();
        assert_eq!(array_expression.get_term().get_value(), "10");

        let expression = let_statement.get_expression();
        assert_eq!(expression.get_term().get_value(), "test");
    }

    #[test]
    fn build_subroutine_dec_list_string_function() {
        let mut tokenizer = Tokenizer::new("function String print() {}");

        let result = SubroutineDec::build(&mut tokenizer);

        assert_eq!(result.len(), 1);

        let subroutine = result.get(0).unwrap();
        assert_eq!(subroutine.get_descriptor(), "function");
        assert_eq!(subroutine.get_type(), "String");
        assert_eq!(subroutine.get_name(), "print");
        assert_eq!(subroutine.get_parameters().len(), 0);
    }

    #[test]
    fn build_subroutine_dec_list_multiple_items() {
        let mut tokenizer = Tokenizer::new("method void test() {} function String print() {}");

        let result = SubroutineDec::build(&mut tokenizer);

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn build_subroutine_dec_list_void_method() {
        let mut tokenizer = Tokenizer::new("method void test(int x, String name) {var int y; let y = x + 1; do print(y, name); return;}");

        let result = SubroutineDec::build(&mut tokenizer);

        assert_eq!(result.len(), 1);

        let subroutine = result.get(0).unwrap();
        assert_eq!(subroutine.get_descriptor(), "method");
        assert_eq!(subroutine.get_type(), "void");
        assert_eq!(subroutine.get_name(), "test");
        assert_eq!(subroutine.get_parameters().len(), 2);
    }
}
