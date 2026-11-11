use super::*;
impl PurgeView {
    pub(crate) fn checkbox(&self, checked: bool) -> Div {
        let t = self.theme;
        div()
            .size(px(16.))
            .flex_shrink_0()
            .rounded(px(4.))
            .border(px(0.5))
            .border_color(rgb(if checked { t.action } else { t.border_strong }))
            .bg(rgb(if checked {
                t.action
            } else if t.dark {
                t.border_strong
            } else {
                t.card
            }))
            .flex()
            .items_center()
            .justify_center()
            .when(checked, |e| e.child(icon("check", t.on_action, 10.)))
    }

    fn review_entry(&self, item: &Item, index: usize, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let path = item.path.clone();
        div().h(px(88.)).py(px(style::SPACE_4)).child(
            div()
                .size_full()
                .rounded(px(style::CARD_RADIUS))
                .bg(rgb(t.card))
                .border(px(0.5))
                .border_color(rgb(t.border))
                .px(px(style::ROW_PADDING))
                .py(px(style::SPACE_12))
                .flex()
                .items_center()
                .gap(px(style::SPACE_12))
                .child(icon(category_icon(item.category()), t.muted, 20.))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .gap(px(style::SPACE_2))
                        .child(
                            div()
                                .type_style(TextRole::Headline)
                                .overflow_hidden()
                                .child(elided(item.name.clone(), TextRole::Headline)),
                        )
                        .child(
                            div()
                                .id(("review-path", index))
                                .type_style(TextRole::Metadata)
                                .font_family(style::MONOSPACE)
                                .text_color(rgb(t.muted))
                                .overflow_hidden()
                                .cursor_pointer()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.handle(platform::reveal(&path), cx)
                                }))
                                .child(elided_middle(
                                    platform::display_path(&item.path),
                                    TextRole::Metadata,
                                )),
                        )
                        .child(
                            div()
                                .type_style(TextRole::Metadata)
                                .text_color(rgb(t.muted))
                                .overflow_hidden()
                                .child(elided(item.description.clone(), TextRole::Metadata)),
                        ),
                )
                .child(
                    div()
                        .type_style(TextRole::MetadataEmphasis)
                        .numeric()
                        .text_color(rgb(t.muted))
                        .child(format_bytes(item.bytes())),
                ),
        )
    }

    pub(crate) fn review_modal(&self, items: &[Item], cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme;
        let bytes = items.iter().map(Item::bytes).sum::<u64>();
        let entries = items.to_vec();
        let paths = uniform_list(
            "review-paths",
            entries.len(),
            cx.processor(move |this, range: std::ops::Range<usize>, _, cx| {
                range
                    .map(|index| this.review_entry(&entries[index], index, cx))
                    .collect()
            }),
        )
        .h(px((items.len() as f32 * 88.).min(264.)))
        .min_h(px(88.));
        div()
            .id("review-scrim")
            .absolute()
            .inset_0()
            .occlude()
            .bg(rgba(t.overlay))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(560.))
                    .max_h(relative(0.9))
                    .bg(rgb(t.base))
                    .border(px(0.5))
                    .border_color(rgb(t.border))
                    .rounded(px(style::CARD_RADIUS))
                    .p(px(style::SPACE_24))
                    .flex()
                    .flex_col()
                    .gap(px(style::SPACE_16))
                    .child(
                        div()
                            .flex_shrink_0()
                            .type_style(TextRole::PageTitle)
                            .child("将所选项目移入回收站？"),
                    )
                    .child(
                        div()
                            .flex_shrink_0()
                            .type_style(TextRole::Body)
                            .text_color(rgb(t.muted))
                            .child("只会处理你选中的文件，可在清空系统回收站前恢复。"),
                    )
                    .child(paths)
                    .child(
                        div()
                            .id("acknowledge")
                            .focusable()
                            .tab_index(0)
                            .track_focus(&self.review_focus[0])
                            .flex_shrink_0()
                            .flex()
                            .items_start()
                            .gap(px(style::SPACE_12))
                            .cursor_pointer()
                            .rounded(px(style::FIELD_RADIUS))
                            .p(px(style::SPACE_4))
                            .border(px(0.5))
                            .border_color(rgba(0))
                            .focus(move |e| e.border_color(rgb(t.border_strong)))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.controller.model.toggle_acknowledgement();
                                cx.notify();
                            }))
                            .child(self.checkbox(self.controller.model.acknowledged()))
                            .child(
                                div()
                                    .type_style(TextRole::Callout)
                                    .text_color(rgb(t.muted))
                                    .child("我已检查这些文件，并关闭相关应用和开发任务。"),
                            ),
                    )
                    .child(
                        div()
                            .flex_shrink_0()
                            .flex()
                            .items_center()
                            .gap(px(style::SPACE_12))
                            .pt(px(style::SPACE_8))
                            .child(
                                div()
                                    .flex_1()
                                    .type_style(TextRole::MetadataEmphasis)
                                    .numeric()
                                    .text_color(rgb(t.muted))
                                    .child(format!("{} 项 · {}", items.len(), format_bytes(bytes))),
                            )
                            .child(
                                button("cancel-clean", "取消", "x", t, false, true)
                                    .track_focus(&self.review_focus[1])
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.controller.model.review = None;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                destructive_button(
                                    "confirm-clean",
                                    "移入回收站",
                                    t,
                                    self.controller.model.acknowledged() && !items.is_empty(),
                                )
                                .track_focus(&self.review_focus[2])
                                .on_click(cx.listener(|this, _, _, cx| this.confirm_clean(cx))),
                            ),
                    ),
            )
            .into_any_element()
    }
}
