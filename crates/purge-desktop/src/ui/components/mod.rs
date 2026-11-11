use super::*;

pub(crate) fn icon(name: &'static str, color: u32, size: f32) -> Svg {
    svg()
        .path(name)
        .size(px(size))
        .text_color(rgb(color))
        .flex_shrink_0()
}
pub(crate) fn category_icon(c: Category) -> &'static str {
    match c {
        Category::Caches => "cache",
        Category::Developer => "dev",
        Category::LargeFiles => "file",
        Category::Applications => "apps",
    }
}

pub(crate) fn filled_icon(name: &'static str) -> &'static str {
    match name {
        "overview" => "overview-filled",
        "cache" => "cache-filled",
        "dev" => "dev-filled",
        "file" => "file-filled",
        "apps" => "apps-filled",
        _ => name,
    }
}

pub(crate) fn value_control(
    id: impl Into<ElementId>,
    text: impl Into<SharedString>,
    t: Theme,
    enabled: bool,
) -> Stateful<Div> {
    div()
        .id(id)
        .h(px(28.))
        .min_w(px(130.))
        .px(px(style::SPACE_12))
        .rounded(px(style::FIELD_RADIUS))
        .border(px(0.5))
        .border_color(rgb(t.border))
        .bg(rgb(t.fill))
        .type_style(TextRole::Body)
        .text_color(rgb(t.text))
        .flex()
        .items_center()
        .gap(px(style::SPACE_12))
        .opacity(if enabled { 1. } else { 0.45 })
        .when(enabled, |e| {
            e.focusable()
                .tab_index(0)
                .cursor_pointer()
                .hover(move |e| e.bg(rgb(t.fill_hover)))
                .focus(move |e| e.border_color(rgb(t.accent)))
        })
        .child(div().flex_1().child(text.into()))
        .child(icon("chevrons", t.muted, 12.))
}

pub(crate) fn toggle(id: impl Into<ElementId>, checked: bool, t: Theme) -> Stateful<Div> {
    div()
        .id(id)
        .w(px(52.))
        .h(px(24.))
        .p(px(style::SPACE_2))
        .rounded_full()
        .bg(rgb(if checked { t.toggle_on } else { t.toggle_off }))
        .flex()
        .items_center()
        .when(checked, |e| e.justify_end())
        .focusable()
        .tab_index(0)
        .cursor_pointer()
        .focus(move |e| e.border(px(0.5)).border_color(rgb(t.accent)))
        .child(
            div()
                .w(px(32.))
                .h(px(20.))
                .rounded_full()
                .bg(rgb(t.toggle_thumb))
                .shadow_sm(),
        )
}
pub(crate) fn tag(text: impl Into<SharedString>, color: u32, bg: u32) -> Div {
    div()
        .px(px(style::SPACE_6))
        .py(px(style::SPACE_2))
        .rounded(px(style::BADGE_RADIUS))
        .border(px(0.5))
        .border_color(rgba((color << 8) | 0x38))
        .bg(rgb(bg))
        .text_color(rgb(color))
        .type_style(TextRole::MetadataEmphasis)
        .child(text.into())
}
pub(crate) fn button(
    id: impl Into<ElementId>,
    text: impl Into<SharedString>,
    glyph: &'static str,
    t: Theme,
    primary: bool,
    enabled: bool,
) -> Stateful<Div> {
    let fg = if primary { t.on_action } else { t.text };
    let background = if primary { t.action } else { t.secondary };
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .gap(px(style::SPACE_6))
        .h(px(style::BUTTON_HEIGHT))
        .px(px(style::BUTTON_PADDING))
        .rounded_full()
        .when(!primary, |e| {
            e.border_1().border_color(rgb(t.border_strong))
        })
        .bg(rgb(background))
        .text_color(rgb(fg))
        .type_style(TextRole::Headline)
        .whitespace_nowrap()
        .flex_shrink_0()
        .opacity(if enabled { 1. } else { 0.45 })
        .when(enabled, |e| {
            e.focusable()
                .tab_index(0)
                .focus(move |s| s.border_1().border_color(rgb(t.border_strong)))
                .cursor_pointer()
                .hover(move |s| {
                    s.bg(rgb(if primary {
                        t.action_hover
                    } else {
                        t.fill_hover
                    }))
                })
                .active(move |s| {
                    s.bg(rgb(if primary {
                        t.action_pressed
                    } else {
                        t.fill_pressed
                    }))
                })
        })
        .child(icon(glyph, fg, 14.))
        .child(text.into())
}

pub(crate) fn destructive_button(
    id: impl Into<ElementId>,
    text: impl Into<SharedString>,
    theme: Theme,
    enabled: bool,
) -> Stateful<Div> {
    button(id, text, "trash", theme.destructive(), true, enabled)
}
