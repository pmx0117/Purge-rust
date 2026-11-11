use super::*;
use purge::text::fit_label;

/// Shape a fresh run at the final painted width. GPUI 0.2.2's measured
/// ellipsis reuses mutated run lengths across layout passes; a UTF-8 app name
/// can then be sliced inside a character by the Windows text backend.
pub(super) fn elided(text: impl Into<SharedString>, role: TextRole) -> impl IntoElement {
    label(text.into(), role, false)
}

pub(super) fn elided_middle(text: impl Into<SharedString>, role: TextRole) -> impl IntoElement {
    label(text.into(), role, true)
}

fn label(text: SharedString, role: TextRole, middle: bool) -> impl IntoElement {
    canvas(
        move |bounds, window, _| {
            let style = window.text_style().clone();
            let font_size = style.font_size.to_pixels(window.rem_size());
            let display = fit_label(&text, bounds.size.width.into(), middle, |candidate| {
                let run = style.to_run(candidate.len());
                let shaped = window.text_system().shape_line(
                    candidate.to_owned().into(),
                    font_size,
                    &[run],
                    None,
                );
                shaped.width.into()
            });
            if display.is_empty() {
                return None;
            }
            let run = style.to_run(display.len());
            let shaped = window
                .text_system()
                .shape_line(display.into(), font_size, &[run], None);
            let remaining = (bounds.size.width - shaped.width).max(px(0.));
            let offset = match style.text_align {
                TextAlign::Center => remaining / 2.,
                TextAlign::Right => remaining,
                _ => px(0.),
            };
            Some((shaped, offset))
        },
        move |bounds, shaped, window, cx| {
            let Some((line, offset)) = shaped else {
                return;
            };
            let _ = line.paint(
                bounds.origin + point(offset, px(0.)),
                px(role.line_height()),
                TextAlign::Left,
                None,
                window,
                cx,
            );
        },
    )
    .w_full()
    .h(px(role.line_height()))
}
