#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DevConsoleSelection {
    pub start: usize,
    pub end: usize,
}

impl DevConsoleSelection {
    pub fn new(start: usize, end: usize) -> Option<Self> {
        if start == end {
            None
        } else if start < end {
            Some(Self { start, end })
        } else {
            Some(Self {
                start: end,
                end: start,
            })
        }
    }

    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevConsoleInputSnapshot {
    pub text: String,
    pub cursor: usize,
    pub selection: Option<DevConsoleSelection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DevConsoleInputBuffer {
    text: String,
    cursor: usize,
    selection_anchor: Option<usize>,
}

impl DevConsoleInputBuffer {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn snapshot(&self) -> DevConsoleInputSnapshot {
        DevConsoleInputSnapshot {
            text: self.text.clone(),
            cursor: self.cursor,
            selection: self.selection(),
        }
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.cursor = self.text.len();
        self.selection_anchor = None;
    }

    pub fn set_text_with_cursor(&mut self, text: impl Into<String>, cursor: usize) {
        self.text = text.into();
        self.cursor = clamp_to_char_boundary(&self.text, cursor.min(self.text.len()));
        self.selection_anchor = None;
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
        self.selection_anchor = None;
    }

    pub fn selection(&self) -> Option<DevConsoleSelection> {
        DevConsoleSelection::new(self.selection_anchor?, self.cursor)
    }

    pub fn selected_text(&self) -> Option<String> {
        let selection = self.selection()?;
        Some(self.text[selection.start..selection.end].to_owned())
    }

    pub fn insert_text(&mut self, text: &str) {
        self.delete_selection();
        self.text.insert_str(self.cursor, text);
        self.cursor += text.len();
        self.selection_anchor = None;
    }

    pub fn backspace(&mut self) {
        if self.delete_selection() {
            return;
        }

        if self.cursor == 0 {
            return;
        }

        let previous = previous_char_boundary(&self.text, self.cursor);
        self.text.replace_range(previous..self.cursor, "");
        self.cursor = previous;
        self.selection_anchor = None;
    }

    pub fn delete(&mut self) {
        if self.delete_selection() {
            return;
        }

        if self.cursor >= self.text.len() {
            return;
        }

        let next = next_char_boundary(&self.text, self.cursor);
        self.text.replace_range(self.cursor..next, "");
        self.selection_anchor = None;
    }

    pub fn move_left(&mut self, select: bool, word: bool) {
        let target = if word {
            previous_word_boundary(&self.text, self.cursor)
        } else {
            previous_char_boundary(&self.text, self.cursor)
        };
        self.move_cursor(target, select);
    }

    pub fn move_right(&mut self, select: bool, word: bool) {
        let target = if word {
            next_word_boundary(&self.text, self.cursor)
        } else {
            next_char_boundary(&self.text, self.cursor)
        };
        self.move_cursor(target, select);
    }

    pub fn move_home(&mut self, select: bool) {
        self.move_cursor(0, select);
    }

    pub fn move_end(&mut self, select: bool) {
        self.move_cursor(self.text.len(), select);
    }

    pub fn select_all(&mut self) {
        self.cursor = self.text.len();
        self.selection_anchor = Some(0);
    }

    pub fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    pub fn cut_selection(&mut self) -> Option<String> {
        let selection = self.selection()?;
        let value = self.text[selection.start..selection.end].to_owned();
        self.text.replace_range(selection.start..selection.end, "");
        self.cursor = selection.start;
        self.selection_anchor = None;
        Some(value)
    }

    fn delete_selection(&mut self) -> bool {
        let Some(selection) = self.selection() else {
            return false;
        };

        self.text.replace_range(selection.start..selection.end, "");
        self.cursor = selection.start;
        self.selection_anchor = None;
        true
    }

    fn move_cursor(&mut self, target: usize, select: bool) {
        let target = clamp_to_char_boundary(&self.text, target.min(self.text.len()));

        if select {
            if self.selection_anchor.is_none() {
                self.selection_anchor = Some(self.cursor);
            }
        } else {
            self.selection_anchor = None;
        }

        self.cursor = target;

        if self.selection_anchor == Some(self.cursor) {
            self.selection_anchor = None;
        }
    }
}

fn clamp_to_char_boundary(text: &str, mut index: usize) -> usize {
    index = index.min(text.len());
    while index > 0 && !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}

fn previous_char_boundary(text: &str, index: usize) -> usize {
    let index = clamp_to_char_boundary(text, index);
    text[..index]
        .char_indices()
        .last()
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn next_char_boundary(text: &str, index: usize) -> usize {
    let index = clamp_to_char_boundary(text, index);
    if index >= text.len() {
        return text.len();
    }

    text[index..]
        .char_indices()
        .nth(1)
        .map(|(offset, _)| index + offset)
        .unwrap_or(text.len())
}

fn previous_word_boundary(text: &str, index: usize) -> usize {
    let mut cursor = clamp_to_char_boundary(text, index);

    while cursor > 0 {
        let previous = previous_char_boundary(text, cursor);
        let Some(ch) = text[previous..cursor].chars().next() else {
            break;
        };
        if !ch.is_whitespace() {
            break;
        }
        cursor = previous;
    }

    while cursor > 0 {
        let previous = previous_char_boundary(text, cursor);
        let Some(ch) = text[previous..cursor].chars().next() else {
            break;
        };
        if ch.is_whitespace() {
            break;
        }
        cursor = previous;
    }

    cursor
}

fn next_word_boundary(text: &str, index: usize) -> usize {
    let mut cursor = clamp_to_char_boundary(text, index);

    while cursor < text.len() {
        let next = next_char_boundary(text, cursor);
        let Some(ch) = text[cursor..next].chars().next() else {
            break;
        };
        if ch.is_whitespace() {
            break;
        }
        cursor = next;
    }

    while cursor < text.len() {
        let next = next_char_boundary(text, cursor);
        let Some(ch) = text[cursor..next].chars().next() else {
            break;
        };
        if !ch.is_whitespace() {
            break;
        }
        cursor = next;
    }

    cursor
}

#[cfg(test)]
mod tests {
    use super::DevConsoleInputBuffer;

    #[test]
    fn inserts_text_at_cursor() {
        let mut input = DevConsoleInputBuffer::default();
        input.set_text("postfx.items ad blur");
        input.set_text_with_cursor(input.text().to_owned(), "postfx.items ad".len());
        input.insert_text("d");

        assert_eq!(input.text(), "postfx.items add blur");
    }

    #[test]
    fn backspace_removes_before_cursor() {
        let mut input = DevConsoleInputBuffer::default();
        input.set_text_with_cursor("abc", 2);
        input.backspace();

        assert_eq!(input.text(), "ac");
        assert_eq!(input.cursor(), 1);
    }

    #[test]
    fn delete_removes_after_cursor() {
        let mut input = DevConsoleInputBuffer::default();
        input.set_text_with_cursor("abc", 1);
        input.delete();

        assert_eq!(input.text(), "ac");
        assert_eq!(input.cursor(), 1);
    }

    #[test]
    fn shift_selection_replaces_inserted_text() {
        let mut input = DevConsoleInputBuffer::default();
        input.set_text("opacity");
        input.move_home(false);
        input.move_right(true, false);
        input.move_right(true, false);
        input.insert_text("vi");

        assert_eq!(input.text(), "viacity");
        assert_eq!(input.cursor(), 2);
    }

    #[test]
    fn ctrl_like_word_navigation_moves_by_word() {
        let mut input = DevConsoleInputBuffer::default();
        input.set_text("scene.entities list");
        input.move_left(false, true);

        assert_eq!(input.cursor(), "scene.entities ".len());
    }

    #[test]
    fn select_all_and_cut_selection() {
        let mut input = DevConsoleInputBuffer::default();
        input.set_text("scene stats");
        input.select_all();

        assert_eq!(input.selected_text().as_deref(), Some("scene stats"));
        assert_eq!(input.cut_selection().as_deref(), Some("scene stats"));
        assert_eq!(input.text(), "");
    }
}
