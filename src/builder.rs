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
    let line: Vec<&str> = line.split('/').collect();
    let line = line[0];
    String::from(line.trim())
}
