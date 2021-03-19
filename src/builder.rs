pub fn build_content(content: String) -> String {
    let mut code_lines: Vec<String> = Vec::new();

    for line in content.lines() {
        let line = clean_line(line);

        if line.len() == 0 {
            continue;
        }

        code_lines.push(String::from(line));
    }

    code_lines.join("")
}

fn clean_line(line: &str) -> String {
    let line: Vec<&str> = line.split("//").collect();
    let line = line[0];
    let line: Vec<&str> = line.split("/*").collect();
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
        let token = clean_line("   test(x);    /** should test with coment */");

        assert_eq!("test(x);", token);
    }
}
