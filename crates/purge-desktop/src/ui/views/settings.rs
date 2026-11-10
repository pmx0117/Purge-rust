use super::*;
impl PurgeView {
    // AppearanceOptionButton: 64 x 44 thumbnails, 6pt corners and selected rim.
    fn appearance_picker(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let mut options = div().flex().items_center().gap(px(20.));
        for (index, (dark, title, asset)) in [
            (false, "浅色", "appearance-light"),
            (true, "深色", "appearance-dark"),
        ]
        .into_iter()
        .enumerate()
        {
            let selected = self.theme.dark == dark;
            options = options.child(
                div()
                    .id(("appearance", index))
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(8.))
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.controller.set_appearance(dark);
                        this.theme = Theme::new(dark);
                        cx.notify();
                    }))
                    .child(
                        div()
                            .p(px(3.))
                            .rounded(px(8.))
                            .border(px(2.))
                            .border_color(if selected {
                                rgb(if t.dark { t.text } else { t.accent })
                            } else {
                                rgba(0)
                            })
                            .child(
                                img(ImageSource::Resource(Resource::Embedded(asset.into())))
                                    .w(px(64.))
                                    .h(px(44.))
                                    .rounded(px(6.)),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(11.))
                            .font_weight(if selected {
                                FontWeight::SEMIBOLD
                            } else {
                                FontWeight::NORMAL
                            })
                            .text_color(rgb(if selected { t.text } else { t.muted }))
                            .child(title),
                    ),
            );
        }
        div()
            .bg(rgb(if t.dark { t.fill } else { t.card }))
            .rounded(px(14.))
            .border(px(0.5))
            .border_color(rgb(t.border))
            .p(px(16.))
            .flex()
            .items_center()
            .gap(px(24.))
            .child(
                div()
                    .flex_1()
                    .child(div().font_weight(FontWeight::SEMIBOLD).child("外观"))
                    .child(
                        div()
                            .mt(px(4.))
                            .text_size(px(12.))
                            .text_color(rgb(t.muted))
                            .child("选择浅色或深色。"),
                    ),
            )
            .child(options)
    }
    fn setting_row(&self, title: &str, subtitle: &str, control: impl IntoElement) -> Div {
        let t = self.theme;
        div()
            .flex()
            .items_center()
            .gap_5()
            .py_4()
            .border_b(px(0.5))
            .border_color(rgb(t.border))
            .child(
                div()
                    .flex_1()
                    .child(
                        div()
                            .font_weight(FontWeight::MEDIUM)
                            .child(title.to_string()),
                    )
                    .child(
                        div()
                            .mt_1()
                            .text_size(px(12.))
                            .text_color(rgb(t.muted))
                            .child(subtitle.to_string()),
                    ),
            )
            .child(control)
    }
    pub(in crate::ui) fn settings_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme;
        let sizes = [5, 25, 100, 500, 1024];
        let days = [0, 30, 90, 180, 365];
        let mut content = div()
            .flex()
            .flex_col()
            .gap_6()
            .child(self.appearance_picker(cx))
            .child(
                div()
                    .bg(rgb(t.card))
                    .border(px(0.5))
                    .border_color(rgb(t.border))
                    .rounded(px(14.))
                    .px_5()
                    .child(
                        self.setting_row(
                            "启动时扫描",
                            "启动后自动检查各分类；不会自动清理。",
                            button(
                                "auto-scan",
                                if self.controller.model.settings.scan_on_start {
                                    "已开启"
                                } else {
                                    "已关闭"
                                },
                                if self.controller.model.settings.scan_on_start {
                                    "check"
                                } else {
                                    "x"
                                },
                                t,
                                false,
                                true,
                            )
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.controller.model.settings.scan_on_start =
                                    !this.controller.model.settings.scan_on_start;
                                this.save();
                                cx.notify();
                            })),
                        ),
                    )
                    .child(
                        self.setting_row(
                            "大文件阈值",
                            "仅列出大于该大小的个人文件，修改后需重新扫描。",
                            button(
                                "threshold",
                                format!(
                                    "{} MB",
                                    self.controller.model.settings.scan.minimum_file_mb
                                ),
                                "file",
                                t,
                                false,
                                !self.busy(),
                            )
                            .on_click(cx.listener(
                                move |this, _, _, cx| {
                                    if this.busy() {
                                        return;
                                    }
                                    let i = sizes
                                        .iter()
                                        .position(|v| {
                                            *v == this
                                                .controller
                                                .model
                                                .settings
                                                .scan
                                                .minimum_file_mb
                                        })
                                        .unwrap_or(1);
                                    this.controller.model.settings.scan.minimum_file_mb =
                                        sizes[(i + 1) % sizes.len()];
                                    this.save();
                                    cx.notify();
                                },
                            )),
                        ),
                    )
                    .child(
                        self.setting_row(
                            "项目闲置时间",
                            "从项目内最新修改时间判断；项目产物始终需要确认。",
                            button(
                                "stale",
                                if self.controller.model.settings.scan.stale_project_days == 0 {
                                    "显示全部".into()
                                } else {
                                    format!(
                                        "{} 天",
                                        self.controller.model.settings.scan.stale_project_days
                                    )
                                },
                                "history",
                                t,
                                false,
                                !self.busy(),
                            )
                            .on_click(cx.listener(
                                move |this, _, _, cx| {
                                    if this.busy() {
                                        return;
                                    }
                                    let i = days
                                        .iter()
                                        .position(|v| {
                                            *v == this
                                                .controller
                                                .model
                                                .settings
                                                .scan
                                                .stale_project_days
                                        })
                                        .unwrap_or(0);
                                    this.controller.model.settings.scan.stale_project_days =
                                        days[(i + 1) % days.len()];
                                    this.save();
                                    cx.notify();
                                },
                            )),
                        ),
                    ),
            );
        for (exclusion, title, subtitle, paths) in [
            (
                false,
                "开发项目目录",
                "只识别带有对应项目清单的 node_modules、target、虚拟环境等产物。",
                &self.controller.model.settings.scan.project_roots,
            ),
            (
                true,
                "排除目录",
                "这些目录及其子目录不会被扫描，也不能被清理。",
                &self.controller.model.settings.scan.exclusions,
            ),
        ] {
            let mut group = div()
                .bg(rgb(t.card))
                .border(px(0.5))
                .border_color(rgb(t.border))
                .rounded(px(14.))
                .p_5()
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .items_center()
                        .gap_4()
                        .child(
                            div()
                                .flex_1()
                                .child(div().font_weight(FontWeight::SEMIBOLD).child(title))
                                .child(
                                    div()
                                        .text_size(px(12.))
                                        .text_color(rgb(t.muted))
                                        .mt_1()
                                        .child(subtitle),
                                ),
                        )
                        .child(
                            button(
                                if exclusion {
                                    "add-exclusion"
                                } else {
                                    "add-project"
                                },
                                "添加目录",
                                "plus",
                                t,
                                false,
                                !self.busy(),
                            )
                            .on_click(cx.listener(
                                move |this, _, _, cx| {
                                    if !this.busy() {
                                        this.choose_folder(exclusion, cx);
                                    }
                                },
                            )),
                        ),
                );
            if paths.is_empty() {
                group = group.child(
                    div()
                        .pt_4()
                        .text_color(rgb(t.faint))
                        .text_size(px(12.))
                        .child("尚未添加目录"),
                );
            }
            for (index, path) in paths.iter().enumerate() {
                group = group.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .pt_4()
                        .child(icon("folder", t.faint, 14.))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .text_ellipsis()
                                .text_size(px(12.))
                                .text_color(rgb(t.muted))
                                .child(platform::display_path(path)),
                        )
                        .child(
                            div()
                                .id((
                                    if exclusion {
                                        "remove-exclusion"
                                    } else {
                                        "remove-project"
                                    },
                                    index,
                                ))
                                .cursor_pointer()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    if this.busy() {
                                        return;
                                    }
                                    let paths = if exclusion {
                                        &mut this.controller.model.settings.scan.exclusions
                                    } else {
                                        &mut this.controller.model.settings.scan.project_roots
                                    };
                                    if index < paths.len() {
                                        paths.remove(index);
                                    }
                                    this.save();
                                    cx.notify();
                                }))
                                .child(icon("x", t.faint, 11.)),
                        ),
                );
            }
            content = content.child(group);
        }
        content = content.child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .text_size(px(12.))
                .text_color(rgb(t.muted))
                .child(icon("shield", t.safe, 16.))
                .child("Purge 不扫描 Windows 系统目录，不请求管理员权限，也不上传扫描数据。"),
        );
        div()
            .id("settings-scroll")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .pb_5()
            .child(content)
            .into_any_element()
    }
}
