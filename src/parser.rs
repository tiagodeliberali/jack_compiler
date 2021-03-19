use crate::tokenizer::{Tokenizer, TokenType};


struct ClassVarDec {
    descriptor: String,
    var_type: String,
    names: Vec<String>,
}

impl ClassVarDec {
    pub fn build(tokenizer: &mut Tokenizer) -> Vec<ClassVarDec> {
        let mut result = Vec::new();

        while let Some(current_token) = tokenizer.peek_next() {
            match current_token.get_value().as_str() {
                "field" => result.push(ClassVarDec::build_field(tokenizer, "field")),
                "static" => result.push(ClassVarDec::build_field(tokenizer, "static")),
                _ => break,
            }
        }        

        result
    }

    fn build_field(tokenizer: &mut Tokenizer, descriptor: &str) -> ClassVarDec {
        let mut names: Vec<String> = Vec::new();
        
        tokenizer.consume(descriptor);
        let var_type = tokenizer.retrieve_any(Vec::from([TokenType::Identifier, TokenType::Keyword]));
        let name = tokenizer.retrieve_identifier();
        names.push(name);

        while let Some(token) = tokenizer.get_next() {
            match token.get_value().as_str() {
                "," => names.push(tokenizer.retrieve_identifier()),
                ";" => break,
                value => panic!(format!("Expecting ',' or ';', but retrieved '{}'", value))
            }
        }

        ClassVarDec {
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

}

impl SubroutineDec {
    pub fn build(tokenizer: &mut Tokenizer) -> Vec<SubroutineDec> {
        Vec::new()
    }
}

pub struct ClassNode {
    identifier: String,
    class_var_dec: Vec<ClassVarDec>,
    subroutine_dec: Vec<SubroutineDec>
}

impl ClassNode {
    pub fn build(tokenizer: &mut Tokenizer) -> ClassNode {

        tokenizer.consume("class");
        let identifier = tokenizer.retrieve_identifier();       
        tokenizer.consume("{");
        let class_var_dec = ClassVarDec::build(tokenizer);
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

    pub fn get_class_var_dec(&self) -> &Vec<ClassVarDec> {
        &self.class_var_dec
    }

    pub fn get_subroutine_dec(&self) -> &Vec<SubroutineDec> {
        &self.subroutine_dec
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

        let result = ClassVarDec::build(&mut tokenizer);

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

        let result = ClassVarDec::build(&mut tokenizer);

        assert_eq!(result.len(), 0);
    }
}