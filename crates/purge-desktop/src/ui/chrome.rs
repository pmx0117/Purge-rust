use super::*;

pub(super) const HEIGHT: f32 = 46.;
const RESIZE_INSET: f32 = 6.;

pub(super) fn hairline(window: &Window) -> Pixels {
    px(1. / window.scale_factor())
}

pub(super) fn titlebar(t: Theme, window: &Window, cx: &mut App) -> Stateful<Div> {
    let divider_width = hairline(window);
    div()
        .id("window-titlebar")
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .h(px(HEIGHT))
        .flex()
        .occlude()
        .bg(rgb(t.base))
        .child(
            div()
                .w(px(style::SIDEBAR_WIDTH))
                .flex_shrink_0()
                .bg(rgb(t.sidebar))
                .border_r(divider_width)
                .border_color(rgb(t.chrome_edge))
                // Leave the top edge available for Windows resize hit testing.
                .pt(px(RESIZE_INSET))
                .child(
                    div()
                        .id("window-brand")
                        .window_control_area(WindowControlArea::Drag)
                        .on_mouse_down(MouseButton::Left, move_window)
                        .h(px(HEIGHT - RESIZE_INSET))
                        .pl(px(style::SPACE_16))
                        .flex()
                        .items_center()
                        .child(brand(t)),
                ),
        )
        .child(main_caption(t, window, divider_width, cx))
}

pub(super) fn brand(t: Theme) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(style::SPACE_8))
        .child(
            div()
                .size(px(purge::brand::SIZE))
                .flex_shrink_0()
                .rounded(px(purge::brand::RADIUS))
                .bg(rgb(t.brand))
                .flex()
                .items_center()
                .justify_center()
                .child(icon("purge-mark.svg", 0xffffff, purge::brand::GLYPH_SIZE)),
        )
        .child(div().type_style(TextRole::SectionTitle).child("Purge"))
}

pub(super) fn resize_handles(t: Theme, window: &Window) -> Div {
    let edge = px(4.);
    let corner = px(8.);
    div()
        .absolute()
        .inset_0()
        .child(
            div()
                .absolute()
                .inset_0()
                .border(hairline(window))
                .border_color(rgb(t.chrome_edge)),
        )
        .child(
            resize_handle("resize-top", ResizeEdge::Top, CursorStyle::ResizeUpDown)
                .top_0()
                .left(corner)
                .right(corner)
                .h(edge),
        )
        .child(
            resize_handle(
                "resize-bottom",
                ResizeEdge::Bottom,
                CursorStyle::ResizeUpDown,
            )
            .bottom_0()
            .left(corner)
            .right(corner)
            .h(edge),
        )
        .child(
            resize_handle(
                "resize-left",
                ResizeEdge::Left,
                CursorStyle::ResizeLeftRight,
            )
            .left_0()
            .top(corner)
            .bottom(corner)
            .w(edge),
        )
        .child(
            resize_handle(
                "resize-right",
                ResizeEdge::Right,
                CursorStyle::ResizeLeftRight,
            )
            .right_0()
            .top(corner)
            .bottom(corner)
            .w(edge),
        )
        .child(
            resize_handle(
                "resize-top-left",
                ResizeEdge::TopLeft,
                CursorStyle::ResizeUpLeftDownRight,
            )
            .top_0()
            .left_0()
            .size(corner),
        )
        .child(
            resize_handle(
                "resize-top-right",
                ResizeEdge::TopRight,
                CursorStyle::ResizeUpRightDownLeft,
            )
            .top_0()
            .right_0()
            .size(corner),
        )
        .child(
            resize_handle(
                "resize-bottom-left",
                ResizeEdge::BottomLeft,
                CursorStyle::ResizeUpRightDownLeft,
            )
            .bottom_0()
            .left_0()
            .size(corner),
        )
        .child(
            resize_handle(
                "resize-bottom-right",
                ResizeEdge::BottomRight,
                CursorStyle::ResizeUpLeftDownRight,
            )
            .bottom_0()
            .right_0()
            .size(corner),
        )
}

fn resize_handle(id: &'static str, edge: ResizeEdge, cursor: CursorStyle) -> Stateful<Div> {
    div()
        .id(id)
        .absolute()
        .cursor(cursor)
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            cx.stop_propagation();
            window.start_window_resize(edge);
        })
}

fn main_caption(t: Theme, window: &Window, divider_width: Pixels, _cx: &mut App) -> Div {
    div()
        .flex_1()
        .min_w_0()
        .pt(px(RESIZE_INSET))
        .border_b(divider_width)
        .border_color(rgb(t.chrome_edge))
        .child(
            div()
                .flex()
                .h_full()
                .items_center()
                .pr(px(style::SPACE_8))
                .gap(px(style::SPACE_2))
                .child(
                    div()
                        .id("window-drag")
                        .window_control_area(WindowControlArea::Drag)
                        .on_mouse_down(MouseButton::Left, move_window)
                        .h_full()
                        .flex_1(),
                )
                .when(!cfg!(target_os = "macos"), |e| {
                    e.child(caption_button(
                        t,
                        WindowControlArea::Min,
                        "caption-minimize",
                    ))
                    .child(caption_button(
                        t,
                        WindowControlArea::Max,
                        if window.is_maximized() {
                            "caption-restore"
                        } else {
                            "caption-maximize"
                        },
                    ))
                    .child(caption_button(
                        t,
                        WindowControlArea::Close,
                        "caption-close",
                    ))
                }),
        )
}

fn caption_button(t: Theme, area: WindowControlArea, glyph: &'static str) -> Stateful<Div> {
    let is_close = matches!(area, WindowControlArea::Close);
    let hover = if is_close {
        t.destructive().action
    } else {
        t.fill_hover
    };
    let pressed = if is_close {
        t.destructive().action_pressed
    } else {
        t.fill_pressed
    };
    div()
        .id(glyph)
        .group(glyph)
        .when(
            cfg!(windows) && !matches!(area, WindowControlArea::Min),
            |e| e.window_control_area(area),
        )
        .w(px(42.))
        .h(px(30.))
        .rounded(px(6.))
        .flex()
        .items_center()
        .justify_center()
        .hover(move |e| e.bg(rgb(hover)))
        .active(move |e| e.bg(rgb(pressed)))
        .when(
            !cfg!(windows) || matches!(area, WindowControlArea::Min),
            |e| {
                e.on_click(move |_, window, cx| {
                    if matches!(area, WindowControlArea::Min) {
                        shell::request(purge::lifecycle::Request::Minimize, cx);
                    } else if !cfg!(windows) {
                        match area {
                            WindowControlArea::Max => window.zoom_window(),
                            WindowControlArea::Close => {
                                shell::request(purge::lifecycle::Request::Quit, cx)
                            }
                            _ => {}
                        }
                    }
                })
            },
        )
        .child(icon(glyph, t.muted, 14.).group_hover(glyph, move |e| {
            e.text_color(rgb(if is_close { 0xffffff } else { t.text }))
        }))
}

fn move_window(event: &MouseDownEvent, window: &mut Window, _: &mut App) {
    if cfg!(windows) {
        return;
    }
    if event.click_count == 2 {
        window.zoom_window();
    } else {
        window.start_window_move();
    }
}
