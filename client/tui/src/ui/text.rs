use ratatui::text::Line;

/// Wrap text efficiently to match the exact visual lines it will take on screen.
pub fn wrap_str_to_lines<'b>(text: &str, max_width: usize) -> Vec<Line<'b>> {
    let wrapped = textwrap::wrap(text, max_width);
    wrapped
        .into_iter()
        .map(|w| Line::from(w.into_owned()))
        .collect()
}

/// Helper to wrap a slice of strings into a continuous vector of lines.
pub fn wrap_slice_to_lines(strs: &[String], max_width: usize) -> Vec<Line<'static>> {
    let mut visual_lines = Vec::new();
    for text in strs {
        let wrapped = textwrap::wrap(text, max_width);
        for w in wrapped {
            visual_lines.push(Line::from(w.into_owned()));
        }
    }
    visual_lines
}
