use super::*;
impl PurgeView {
    pub(in crate::ui) fn about(&self, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme;
        let mut information = div()
            .bg(rgb(t.card))
            .border_1()
            .border_color(rgb(t.border))
            .rounded(px(14.))
            .px(px(20.));
        for (index, (title, description, glyph)) in [
            (
                "文件去向",
                "清理的文件默认移入 Windows 回收站，可在清空前恢复。",
                "trash",
            ),
            (
                "隐私",
                "扫描在本机完成，设置和清理记录保存在本机，不上传文件。",
                "shield",
            ),
        ]
        .into_iter()
        .enumerate()
        {
            information = information.child(
                div()
                    .flex()
                    .items_start()
                    .gap(px(12.))
                    .py(px(18.))
                    .when(index == 0, |e| {
                        e.border_b(px(0.5)).border_color(rgb(t.border))
                    })
                    .child(div().pt(px(2.)).child(icon(glyph, t.muted, 16.)))
                    .child(
                        div()
                            .flex_1()
                            .child(div().font_weight(FontWeight::MEDIUM).child(title))
                            .child(
                                div()
                                    .mt(px(4.))
                                    .text_size(px(12.))
                                    .line_height(relative(1.6))
                                    .text_color(rgb(t.muted))
                                    .child(description),
                            ),
                    ),
            );
        }
        div()
            .id("about-scroll")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .child(
                div()
                    .max_w(px(620.))
                    .pt(px(20.))
                    .pb(px(24.))
                    .flex()
                    .flex_col()
                    .gap(px(24.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(20.))
                            .child(
                                img(ImageSource::Resource(Resource::Embedded(
                                    "purge.png".into(),
                                )))
                                .size(px(64.))
                                .rounded(px(14.)),
                            )
                            .child(
                                div()
                                    .child(
                                        div()
                                            .font_family("Onest")
                                            .text_size(px(28.))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child("Purge for Windows"),
                                    )
                                    .child(
                                        div()
                                            .mt(px(6.))
                                            .text_size(px(12.))
                                            .text_color(rgb(t.muted))
                                            .child(format!("版本 {}", env!("CARGO_PKG_VERSION"))),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(14.))
                            .text_color(rgb(t.muted))
                            .child("清理缓存，整理磁盘空间。"),
                    )
                    .child(information)
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap(px(12.))
                            .child(
                                button("about-recycle-bin", "打开回收站", "trash", t, false, true)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.handle(platform::open_recycle_bin(), cx)
                                    })),
                            )
                            .child(
                                button("data-dir", "本地数据目录", "folder", t, false, true)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        let result = this.controller.data_directory().and_then(
                                            |directory| platform::open_folder(&directory),
                                        );
                                        this.handle(result, cx);
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(rgb(t.faint))
                            .line_height(relative(1.8))
                            .child("Ctrl+R 重新扫描 · Ctrl+Shift+R 扫描全部 · Ctrl+1–5 切换分类"),
                    ),
            )
            .into_any_element()
    }
}
