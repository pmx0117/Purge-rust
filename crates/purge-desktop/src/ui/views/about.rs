use super::*;
impl PurgeView {
    pub(in crate::ui) fn about(&self, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme;
        let mut information = div()
            .bg(rgb(t.card))
            .border_1()
            .border_color(rgb(t.border))
            .rounded(px(style::CARD_RADIUS))
            .px(px(20.));
        for (index, (title, description, glyph)) in [
            (
                "文件去向",
                "清理的文件默认移入系统回收站，可在清空前恢复。",
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
                    .gap(px(style::SPACE_12))
                    .py(px(18.))
                    .when(index == 0, |e| {
                        e.border_b(px(0.5)).border_color(rgb(t.border))
                    })
                    .child(
                        div()
                            .pt(px(style::SPACE_2))
                            .child(icon(glyph, t.muted, 16.)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .child(div().font_weight(FontWeight::MEDIUM).child(title))
                            .child(
                                div()
                                    .mt(px(style::SPACE_4))
                                    .type_style(TextRole::Callout)
                                    .line_height(px(18.))
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
                    .pb(px(style::SPACE_24))
                    .flex()
                    .flex_col()
                    .gap(px(style::SPACE_24))
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
                                .rounded(px(style::CARD_RADIUS)),
                            )
                            .child(
                                div()
                                    .child(
                                        div()
                                            .font_family("Onest")
                                            .type_style(TextRole::Title)
                                            .child(format!(
                                                "Purge for {}",
                                                purge_engine::platform::OperatingSystem::current()
                                                    .label()
                                            )),
                                    )
                                    .child(
                                        div()
                                            .mt(px(style::SPACE_6))
                                            .type_style(TextRole::Callout)
                                            .text_color(rgb(t.muted))
                                            .child(format!("版本 {}", env!("CARGO_PKG_VERSION"))),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .type_style(TextRole::Body)
                            .text_color(rgb(t.muted))
                            .child("清理缓存，整理磁盘空间。"),
                    )
                    .child(information)
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap(px(style::SPACE_12))
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
                            .type_style(TextRole::Metadata)
                            .text_color(rgb(t.faint))
                            .line_height(relative(1.8))
                            .child("Ctrl+R 重新扫描 · Ctrl+Shift+R 扫描全部 · Ctrl+1–5 切换分类"),
                    ),
            )
            .into_any_element()
    }
}
