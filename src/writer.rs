use crate::{
    parser::{Expression, TokenTreeItem},
    tokenizer::{TokenType, Tokenizer},
};
struct VmWriter {}

impl VmWriter {
    pub fn build(tree: &TokenTreeItem) -> Vec<String> {
        let mut result = Vec::new();
        let group = tree.get_name();

        if group.is_none() {
            return result;
        }

        let group = group.as_ref().unwrap().as_str();

        match group {
            "expression" => VmWriter::build_expression(tree),
            "term" => VmWriter::build_term(tree),
            value => panic!(format!("Unexpected token: {}", value)),
        }
    }

    fn build_expression(tree: &TokenTreeItem) -> Vec<String> {
        VmWriter::validate_name(tree, "expression");

        let mut result = Vec::new();

        let term = tree.get_nodes().get(0).unwrap();
        result.extend(VmWriter::build(term));

        let mut i = 1;

        while i < tree.get_nodes().len() {
            let term = tree.get_nodes().get(i + 1).unwrap();
            result.extend(VmWriter::build(term));

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

    fn build_term(tree: &TokenTreeItem) -> Vec<String> {
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
            _ => panic!("Unexpected term type"),
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

    #[test]
    fn build_epression_with_constants() {
        let mut tokenizer = Tokenizer::new("1 + 4 - 3");
        let tree = Expression::build(&mut tokenizer);

        let code: Vec<String> = VmWriter::build(&tree);

        assert_eq!(code.get(0).unwrap(), "push constant 1");
        assert_eq!(code.get(1).unwrap(), "push constant 4");
        assert_eq!(code.get(2).unwrap(), "+");
        assert_eq!(code.get(3).unwrap(), "push constant 3");
        assert_eq!(code.get(4).unwrap(), "-");
    }
}
