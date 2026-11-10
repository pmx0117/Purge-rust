use super::*;
impl PurgeView {
    pub(crate) fn header(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let subtitle = self.controller.model.header_subtitle(self.busy());
        div()
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_between()
            .gap_4()
            .pb(px(20.))
            .child(
                div()
                    .flex_1()
                    .child(
                        div()
                            .text_size(px(style::TITLE_SIZE))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(self.controller.model.page.title()),
                    )
                    .child(
                        div()
                            .mt_1()
                            .text_size(px(11.))
                            .text_color(rgb(t.muted))
                            .child(subtitle),
                    ),
            )
            .when(
                matches!(
                    self.controller.model.page,
                    Page::Overview | Page::Category(_)
                ),
                |e| {
                    e.child(
                        button(
                            "scan",
                            if self.controller.operation() == Some(Operation::Scan) {
                                "停止扫描"
                            } else if self.controller.model.page == Page::Overview {
                                "扫描全部"
                            } else {
                                "重新扫描"
                            },
                            "refresh",
                            t,
                            false,
                            !matches!(
                                self.controller.operation(),
                                Some(Operation::Cleanup | Operation::Duplicates)
                            ),
                        )
                        .on_click(cx.listener(|this, _, w, cx| {
                            if this.controller.operation() == Some(Operation::Scan) {
                                this.controller.cancel();
                            } else {
                                this.rescan(&Rescan, w, cx);
                            }
                        })),
                    )
                    .when(self.controller.model.page == Page::Overview, |e| {
                        e.child(
                            button(
                                "overview-clean",
                                "清理安全项目",
                                "trash",
                                t,
                                true,
                                !self.busy() && self.safe_bytes() > 0,
                            )
                            .on_click(cx.listener(|this, _, _, cx| this.review_safe(cx))),
                        )
                    })
                },
            )
            .when_some(
                match self.controller.model.page {
                    Page::Category(c) if c != Category::Applications => Some(c),
                    _ => None,
                },
                |e, c| {
                    let bytes = self
                        .controller
                        .model
                        .items
                        .iter()
                        .filter(|i| {
                            i.category() == c && self.controller.model.selected.contains(&i.id)
                        })
                        .map(Item::bytes)
                        .sum::<u64>();
                    e.child(
                        button(
                            "header-clean",
                            if bytes > 0 {
                                format!("清理所选 ({})", format_bytes(bytes))
                            } else {
                                "清理所选".into()
                            },
                            "trash",
                            t,
                            true,
                            !self.busy() && bytes > 0,
                        )
                        .on_click(cx.listener(move |this, _, _, cx| {
                            let items = this
                                .controller
                                .model
                                .items
                                .iter()
                                .filter(|i| {
                                    i.category() == c
                                        && this.controller.model.selected.contains(&i.id)
                                })
                                .cloned()
                                .collect();
                            this.review_items(items, cx);
                        })),
                    )
                },
            )
    }

    // OverviewView.diskSummary/categoriesCard/restOfDiskCard, with Windows data.
}
