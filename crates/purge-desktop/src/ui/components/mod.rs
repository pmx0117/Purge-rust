use super::*;

pub(crate) fn icon(name: &'static str, color: u32, size: f32) -> Svg {
    svg()
        .path(name)
        .size(px(size))
        .text_color(rgb(color))
        .flex_shrink_0()
}
pub(crate) trait CategoryColor {
    fn color(self) -> u32;
}
impl CategoryColor for Category {
    fn color(self) -> u32 {
        match self {
            Category::Caches => 0x6cc3f0,
            Category::Developer => 0x3a94d0,
            Category::LargeFiles => 0xf0b020,
            Category::Applications => 0xe8772a,
        }
    }
}

pub(crate) fn category_icon(c: Category) -> &'static str {
    match c {
        Category::Caches => "cache",
        Category::Developer => "dev",
        Category::LargeFiles => "file",
        Category::Applications => "apps",
    }
}
pub(crate) fn tag(text: impl Into<SharedString>, color: u32, bg: u32) -> Div {
    div()
        .px(px(6.))
        .py(px(2.))
        .rounded(px(style::BADGE_RADIUS))
        .border(px(0.5))
        .border_color(rgba((color << 8) | 0x38))
        .bg(rgb(bg))
        .text_color(rgb(color))
        .text_size(px(style::METADATA_SIZE))
        .font_weight(FontWeight::MEDIUM)
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
    let disabled = !enabled && !t.dark;
    let fg = if disabled {
        t.disabled_text
    } else if primary {
        t.on_action
    } else {
        t.text
    };
    let background = if disabled {
        t.disabled_surface
    } else if primary {
        t.action
    } else {
        t.secondary
    };
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .gap_2()
        .h(px(style::BUTTON_HEIGHT))
        .px(px(style::BUTTON_PADDING))
        .rounded_full()
        .border_1()
        .border_color(rgb(if disabled {
            t.border
        } else if primary {
            t.action
        } else {
            t.border_strong
        }))
        .bg(rgb(background))
        .text_color(rgb(fg))
        .text_size(px(style::BODY_SIZE))
        .font_weight(FontWeight::SEMIBOLD)
        .opacity(if enabled || !t.dark { 1. } else { 0.45 })
        .when(enabled, |e| {
            e.focusable()
                .tab_index(0)
                .focus(move |s| s.border_color(rgb(t.accent)))
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
    let mut tint = theme;
    tint.action = if theme.dark { 0xd9372d } else { 0xd1312a };
    tint.action_hover = if theme.dark { 0xe2453b } else { 0xbc2b25 };
    tint.action_pressed = if theme.dark { 0xc4302a } else { 0xa92620 };
    tint.on_action = 0xffffff;
    button(id, text, "trash", tint, true, enabled)
}
