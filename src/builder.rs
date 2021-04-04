use regex::Regex;

pub fn build_content(content: String) -> String {
    let mut code_lines: Vec<String> = Vec::new();

    let content = clear_special_coments(content);

    for line in content.lines() {
        let line = clean_line(line);

        if line.len() == 0 {
            continue;
        }

        code_lines.push(String::from(line));
    }

    code_lines.join("")
}

fn clear_special_coments(content: String) -> String {
    let re = Regex::new(r"/\*(.|\r\n|\r|\n)*?\*/").unwrap();
    re.replace_all(&content.as_str(), "").to_string()
}

fn clean_line(line: &str) -> String {
    let line: Vec<&str> = line.split("//").collect();
    let line = line[0];
    String::from(line.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_line_with_spaces() {
        let token = clean_line("   test(x);    ");

        assert_eq!("test(x);", token);
    }

    #[test]
    fn clean_line_with_simple_comment() {
        let token = clean_line("   test(x);    // should test with coment");

        assert_eq!("test(x);", token);
    }

    #[test]
    fn clean_line_with_special_comment() {
        let clean_code = clear_special_coments(String::from(
            "   test(x);    /** should test with coment */",
        ));
        let token = clean_line(&clean_code.as_str());

        assert_eq!("test(x);", token);
    }

    #[test]
    fn test_clear_special_coments() {
        let clean_code = clear_special_coments(String::from(
            "   test(x);    /** should test with coment \r\n * test \r\n * another test \r\n * end test */ \r\n antoherTest();"));

        let token = clean_line(&clean_code.as_str());

        assert_eq!("test(x);     \r\n antoherTest();", token);
    }
}
