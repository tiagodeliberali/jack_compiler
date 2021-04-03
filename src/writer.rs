use crate::{
    parser::{SymbolTable, TokenTreeItem},
    tokenizer::TokenType,
};
struct VmWriter {
    symbol_table: Option<SymbolTable>,
}

impl VmWriter {
    pub fn new() -> VmWriter {
        VmWriter { symbol_table: None }
    }

    pub fn get_symbol_table(&self) -> &SymbolTable {
        &self
            .symbol_table
            .as_ref()
            .expect("Try to get symbol table before set it")
    }

    pub fn set_symbol_table(&mut self, symbol_table: SymbolTable) {
        self.symbol_table.replace(symbol_table);
    }

    pub fn build(&self, tree: &TokenTreeItem) -> Vec<String> {
        let mut result = Vec::new();
        let group = tree.get_name();

        if group.is_none() {
            return result;
        }

        let group = group.as_ref().unwrap().as_str();

        match group {
            "expression" => self.build_expression(tree),
            "term" => self.build_term(tree),
            "letStatement" => self.build_let(tree),
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
            v => panic!(format!("Unexpected term type: {:?}", v)),
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
        let mut tokenizer = Tokenizer::new("1 + 4 - 3");
        let tree = Expression::build(&mut tokenizer);

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
        let mut tokenizer = Tokenizer::new("let x = 2 + 2;");

        let mut symbol_table = SymbolTable::new();
        symbol_table.add("var", "int", "x");

        let tree = Statement::build(&mut tokenizer);

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
        let mut tokenizer = Tokenizer::new("let x = x + 2;");

        let mut symbol_table = SymbolTable::new();
        symbol_table.add("var", "int", "x");

        let tree = Statement::build(&mut tokenizer);

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
        let mut tokenizer = Tokenizer::new("let name = \"Ola\";");

        let mut symbol_table = SymbolTable::new();
        symbol_table.add("var", "String", "name");

        let tree = Statement::build(&mut tokenizer);

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
}
