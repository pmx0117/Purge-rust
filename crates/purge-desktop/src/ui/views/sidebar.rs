use super::*;
impl PurgeView {
    pub(crate) fn sidebar(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let mut nav = div().flex().flex_col().gap(px(2.)).px(px(12.));
        for (index, (section, page, glyph, title)) in [
            ("", Page::Overview, "overview", "概览"),
            (
                "清理",
                Page::Category(Category::Caches),
                "cache",
                "应用缓存",
            ),
            ("", Page::Category(Category::Developer), "dev", "开发工具"),
            (
                "检查",
                Page::Category(Category::LargeFiles),
                "file",
                "大文件",
            ),
            (
                "",
                Page::Category(Category::Applications),
                "trash",
                "应用卸载",
            ),
        ]
        .into_iter()
        .enumerate()
        {
            if !section.is_empty() {
                nav = nav.child(
                    div()
                        .pt(px(16.))
                        .pb(px(4.))
                        .px(px(8.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_size(px(11.))
                        .text_color(rgb(t.faint))
                        .child(section),
                );
            }
            let active = self.controller.model.page == page;
            nav = nav.child(
                div()
                    .id(("nav", index))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .h(px(30.))
                    .px(px(8.))
                    .rounded(px(8.))
                    .bg(rgb(if active { t.selection } else { t.sidebar }))
                    .text_color(rgb(if active { t.selection_text } else { t.muted }))
                    .cursor_pointer()
                    .hover(move |e| {
                        e.bg(rgb(if active {
                            t.selection_hover
                        } else {
                            t.nav_hover
                        }))
                    })
                    .on_click(cx.listener(move |this, _, w, cx| this.navigate(page, w, cx)))
                    .child(icon(
                        glyph,
                        if active { t.selection_text } else { t.faint },
                        16.,
                    ))
                    .child(
                        div()
                            .flex_1()
                            .font_weight(if active {
                                FontWeight::SEMIBOLD
                            } else {
                                FontWeight::MEDIUM
                            })
                            .child(title),
                    )
                    .when_some(
                        match page {
                            Page::Category(c) => Some(c),
                            _ => None,
                        },
                        |e, c| {
                            e.child(
                                div()
                                    .font_family("Segoe UI")
                                    .text_size(px(11.))
                                    .text_color(rgb(if active && !t.dark {
                                        t.selection_text
                                    } else {
                                        t.faint
                                    }))
                                    .child(if self.total(c) > 0 {
                                        format_bytes(self.total(c))
                                    } else {
                                        "—".into()
                                    }),
                            )
                        },
                    ),
            );
        }
        let mut utility = div().px(px(12.)).pb(px(8.)).flex().flex_col().gap(px(2.));
        for (index, (page, glyph, title)) in [
            (Page::History, "history", "清理历史"),
            (Page::Settings, "settings", "设置"),
            (Page::About, "info", "关于"),
        ]
        .into_iter()
        .enumerate()
        {
            let active = self.controller.model.page == page;
            utility = utility.child(
                div()
                    .id(("utility", index))
                    .h(px(30.))
                    .px(px(8.))
                    .rounded(px(8.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .bg(rgb(if active { t.selection } else { t.sidebar }))
                    .cursor_pointer()
                    .hover(move |e| {
                        e.bg(rgb(if active {
                            t.selection_hover
                        } else {
                            t.nav_hover
                        }))
                    })
                    .on_click(cx.listener(move |this, _, w, cx| this.navigate(page, w, cx)))
                    .child(icon(
                        glyph,
                        if active { t.selection_text } else { t.faint },
                        16.,
                    ))
                    .child(
                        div()
                            .font_weight(if active {
                                FontWeight::SEMIBOLD
                            } else {
                                FontWeight::MEDIUM
                            })
                            .text_color(rgb(if active { t.selection_text } else { t.muted }))
                            .child(title),
                    ),
            );
        }
        div()
            .w(px(style::SIDEBAR_WIDTH))
            .h_full()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .bg(rgb(t.sidebar))
            .border_r_1()
            .border_color(rgb(t.border))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .px(px(12.))
                    .pt(px(20.))
                    .pb(px(24.))
                    .child(
                        img(ImageSource::Resource(Resource::Embedded(
                            "purge.png".into(),
                        )))
                        .size(px(22.))
                        .rounded(px(4.)),
                    )
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_size(px(style::SECTION_SIZE))
                            .child("Purge"),
                    ),
            )
            .child(nav)
            .child(div().flex_1().min_h(px(16.)))
            .child(utility)
            .child(
                div().px(px(12.)).pb(px(12.)).child(
                    div()
                        .id("sidebar-trash")
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .p(px(12.))
                        .rounded(px(14.))
                        .bg(rgb(t.secondary))
                        .cursor_pointer()
                        .hover(move |e| e.bg(rgb(t.fill_hover)))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.handle(platform::open_recycle_bin(), cx)
                        }))
                        .child(icon("trash", t.muted, 14.))
                        .child(
                            div().text_size(px(12.)).text_color(rgb(t.muted)).child(
                                self.controller
                                    .model
                                    .trash_bytes
                                    .map(|bytes| format!("{} 已在回收站", format_bytes(bytes)))
                                    .unwrap_or_else(|| "打开 Windows 回收站".into()),
                            ),
                        ),
                ),
            )
    }
}
