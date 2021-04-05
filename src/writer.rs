use crate::{
    parser::{SymbolTable, TokenTreeItem},
    tokenizer::TokenType,
};

pub struct VmWriter {
    class_symbol_table: SymbolTable,
    symbol_table: SymbolTable,
    class_name: String,
    current_id: usize,
}

impl VmWriter {
    pub fn new() -> VmWriter {
        VmWriter {
            class_symbol_table: SymbolTable::new(),
            symbol_table: SymbolTable::new(),
            class_name: String::new(),
            current_id: 0,
        }
    }

    pub fn get_class_symbol_table(&self) -> &SymbolTable {
        &self.class_symbol_table
    }

    fn set_class_symbol_table(&mut self, symbol_table: SymbolTable) {
        self.class_symbol_table = symbol_table;
    }

    pub fn get_symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    fn set_symbol_table(&mut self, symbol_table: SymbolTable) {
        self.symbol_table = symbol_table;
    }

    pub fn get_class_name(&self) -> &String {
        &self.class_name
    }

    fn set_class_name(&mut self, value: String) {
        self.class_name = value;
    }

    pub fn get_next_id(&mut self) -> usize {
        let id = self.current_id;
        self.current_id = id + 1;

        id
    }

    pub fn build(&mut self, tree: &TokenTreeItem) -> Vec<String> {
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
            "ifStatement" => self.build_if(tree),
            "expressionList" => self.build_expression_list(tree),
            "class" => self.build_class(tree),
            "classVarDec" => {
                let symbol_table = self.build_class_var_dec(tree);
                self.set_class_symbol_table(symbol_table);
                Vec::new()
            }
            "subroutineDec" => self.build_subroutine_dec(tree),
            "parameterList" => {
                let symbol_table = self.get_class_symbol_table();
                let symbol_table = self.build_parameter_list(tree, symbol_table);

                self.set_symbol_table(symbol_table);

                Vec::new()
            }
            "varDec" => {
                let symbol_table = self.get_symbol_table();
                let symbol_table = self.build_var_dec(tree, symbol_table);

                self.set_symbol_table(symbol_table);

                Vec::new()
            }
            "subroutineBody" => self.build_subroutine_body(tree),
            value => panic!(format!("Unexpected token: {}", value)),
        }
    }

    fn build_class(&mut self, tree: &TokenTreeItem) -> Vec<String> {
        VmWriter::validate_name(tree, "class");

        if tree.get_nodes().len() <= 4 {
            return Vec::new();
        }

        let mut result = Vec::new();

        let class_name = tree
            .get_nodes()
            .get(1)
            .unwrap()
            .get_item()
            .as_ref()
            .unwrap()
            .get_value();
        self.set_class_name(class_name);

        let item = tree.get_nodes().get(3).unwrap();
        result.extend(self.build(item));

        if tree.get_nodes().len() > 5 {
            let item = tree.get_nodes().get(4).unwrap();
            result.extend(self.build(item));
        }

        result
    }

    fn build_subroutine_dec(&mut self, tree: &TokenTreeItem) -> Vec<String> {
        VmWriter::validate_name(tree, "subroutineDec");

        let mut result = Vec::new();

        let routine_type = tree
            .get_nodes()
            .get(0)
            .unwrap()
            .get_item()
            .as_ref()
            .unwrap()
            .get_value();

        let name = tree
            .get_nodes()
            .get(2)
            .unwrap()
            .get_item()
            .as_ref()
            .unwrap()
            .get_value();
        let arguments = tree.get_nodes().get(4).unwrap();
        let count_arguments = (arguments.get_nodes().len() + 1) / 3;
        let body = tree.get_nodes().get(6).unwrap();

        result.push(format!(
            "function {}.{} {}",
            self.get_class_name(),
            name,
            count_arguments
        ));

        match routine_type.as_str() {
            "constructor" => {
                result.push(format!(
                    "push constant {}",
                    self.get_class_symbol_table().count_fields()
                ));
                result.push(String::from("call Memory.alloc 1"));
                result.push(String::from("pop pointer 0"));
            }
            "function" => {}
            "method" => {
                result.push(String::from("push argument 0"));
                result.push(String::from("pop pointer 0"));
            }
            v => panic!(format!("Invalid routine type: {}", v)),
        }

        result.extend(self.build(arguments));
        result.extend(self.build(body));

        result
    }

    fn build_subroutine_body(&mut self, tree: &TokenTreeItem) -> Vec<String> {
        VmWriter::validate_name(tree, "subroutineBody");

        let mut result = Vec::new();

        if tree.get_nodes().len() > 2 {
            let item = tree.get_nodes().get(1).unwrap();
            result.extend(self.build(item));
        }

        if tree.get_nodes().len() > 3 {
            let item = tree.get_nodes().get(2).unwrap();
            result.extend(self.build(item));
        }

        result
    }
    fn build_class_var_dec(&mut self, tree: &TokenTreeItem) -> SymbolTable {
        VmWriter::validate_name(tree, "classVarDec");

        let mut symbol_table = SymbolTable::new();

        let symbol_type = tree
            .get_nodes()
            .get(0)
            .unwrap()
            .get_item()
            .as_ref()
            .unwrap()
            .get_value();
        let kind = tree
            .get_nodes()
            .get(1)
            .unwrap()
            .get_item()
            .as_ref()
            .unwrap()
            .get_value();
        let name = tree
            .get_nodes()
            .get(2)
            .unwrap()
            .get_item()
            .as_ref()
            .unwrap()
            .get_value();

        symbol_table.add(symbol_type.as_str(), kind.as_str(), name.as_str());

        let mut position = 4;

        while position < tree.get_nodes().len() {
            let name = tree
                .get_nodes()
                .get(position)
                .unwrap()
                .get_item()
                .as_ref()
                .unwrap()
                .get_value();
            symbol_table.add(symbol_type.as_str(), kind.as_str(), name.as_str());
            position += 2;
        }

        symbol_table
    }

    fn build_parameter_list(
        &self,
        tree: &TokenTreeItem,
        symbol_table: &SymbolTable,
    ) -> SymbolTable {
        VmWriter::validate_name(tree, "parameterList");

        let mut symbol_table = symbol_table.clone();

        let symbol_type = "argument";
        let kind = tree
            .get_nodes()
            .get(0)
            .unwrap()
            .get_item()
            .as_ref()
            .unwrap()
            .get_value();
        let name = tree
            .get_nodes()
            .get(1)
            .unwrap()
            .get_item()
            .as_ref()
            .unwrap()
            .get_value();

        symbol_table.add(symbol_type, kind.as_str(), name.as_str());

        let mut position = 3;

        while position < tree.get_nodes().len() {
            let kind = tree
                .get_nodes()
                .get(position)
                .unwrap()
                .get_item()
                .as_ref()
                .unwrap()
                .get_value();
            let name = tree
                .get_nodes()
                .get(position + 1)
                .unwrap()
                .get_item()
                .as_ref()
                .unwrap()
                .get_value();
            symbol_table.add(symbol_type, kind.as_str(), name.as_str());
            position += 3;
        }

        symbol_table
    }

    fn build_var_dec(&self, tree: &TokenTreeItem, symbol_table: &SymbolTable) -> SymbolTable {
        VmWriter::validate_name(tree, "varDec");

        let mut symbol_table = symbol_table.clone();

        let symbol_type = "var";
        let kind = tree
            .get_nodes()
            .get(1)
            .unwrap()
            .get_item()
            .as_ref()
            .unwrap()
            .get_value();
        let name = tree
            .get_nodes()
            .get(2)
            .unwrap()
            .get_item()
            .as_ref()
            .unwrap()
            .get_value();

        symbol_table.add(symbol_type, kind.as_str(), name.as_str());

        let mut position = 4;

        while position < tree.get_nodes().len() {
            let name = tree
                .get_nodes()
                .get(position)
                .unwrap()
                .get_item()
                .as_ref()
                .unwrap()
                .get_value();
            symbol_table.add(symbol_type, kind.as_str(), name.as_str());
            position += 2;
        }

        symbol_table
    }

    fn build_expression(&mut self, tree: &TokenTreeItem) -> Vec<String> {
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
            "+" => "add",
            "-" => "sub",
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

    fn build_term(&mut self, tree: &TokenTreeItem) -> Vec<String> {
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

                if tree.get_nodes().len() == 4 {
                    let another_term = tree.get_nodes().get(2).unwrap();
                    result.extend(self.build(another_term));
                    result.push(String::from("add"));
                    result.push(String::from("pop pointer 1"));
                    result.push(String::from("push that 0"));
                }
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
                    "~" => {
                        let another_term = tree.get_nodes().get(1).unwrap();
                        result.extend(self.build(another_term));
                        result.push(String::from("not"))
                    }
                    "(" => {
                        let another_term = tree.get_nodes().get(1).unwrap();

                        result.extend(self.build(another_term));
                    }
                    v => panic!(format!("Invalid symbol on term build: {}", v)),
                }
            }
            v => panic!(format!("Unexpected term type: {:?}", v)),
        }

        result
    }

    fn build_statements(&mut self, tree: &TokenTreeItem) -> Vec<String> {
        VmWriter::validate_name(tree, "statements");
        let mut result = Vec::new();

        for node in tree.get_nodes() {
            result.extend(self.build(node));
        }

        result
    }

    fn build_let(&mut self, tree: &TokenTreeItem) -> Vec<String> {
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
        } else if tree.get_nodes().len() == 8 {
            let identifier = tree
                .get_nodes()
                .get(1)
                .unwrap()
                .get_item()
                .as_ref()
                .unwrap()
                .get_value();

            result.push(self.get_symbol_table().get_push(identifier.as_str()));

            let expression = tree.get_nodes().get(3).unwrap();
            result.extend(self.build(expression));

            result.push(String::from("add"));

            let expression = tree.get_nodes().get(6).unwrap();
            result.extend(self.build(expression));

            result.push(String::from("pop temp 0"));
            result.push(String::from("pop pointer 1"));
            result.push(String::from("push temp 0"));
            result.push(String::from("pop that 0"));
        } else {
            panic!("Invalid number of arguments on build let statement");
        }

        result
    }

    fn build_return(&mut self, tree: &TokenTreeItem) -> Vec<String> {
        VmWriter::validate_name(tree, "returnStatement");
        let mut result = Vec::new();

        if tree.get_nodes().len() == 3 {
            let expression = tree.get_nodes().get(1).unwrap();
            result.extend(self.build(expression));
        } else {
            result.push(String::from("push constant 0"));
        }

        result.push(String::from("return"));

        result
    }

    fn build_do(&mut self, tree: &TokenTreeItem) -> Vec<String> {
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
        result.push(String::from("pop temp 0"));

        result
    }

    fn build_while(&mut self, tree: &TokenTreeItem) -> Vec<String> {
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

    fn build_if(&mut self, tree: &TokenTreeItem) -> Vec<String> {
        VmWriter::validate_name(tree, "ifStatement");
        let mut result = Vec::new();
        let count = self.get_next_id();

        let expression = tree.get_nodes().get(2).unwrap();
        result.extend(self.build(expression));

        result.push(format!("if-goto IF_TRUE{}", count));
        result.push(format!("goto IF_FALSE{}", count));
        result.push(format!("label IF_TRUE{}", count));

        let expression = tree.get_nodes().get(5).unwrap();
        result.extend(self.build(expression));

        if tree.get_nodes().len() == 7 {
            result.push(format!("label IF_FALSE{}", count));
        } else {
            result.push(format!("goto IF_END{}", count));
            result.push(format!("label IF_FALSE{}", count));

            let expression = tree.get_nodes().get(9).unwrap();
            result.extend(self.build(expression));

            result.push(format!("label IF_END{}", count));
        }

        result
    }

    fn build_expression_list(&mut self, tree: &TokenTreeItem) -> Vec<String> {
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
        parser::{ClassNode, Expression, Statement},
        tokenizer::Tokenizer,
    };

    #[test]
    fn build_expression_with_constants() {
        let tokenizer = Tokenizer::new("1 + 4 - 3");
        let tree = Expression::build(&tokenizer);

        let mut writer = VmWriter::new();
        let code: Vec<String> = writer.build(&tree);

        assert_eq!(code.get(0).unwrap(), "push constant 1");
        assert_eq!(code.get(1).unwrap(), "push constant 4");
        assert_eq!(code.get(2).unwrap(), "add");
        assert_eq!(code.get(3).unwrap(), "push constant 3");
        assert_eq!(code.get(4).unwrap(), "sub");
    }

    #[test]
    fn build_expression_with_parenthesis() {
        let tokenizer = Tokenizer::new("1 + (4 * 3)");
        let tree = Expression::build(&tokenizer);

        let mut writer = VmWriter::new();
        let code: Vec<String> = writer.build(&tree);

        assert_eq!(code.get(0).unwrap(), "push constant 1");
        assert_eq!(code.get(1).unwrap(), "push constant 4");
        assert_eq!(code.get(2).unwrap(), "push constant 3");
        assert_eq!(code.get(3).unwrap(), "Math.multiply 2");
        assert_eq!(code.get(4).unwrap(), "add");
    }

    #[test]
    fn build_let_with_array() {
        let tokenizer = Tokenizer::new("let a[x + 1] = 5;");
        let tree = Statement::build(&tokenizer);

        let mut symbol_table = SymbolTable::new();
        symbol_table.add("var", "int", "x");
        symbol_table.add("var", "Array", "a");

        let mut writer = VmWriter::new();
        writer.set_symbol_table(symbol_table);
        let code: Vec<String> = writer.build(&tree);

        assert_eq!(code.get(0).unwrap(), "push local 1");
        assert_eq!(code.get(1).unwrap(), "push local 0");
        assert_eq!(code.get(2).unwrap(), "push constant 1");
        assert_eq!(code.get(3).unwrap(), "add");
        assert_eq!(code.get(4).unwrap(), "add");

        assert_eq!(code.get(5).unwrap(), "push constant 5");

        assert_eq!(code.get(6).unwrap(), "pop temp 0");
        assert_eq!(code.get(7).unwrap(), "pop pointer 1");
        assert_eq!(code.get(8).unwrap(), "push temp 0");
        assert_eq!(code.get(9).unwrap(), "pop that 0");
    }

    #[test]
    fn build_let_with_two_arrays() {
        let tokenizer = Tokenizer::new("let a[x] = a[5];");
        let tree = Statement::build(&tokenizer);

        let mut symbol_table = SymbolTable::new();
        symbol_table.add("var", "int", "x");
        symbol_table.add("var", "Array", "a");

        let mut writer = VmWriter::new();
        writer.set_symbol_table(symbol_table);
        let code: Vec<String> = writer.build(&tree);

        assert_eq!(code.get(0).unwrap(), "push local 1");
        assert_eq!(code.get(1).unwrap(), "push local 0");
        assert_eq!(code.get(2).unwrap(), "add");

        assert_eq!(code.get(3).unwrap(), "push local 1");
        assert_eq!(code.get(4).unwrap(), "push constant 5");
        assert_eq!(code.get(5).unwrap(), "add");
        assert_eq!(code.get(6).unwrap(), "pop pointer 1");
        assert_eq!(code.get(7).unwrap(), "push that 0");

        assert_eq!(code.get(8).unwrap(), "pop temp 0");
        assert_eq!(code.get(9).unwrap(), "pop pointer 1");
        assert_eq!(code.get(10).unwrap(), "push temp 0");
        assert_eq!(code.get(11).unwrap(), "pop that 0");
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
        assert_eq!(code.get(2).unwrap(), "add");
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
        assert_eq!(code.get(2).unwrap(), "add");
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

        let mut writer = VmWriter::new();
        let code: Vec<String> = writer.build(&tree);

        assert_eq!(code.get(0).unwrap(), "push constant 0");
        assert_eq!(code.get(1).unwrap(), "not");
        assert_eq!(code.get(2).unwrap(), "return");
    }

    #[test]
    fn build_return_void() {
        let tokenizer = Tokenizer::new("return;");
        let tree = Statement::build(&tokenizer);

        let mut writer = VmWriter::new();
        let code: Vec<String> = writer.build(&tree);

        assert_eq!(code.get(0).unwrap(), "push constant 0");
        assert_eq!(code.get(1).unwrap(), "return");
    }

    #[test]
    fn build_do_this() {
        let tokenizer = Tokenizer::new("do Memory.deAlloc(this);");
        let tree = Statement::build(&tokenizer);

        let mut writer = VmWriter::new();
        let code: Vec<String> = writer.build(&tree);

        assert_eq!(code.get(0).unwrap(), "push pointer 0");
        assert_eq!(code.get(1).unwrap(), "call Memory.deAlloc 1");
        assert_eq!(code.get(2).unwrap(), "pop temp 0");
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
        assert_eq!(code.get(4).unwrap(), "pop temp 0");
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

    #[test]
    fn build_if() {
        let tokenizer = Tokenizer::new("if (~exit) { do print(10); }");
        let tree = Statement::build(&tokenizer);

        let mut symbol_table = SymbolTable::new();
        symbol_table.add("var", "boolean", "exit");

        let mut writer = VmWriter::new();
        writer.set_symbol_table(symbol_table);
        writer.set_class_name(String::from("TestClass"));

        // advance internal id by 1
        let current_id = writer.get_next_id();
        assert_eq!(current_id, 0);

        let code: Vec<String> = writer.build(&tree);

        assert_eq!(code.get(0).unwrap(), "push local 0");
        assert_eq!(code.get(1).unwrap(), "not");

        assert_eq!(code.get(2).unwrap(), "if-goto IF_TRUE1");
        assert_eq!(code.get(3).unwrap(), "goto IF_FALSE1");
        assert_eq!(code.get(4).unwrap(), "label IF_TRUE1");

        assert_eq!(code.get(5).unwrap(), "push constant 10");
        assert_eq!(code.get(6).unwrap(), "call TestClass.print 1");
        assert_eq!(code.get(7).unwrap(), "pop temp 0");

        assert_eq!(code.get(8).unwrap(), "label IF_FALSE1");
    }

    #[test]
    fn build_if_else() {
        let tokenizer = Tokenizer::new("if (~exit) { do print(10); } else { do exit(); }");
        let tree = Statement::build(&tokenizer);

        let mut symbol_table = SymbolTable::new();
        symbol_table.add("var", "boolean", "exit");

        let mut writer = VmWriter::new();
        writer.set_symbol_table(symbol_table);
        writer.set_class_name(String::from("TestClass"));

        // advance internal id by 1
        let current_id = writer.get_next_id();
        assert_eq!(current_id, 0);

        let code: Vec<String> = writer.build(&tree);

        assert_eq!(code.get(0).unwrap(), "push local 0");
        assert_eq!(code.get(1).unwrap(), "not");

        assert_eq!(code.get(2).unwrap(), "if-goto IF_TRUE1");
        assert_eq!(code.get(3).unwrap(), "goto IF_FALSE1");
        assert_eq!(code.get(4).unwrap(), "label IF_TRUE1");

        assert_eq!(code.get(5).unwrap(), "push constant 10");
        assert_eq!(code.get(6).unwrap(), "call TestClass.print 1");
        assert_eq!(code.get(7).unwrap(), "pop temp 0");

        assert_eq!(code.get(8).unwrap(), "goto IF_END1");
        assert_eq!(code.get(9).unwrap(), "label IF_FALSE1");

        assert_eq!(code.get(10).unwrap(), "call TestClass.exit 0");
        assert_eq!(code.get(11).unwrap(), "pop temp 0");

        assert_eq!(code.get(12).unwrap(), "label IF_END1");
    }

    #[test]
    fn build_constructor() {
        let source = "class Test { field int a, b; constructor Test new(int set_a) { var boolean exit; let a = set_a; let b = 10; return this; } }";
        let tokenizer = Tokenizer::new(source);
        let tree = ClassNode::build(&tokenizer);
        let mut writer = VmWriter::new();

        let code: Vec<String> = writer.build(&tree);

        assert_eq!(code.get(0).unwrap(), "function Test.new 1");
        assert_eq!(code.get(1).unwrap(), "push constant 2");
        assert_eq!(code.get(2).unwrap(), "call Memory.alloc 1");
        assert_eq!(code.get(3).unwrap(), "pop pointer 0");

        assert_eq!(code.get(4).unwrap(), "push argument 0");
        assert_eq!(code.get(5).unwrap(), "pop this 0");

        assert_eq!(code.get(6).unwrap(), "push constant 10");
        assert_eq!(code.get(7).unwrap(), "pop this 1");

        assert_eq!(code.get(8).unwrap(), "push pointer 0");
        assert_eq!(code.get(9).unwrap(), "return");
    }
}
