use ratatui::layout::Rect;

/// Helper to compute a sub-area of the given size, centered inside `outer_area`.
pub fn centered_rect(outer_area: Rect, width: u16, height: u16) -> Rect {
    Rect {
        x: outer_area.x + outer_area.width.saturating_sub(width) / 2,
        y: outer_area.y + outer_area.height.saturating_sub(height) / 2,
        width,
        height,
    }
}

/// Helper to compute a percentage of a terminal dimension without overflowing `u16`.
pub fn percent_of(value: u16, percent: u16) -> u16 {
    (u32::from(value) * u32::from(percent) / 100) as u16
}

/// Helper to compute a centered sub-area sized as a percentage of `outer_area`.
pub fn centered_rect_percent(outer_area: Rect, width_percent: u16, height_percent: u16) -> Rect {
    centered_rect(
        outer_area,
        percent_of(outer_area.width, width_percent),
        percent_of(outer_area.height, height_percent),
    )
}

/// Helper to compute the centered sub-area of `outer_area` matching the given width/height ratio.
pub fn fit_area(outer_area: Rect, aspect: f32) -> Rect {
    let mut fitted_area = outer_area;
    if aspect <= 0.0 || outer_area.height == 0 {
        return fitted_area;
    }

    let area_aspect = (outer_area.width as f32) / (outer_area.height as f32);

    if aspect > area_aspect {
        let render_height = (outer_area.width as f32 / aspect) as u16;
        if fitted_area.height > render_height {
            fitted_area.y += (fitted_area.height.saturating_sub(render_height)) / 2;
            fitted_area.height = render_height;
        }
    } else {
        let render_width = (outer_area.height as f32 * aspect) as u16;
        if fitted_area.width > render_width {
            fitted_area.x += (fitted_area.width.saturating_sub(render_width)) / 2;
            fitted_area.width = render_width;
        }
    }

    fitted_area
}
