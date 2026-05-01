fn layout_text_lines(
    content: &str,
    max_width: f32,
    font_size: f32,
    word_wrap: bool,
    fit_to_width: bool,
) -> (f32, Vec<String>) {
    let mut effective_font_size = font_size.max(8.0);
    if fit_to_width && !word_wrap && max_width > 0.0 {
        let width = measure_text_line_width(content, effective_font_size);
        if width > max_width {
            effective_font_size = (effective_font_size * (max_width / width))
                .max(8.0)
                .min(effective_font_size);
        }
    }

    let lines = if word_wrap && max_width > 0.0 {
        wrap_text_lines(content, effective_font_size, max_width)
    } else {
        content.split('\n').map(|line| line.to_owned()).collect()
    };

    (
        effective_font_size,
        if lines.is_empty() {
            vec![String::new()]
        } else {
            lines
        },
    )
}

fn wrap_text_lines(content: &str, font_size: f32, max_width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in content.split('\n') {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }

        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            let candidate = if current.is_empty() {
                word.to_owned()
            } else {
                format!("{current} {word}")
            };
            if measure_text_line_width(&candidate, font_size) <= max_width {
                current = candidate;
                continue;
            }

            if !current.is_empty() {
                lines.push(current.clone());
                current.clear();
            }

            if measure_text_line_width(word, font_size) <= max_width {
                current = word.to_owned();
                continue;
            }

            let mut fragment = String::new();
            for ch in word.chars() {
                let candidate = format!("{fragment}{ch}");
                if !fragment.is_empty()
                    && measure_text_line_width(&candidate, font_size) > max_width
                {
                    lines.push(fragment.clone());
                    fragment.clear();
                }
                fragment.push(ch);
            }
            current = fragment;
        }

        if !current.is_empty() {
            lines.push(current);
        }
    }
    lines
}

fn measure_text_line_width(content: &str, font_size: f32) -> f32 {
    let effective_font_size = font_size.max(8.0);
    let pixel_size = effective_font_size / 7.0;
    let advance = 6.0 * pixel_size;
    content.chars().count() as f32 * advance
}

