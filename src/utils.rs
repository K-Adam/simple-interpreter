#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SpanError {
    pub message: String,
    pub span: Span,
}

fn line_info(text: &str, index: usize) -> (usize, usize, &str) {
    let mut line = 1;
    let mut position = 1;
    let mut start = 0;
    let mut end = 0;

    for (i, c) in text.char_indices() {
        end += c.len_utf8();
        if c == '\n' {
            if i >= index {
                return (line, position, &text[start..end]);
            }
            line += 1;
            position = 1;
            start = end;
        } else if i < index {
            position += 1;
        }
    }

    (line, position, &text[start..end])
}

pub fn format_error(error: &SpanError, input: &str) -> String {
    let (line_number, char_number, line) = line_info(input, error.span.start);
    format!(
        "Error: {}, on line {} char {}:\n{}",
        error.message, line_number, char_number, line
    )
}

impl SpanError {
    pub fn new(message: String, start: usize, end: usize) -> SpanError {
        SpanError {
            message,
            span: Span { start, end },
        }
    }
}
