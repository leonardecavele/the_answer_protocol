use ratatui::layout::Rect;
use ratatui::text::Line;

/// Wrap text efficiently to match the exact visual lines it will take on screen.
pub fn wrap_str_to_lines<'a, 'b>(text: &'a str, max_width: usize) -> Vec<Line<'b>> {
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

/// Helper to compute a sub-area of the given size, centered inside `outer_area`.
pub fn centered_rect(outer_area: Rect, width: u16, height: u16) -> Rect {
    Rect {
        x: outer_area.x + outer_area.width.saturating_sub(width) / 2,
        y: outer_area.y + outer_area.height.saturating_sub(height) / 2,
        width,
        height,
    }
}

/// Helper to move a selection cursor through `count` items, wrapping at both ends.
pub fn move_index(current: usize, count: usize, forward: bool) -> usize {
    if count == 0 {
        return 0;
    }

    if forward {
        if current + 1 >= count { 0 } else { current + 1 }
    } else if current == 0 {
        count - 1
    } else {
        current - 1
    }
}

/// Helper to compute the centered sub-area that respects the image's original aspect ratio.
pub fn center_area_with_aspect_ratio(outer_area: Rect, img_width: u32, img_height: u32) -> Rect {
    let mut centered_area = outer_area;
    if img_height == 0 || outer_area.height == 0 {
        return centered_area;
    }

    let img_aspect = (img_width as f32) / (img_height as f32 / 2.0);
    let area_aspect = (outer_area.width as f32) / (outer_area.height as f32);

    if img_aspect > area_aspect {
        let render_height = (outer_area.width as f32 / img_aspect) as u16;
        if centered_area.height > render_height {
            centered_area.y += (centered_area.height.saturating_sub(render_height)) / 2;
            centered_area.height = render_height;
        }
    } else {
        let render_width = (outer_area.height as f32 * img_aspect) as u16;
        if centered_area.width > render_width {
            centered_area.x += (centered_area.width.saturating_sub(render_width)) / 2;
            centered_area.width = render_width;
        }
    }

    centered_area
}
