fn lifecycle_source_from_script(source: &str) -> String {
    let mut output = String::new();
    let mut capturing_fn = false;
    let mut brace_depth = 0_i32;
    let mut saw_fn_body = false;

    for line in source.lines() {
        let trimmed = line.trim_start();

        if capturing_fn {
            output.push_str(line);
            output.push('\n');
            update_brace_depth(line, &mut brace_depth, &mut saw_fn_body);
            if saw_fn_body && brace_depth <= 0 {
                capturing_fn = false;
                brace_depth = 0;
                saw_fn_body = false;
                output.push('\n');
            }
            continue;
        }

        if trimmed.starts_with("import ") || trimmed.starts_with("const ") {
            output.push_str(line);
            output.push('\n');
            continue;
        }

        if trimmed.starts_with("fn ") {
            capturing_fn = true;
            output.push_str(line);
            output.push('\n');
            update_brace_depth(line, &mut brace_depth, &mut saw_fn_body);
            if saw_fn_body && brace_depth <= 0 {
                capturing_fn = false;
                brace_depth = 0;
                saw_fn_body = false;
                output.push('\n');
            }
        }
    }

    output
}

fn update_brace_depth(line: &str, brace_depth: &mut i32, saw_fn_body: &mut bool) {
    for ch in line.chars() {
        match ch {
            '{' => {
                *saw_fn_body = true;
                *brace_depth += 1;
            }
            '}' => *brace_depth -= 1,
            _ => {}
        }
    }
}

