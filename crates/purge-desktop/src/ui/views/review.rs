use super::*;
impl PurgeView {
    pub(crate) fn checkbox(&self, checked: bool) -> Div {
        let t = self.theme;
        div()
            .size(px(16.))
            .flex_shrink_0()
            .rounded(px(4.))
            .border_1()
            .border_color(rgb(if checked { t.action } else { t.faint }))
            .bg(rgb(if checked { t.action } else { t.card }))
            .flex()
            .items_center()
            .justify_center()
            .when(checked, |e| e.child(icon("check", t.on_action, 10.)))
    }

    pub(crate) fn review_modal(&self, items: &[Item], cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme;
        let bytes = items.iter().map(Item::bytes).sum::<u64>();
        let entries = items.to_vec();
        let paths = uniform_list(
            "review-paths",
            entries.len(),
            cx.processor(move |_this, range: std::ops::Range<usize>, _, cx| {
                range
                    .map(|index| {
                        let item = &entries[index];
                        let path = item.path.clone();
                        div()
                            .id(("review-item", index))
                            .h(px(62.))
                            .py(px(8.))
                            .border_b(px(0.5))
                            .border_color(rgb(t.border))
                            .child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .gap(px(12.))
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .text_ellipsis()
                                            .child(item.name.clone()),
                                    )
                                    .child(format_bytes(item.bytes())),
                            )
                            .child(
                                div()
                                    .id(("review-path", index))
                                    .mt(px(4.))
                                    .text_size(px(11.))
                                    .line_height(px(14.))
                                    .text_color(rgb(t.muted))
                                    .line_clamp(2)
                                    .cursor_pointer()
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.handle(platform::reveal(&path), cx)
                                    }))
                                    .child(platform::display_path(&item.path)),
                            )
                    })
                    .collect()
            }),
        )
        .h(px((items.len() as f32 * 62.).min(248.)))
        .flex_shrink_0();
        div().id("review-scrim").absolute().inset_0().occlude().bg(rgba(t.overlay)).flex().items_center().justify_center().child(div().w(px(630.)).max_h(relative(0.9)).bg(rgb(t.card)).border_1().border_color(rgb(t.border)).rounded(px(14.)).p_7().flex().flex_col().gap_4()
            .child(div().flex().items_center().gap_3().child(icon("trash",t.text,23.)).child(div().text_size(px(21.)).font_weight(FontWeight::SEMIBOLD).child("移入回收站前，再看一眼")))
            .child(div().text_size(px(12.)).text_color(rgb(t.muted)).child(format!("共 {} 项，合计 {}。文件可以从 Windows 回收站恢复；清空回收站后才会释放空间。",items.len(),format_bytes(bytes))))
            .child(paths).child(div().id("acknowledge").focusable().tab_index(0).track_focus(&self.review_focus[0]).flex().items_start().gap_3().cursor_pointer().on_click(cx.listener(|this,_,_,cx|{this.controller.model.toggle_acknowledgement();cx.notify();})).child(self.checkbox(self.controller.model.acknowledged())).child(div().text_size(px(12.)).text_color(rgb(t.muted)).child("我已检查这些文件，并关闭了相关应用和开发任务。")))
            .child(div().flex().justify_end().gap_3().pt_2().child(button("cancel-clean","取消","x",t,false,true).track_focus(&self.review_focus[1]).on_click(cx.listener(|this,_,_,cx|{this.controller.model.review=None;cx.notify();})))
                .child(destructive_button("confirm-clean",format!("移入回收站 · {}",format_bytes(bytes)),t,self.controller.model.acknowledged()&&!items.is_empty()).track_focus(&self.review_focus[2]).on_click(cx.listener(|this,_,_,cx|this.confirm_clean(cx))))))
            .into_any_element()
    }
}
