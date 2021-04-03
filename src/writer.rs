use crate::{
    parser::{SymbolTable, TokenTreeItem},
    tokenizer::TokenType,
};
use std::cell::Cell;

pub struct VmWriter {
    symbol_table: Option<SymbolTable>,
    class_name: String,
    current_id: Cell<usize>,
}

impl VmWriter {
    pub fn new() -> VmWriter {
        VmWriter {
            symbol_table: None,
            class_name: String::new(),
            current_id: Cell::new(0),
        }
    }

    pub fn get_symbol_table(&self) -> &SymbolTable {
        &self
            .symbol_table
            .as_ref()
            .expect("Try to get symbol table before set it")
    }

    fn set_symbol_table(&mut self, symbol_table: SymbolTable) {
        self.symbol_table.replace(symbol_table);
    }

    pub fn get_class_name(&self) -> &String {
        &self.class_name
    }

    fn set_class_name(&mut self, value: String) {
        self.class_name = value;
    }

    pub fn get_next_id(&self) -> usize {
        let id = self.current_id.get();
        self.current_id.set(id + 1);

        id
    }

    pub fn build(&self, tree: &TokenTreeItem) -> Vec<String> {
        let group = tree.get_name();

        if group.is_none() {
            return Vec::new();
        }

        let group = group.as_ref().unwrap().as_str();

        match group {
            "expression" => self.build_expression(tree),
            "term" => self.build_term(tree),
            "statements" => self.build_statements(tree),
            "letStatement" => self.build_let(tree),
            "returnStatement" => self.build_return(tree),
            "doStatement" => self.build_do(tree),
            "whileStatement" => self.build_while(tree),
            "expressionList" => self.build_expression_list(tree),
            value => panic!(format!("Unexpected token: {}", value)),
        }
    }

    fn build_expression(&self, tree: &TokenTreeItem) -> Vec<String> {
        VmWriter::validate_name(tree, "expression");

        let mut result = Vec::new();

        let term = tree.get_nodes().get(0).unwrap();
        result.extend(self.build(term));

        let mut i = 1;

        while i < tree.get_nodes().len() {
            let term = tree.get_nodes().get(i + 1).unwrap();
            result.extend(self.build(term));

            let op = tree.get_nodes().get(i).unwrap();
            result.push(VmWriter::build_expression_op(op));

            i += 2;
        }

        result
    }

    fn build_expression_op(op: &TokenTreeItem) -> String {
        let result = match op.get_item().as_ref().unwrap().get_value().as_str() {
            "+" => "+",
            "-" => "-",
            "*" => "Math.multiply 2",
            "/" => "Math.divide 2",
            "&" => "and",
            "|" => "or",
            ">" => "gt",
            "<" => "lt",
            "=" => "eq",
            v => panic!(format!("Invalid op on expression build: {}", v)),
        };

        String::from(result)
    }

    fn build_term(&self, tree: &TokenTreeItem) -> Vec<String> {
        VmWriter::validate_name(tree, "term");
        let mut result = Vec::new();

        let item = tree
            .get_nodes()
            .get(0)
            .unwrap()
            .get_item()
            .as_ref()
            .unwrap();

        match item.get_type() {
            TokenType::Integer => result.push(format!("push constant {}", item.get_value())),
            TokenType::String => {
                let value = item.get_value();
                result.push(format!("push constant {}", value.len()));
                result.push(String::from("call String.new 1"));

                for c in value.chars() {
                    result.push(format!("push constant {}", c as i32));
                    result.push(String::from("call String.appendChar 2"));
                }
            }
            TokenType::Identifier => {
                let identifier = item.get_value();
                result.push(self.get_symbol_table().get_push(identifier.as_str()));
            }
            TokenType::Keyword => {
                let value = item.get_value();
                match value.as_str() {
                    "false" => result.push(String::from("push constant 0")),
                    "true" => {
                        result.push(String::from("push constant 0"));
                        result.push(String::from("not"));
                    }
                    "this" => result.push(String::from("push pointer 0")),
                    "null" => result.push(String::from("push constant 0")),
                    v => panic!(format!("Invalid keywork on term build: {}", v)),
                }
            }
            TokenType::Symbol => {
                let value = item.get_value();
                match value.as_str() {
                    "-" => {
                        let another_term = tree.get_nodes().get(1).unwrap();

                        result.extend(self.build(another_term));
                        result.push(String::from("neg"))
                    }
                    v => panic!(format!("Invalid symbol on term build: {}", v)),
                }
            }
            v => panic!(format!("Unexpected term type: {:?}", v)),
        }

        result
    }

    fn build_statements(&self, tree: &TokenTreeItem) -> Vec<String> {
        VmWriter::validate_name(tree, "statements");
        let mut result = Vec::new();

        for node in tree.get_nodes() {
            result.extend(self.build(node));
        }

        result
    }

    fn build_let(&self, tree: &TokenTreeItem) -> Vec<String> {
        VmWriter::validate_name(tree, "letStatement");
        let mut result = Vec::new();

        if tree.get_nodes().len() == 5 {
            let expression = tree.get_nodes().get(3).unwrap();
            result.extend(self.build(expression));

            let identifier = tree
                .get_nodes()
                .get(1)
                .unwrap()
                .get_item()
                .as_ref()
                .unwrap()
                .get_value();

            result.push(self.get_symbol_table().get_pop(identifier.as_str()))
        }

        result
    }

    fn build_return(&self, tree: &TokenTreeItem) -> Vec<String> {
        VmWriter::validate_name(tree, "returnStatement");
        let mut result = Vec::new();

        if tree.get_nodes().len() == 3 {
            let expression = tree.get_nodes().get(1).unwrap();
            result.extend(self.build(expression));
        }

        result.push(String::from("return"));

        result
    }

    fn build_do(&self, tree: &TokenTreeItem) -> Vec<String> {
        VmWriter::validate_name(tree, "doStatement");
        let mut result = Vec::new();

        let mut base_index: usize = 1;

        let class_name = if tree.get_nodes().len() == 8 {
            base_index += 2;
            tree.get_nodes()
                .get(1)
                .unwrap()
                .get_item()
                .as_ref()
                .unwrap()
                .get_value()
        } else {
            self.get_class_name().clone()
        };

        let method = tree
            .get_nodes()
            .get(base_index)
            .unwrap()
            .get_item()
            .as_ref()
            .unwrap()
            .get_value();
        let expression_list = tree.get_nodes().get(base_index + 2).unwrap();
        let arguments = (expression_list.get_nodes().len() + 1) / 2;

        result.extend(self.build(expression_list));

        result.push(format!("call {}.{} {}", class_name, method, arguments));

        result
    }

    fn build_while(&self, tree: &TokenTreeItem) -> Vec<String> {
        VmWriter::validate_name(tree, "whileStatement");
        let mut result = Vec::new();
        let count = self.get_next_id();

        result.push(format!("label WHILE_EXP{}", count));

        let expression = tree.get_nodes().get(2).unwrap();
        result.extend(self.build(expression));

        result.push(String::from("not"));
        result.push(format!("if-goto WHILE_END{}", count));

        let expression = tree.get_nodes().get(5).unwrap();
        result.extend(self.build(expression));

        result.push(format!("goto WHILE_EXP{}", count));
        result.push(format!("label WHILE_END{}", count));

        result
    }

    fn build_expression_list(&self, tree: &TokenTreeItem) -> Vec<String> {
        VmWriter::validate_name(tree, "expressionList");
        let mut result = Vec::new();

        let mut i = 0;

        while i < tree.get_nodes().len() {
            result.extend(self.build(tree.get_nodes().get(i).unwrap()));
            i += 2;
        }

        result
    }

    fn validate_name(item: &TokenTreeItem, name: &str) {
        let item_name = item.get_name().as_ref();

        if item_name.is_none() {
            panic!(format!("Missing name on TokenTreeItem. Expected {}", name));
        }

        let item_name = item_name.unwrap();
        if item_name != name {
            panic!(format!(
                "Invalid name on TokenTreeItem. Expected {}. Found {}.",
                name, item_name
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        parser::{Expression, Statement},
        tokenizer::Tokenizer,
    };

    #[test]
    fn build_expression_with_constants() {
        let tokenizer = Tokenizer::new("1 + 4 - 3");
        let tree = Expression::build(&tokenizer);

        let writer = VmWriter::new();
        let code: Vec<String> = writer.build(&tree);

        assert_eq!(code.get(0).unwrap(), "push constant 1");
        assert_eq!(code.get(1).unwrap(), "push constant 4");
        assert_eq!(code.get(2).unwrap(), "+");
        assert_eq!(code.get(3).unwrap(), "push constant 3");
        assert_eq!(code.get(4).unwrap(), "-");
    }

    #[test]
    fn build_let_with_constants() {
        let tokenizer = Tokenizer::new("let x = 2 + 2;");

        let mut symbol_table = SymbolTable::new();
        symbol_table.add("var", "int", "x");

        let tree = Statement::build(&tokenizer);

        let mut writer = VmWriter::new();
        writer.set_symbol_table(symbol_table);
        let code: Vec<String> = writer.build(&tree);

        assert_eq!(code.get(0).unwrap(), "push constant 2");
        assert_eq!(code.get(1).unwrap(), "push constant 2");
        assert_eq!(code.get(2).unwrap(), "+");
        assert_eq!(code.get(3).unwrap(), "pop local 0");
    }

    #[test]
    fn build_let_with_constants_both_sides() {
        let tokenizer = Tokenizer::new("let x = x + 2;");

        let mut symbol_table = SymbolTable::new();
        symbol_table.add("var", "int", "x");

        let tree = Statement::build(&tokenizer);

        let mut writer = VmWriter::new();
        writer.set_symbol_table(symbol_table);
        let code: Vec<String> = writer.build(&tree);

        assert_eq!(code.get(0).unwrap(), "push local 0");
        assert_eq!(code.get(1).unwrap(), "push constant 2");
        assert_eq!(code.get(2).unwrap(), "+");
        assert_eq!(code.get(3).unwrap(), "pop local 0");
    }

    #[test]
    fn build_let_with_string() {
        let tokenizer = Tokenizer::new("let name = \"Ola\";");

        let mut symbol_table = SymbolTable::new();
        symbol_table.add("var", "String", "name");

        let tree = Statement::build(&tokenizer);

        let mut writer = VmWriter::new();
        writer.set_symbol_table(symbol_table);
        let code: Vec<String> = writer.build(&tree);

        assert_eq!(code.get(0).unwrap(), "push constant 3");
        assert_eq!(code.get(1).unwrap(), "call String.new 1");
        assert_eq!(code.get(2).unwrap(), "push constant 79");
        assert_eq!(code.get(3).unwrap(), "call String.appendChar 2");
        assert_eq!(code.get(4).unwrap(), "push constant 108");
        assert_eq!(code.get(5).unwrap(), "call String.appendChar 2");
        assert_eq!(code.get(6).unwrap(), "push constant 97");
        assert_eq!(code.get(7).unwrap(), "call String.appendChar 2");
        assert_eq!(code.get(8).unwrap(), "pop local 0");
    }

    #[test]
    fn build_return_false() {
        let tokenizer = Tokenizer::new("return true;");
        let tree = Statement::build(&tokenizer);

        let writer = VmWriter::new();
        let code: Vec<String> = writer.build(&tree);

        assert_eq!(code.get(0).unwrap(), "push constant 0");
        assert_eq!(code.get(1).unwrap(), "not");
        assert_eq!(code.get(2).unwrap(), "return");
    }

    #[test]
    fn build_return_void() {
        let tokenizer = Tokenizer::new("return;");
        let tree = Statement::build(&tokenizer);

        let writer = VmWriter::new();
        let code: Vec<String> = writer.build(&tree);

        assert_eq!(code.get(0).unwrap(), "return");
    }

    #[test]
    fn build_do_this() {
        let tokenizer = Tokenizer::new("do Memory.deAlloc(this);");
        let tree = Statement::build(&tokenizer);

        let writer = VmWriter::new();
        let code: Vec<String> = writer.build(&tree);

        assert_eq!(code.get(0).unwrap(), "push pointer 0");
        assert_eq!(code.get(1).unwrap(), "call Memory.deAlloc 1");
    }

    #[test]
    fn build_do_with_args() {
        let tokenizer = Tokenizer::new("do print(name, age, country);");
        let tree = Statement::build(&tokenizer);

        let mut symbol_table = SymbolTable::new();
        symbol_table.add("var", "String", "name");
        symbol_table.add("var", "int", "age");
        symbol_table.add("var", "String", "country");

        let mut writer = VmWriter::new();
        writer.set_symbol_table(symbol_table);
        writer.set_class_name(String::from("TestClass"));
        let code: Vec<String> = writer.build(&tree);

        assert_eq!(code.get(0).unwrap(), "push local 0");
        assert_eq!(code.get(1).unwrap(), "push local 1");
        assert_eq!(code.get(2).unwrap(), "push local 2");
        assert_eq!(code.get(3).unwrap(), "call TestClass.print 3");
    }

    #[test]
    fn build_while() {
        let tokenizer = Tokenizer::new("while (x < 10) { let a = -1; }");
        let tree = Statement::build(&tokenizer);

        let mut symbol_table = SymbolTable::new();
        symbol_table.add("argument", "int", "x");
        symbol_table.add("var", "int", "a");

        let mut writer = VmWriter::new();
        writer.set_symbol_table(symbol_table);
        writer.set_class_name(String::from("TestClass"));

        // advance internal id by 1
        let current_id = writer.get_next_id();
        assert_eq!(current_id, 0);

        let code: Vec<String> = writer.build(&tree);

        assert_eq!(code.get(0).unwrap(), "label WHILE_EXP1");
        assert_eq!(code.get(1).unwrap(), "push argument 0");
        assert_eq!(code.get(2).unwrap(), "push constant 10");
        assert_eq!(code.get(3).unwrap(), "lt");
        assert_eq!(code.get(4).unwrap(), "not");
        assert_eq!(code.get(5).unwrap(), "if-goto WHILE_END1");

        assert_eq!(code.get(6).unwrap(), "push constant 1");
        assert_eq!(code.get(7).unwrap(), "neg");
        assert_eq!(code.get(8).unwrap(), "pop local 0");

        assert_eq!(code.get(9).unwrap(), "goto WHILE_EXP1");
        assert_eq!(code.get(10).unwrap(), "label WHILE_END1");
    }
}
