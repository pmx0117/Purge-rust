use super::*;

impl PurgeView {
    fn settings_section(&self, title: &str, content: Div) -> Div {
        let t = self.theme;
        div()
            .flex()
            .flex_col()
            .gap(px(style::SPACE_6))
            .child(
                div()
                    .px(px(style::SPACE_4))
                    .type_style(TextRole::MetadataEmphasis)
                    .text_color(rgb(t.muted))
                    .child(title.to_owned()),
            )
            .child(
                content
                    .rounded(px(style::CARD_RADIUS))
                    .bg(rgb(t.fill))
                    .border(px(0.5))
                    .border_color(rgb(t.border))
                    .overflow_hidden(),
            )
    }

    fn setting_divider(&self) -> Div {
        div()
            .mx(px(style::SPACE_16))
            .h(px(0.5))
            .bg(rgb(self.theme.border))
    }

    fn setting_row(&self, title: &str, subtitle: &str, control: impl IntoElement) -> Div {
        let t = self.theme;
        div()
            .flex()
            .items_center()
            .gap(px(style::SPACE_24))
            .px(px(style::SPACE_16))
            .py(px(style::SPACE_16))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(div().type_style(TextRole::RowTitle).child(title.to_owned()))
                    .when(!subtitle.is_empty(), |e| {
                        e.child(
                            div()
                                .mt(px(style::SPACE_2))
                                .type_style(TextRole::Callout)
                                .text_color(rgb(t.muted))
                                .child(subtitle.to_owned()),
                        )
                    }),
            )
            .child(control)
    }

    fn appearance_picker(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let options = div()
            .flex()
            .items_center()
            .gap(px(style::SPACE_16))
            .children(
                [
                    (false, "浅色", "appearance-light"),
                    (true, "深色", "appearance-dark"),
                ]
                .into_iter()
                .enumerate()
                .map(|(index, (dark, title, asset))| {
                    let selected = t.dark == dark;
                    div()
                        .id(("appearance", index))
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(style::SPACE_6))
                        .cursor_pointer()
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.controller.set_appearance(dark);
                            this.theme = Theme::new(dark);
                            cx.notify();
                        }))
                        .child(
                            div()
                                .p(px(style::SPACE_2))
                                .rounded(px(style::FIELD_RADIUS))
                                .border(px(2.))
                                .border_color(if selected { rgb(t.action) } else { rgba(0) })
                                .child(
                                    img(ImageSource::Resource(Resource::Embedded(asset.into())))
                                        .w(px(64.))
                                        .h(px(44.))
                                        .rounded(px(6.)),
                                ),
                        )
                        .child(
                            div()
                                .type_style(TextRole::Metadata)
                                .when(selected, |e| e.font_weight(FontWeight::SEMIBOLD))
                                .text_color(rgb(if selected { t.text } else { t.muted }))
                                .child(title),
                        )
                }),
            );
        self.setting_row("外观", "", options)
    }

    fn scan_settings(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let scan = &self.controller.model.settings.scan;
        div()
            .child(
                self.setting_row(
                    "大文件阈值",
                    "只列出大于该大小的个人文件，修改后重新扫描。",
                    value_control(
                        "threshold",
                        format!("{} MB", scan.minimum_file_mb),
                        t,
                        !self.busy(),
                    )
                    .numeric()
                    .on_click(cx.listener(
                        |this, event: &ClickEvent, _, cx| {
                            this.open_picker(Picker::Threshold, event.position(), cx);
                        },
                    )),
                ),
            )
            .child(self.setting_divider())
            .child(
                self.setting_row(
                    "临时文件保留时间",
                    "临时文件与崩溃报告使用固定保留规则，不随开发项目设置变化。",
                    div()
                        .type_style(TextRole::Callout)
                        .numeric()
                        .text_color(rgb(t.muted))
                        .child(format!(
                            "{} 小时（固定）",
                            purge_engine::scanner::TEMPORARY_RETENTION_HOURS
                        )),
                ),
            )
            .child(self.setting_divider())
            .child(
                self.setting_row(
                    "开发项目闲置时间",
                    "仅用于已添加的开发项目，按项目内最新修改时间判断；修改后重新扫描。",
                    value_control(
                        "stale",
                        menus::stale_days_label(scan.stale_project_days),
                        t,
                        !self.busy(),
                    )
                    .numeric()
                    .on_click(cx.listener(
                        |this, event: &ClickEvent, _, cx| {
                            this.open_picker(Picker::StaleDays, event.position(), cx);
                        },
                    )),
                ),
            )
    }

    fn directory_settings(&self, exclusion: bool, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let (title, description, paths) = if exclusion {
            (
                "排除目录",
                "这些目录及其子目录不会被扫描，也不能被清理。",
                &self.controller.model.settings.scan.exclusions,
            )
        } else {
            (
                "开发项目目录",
                "只识别有对应项目清单的依赖、虚拟环境和构建产物。",
                &self.controller.model.settings.scan.project_roots,
            )
        };
        let content = div()
            .child(
                self.setting_row(
                    title,
                    description,
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
                    .h(px(24.))
                    .px(px(10.))
                    .type_style(TextRole::Callout)
                    .font_weight(FontWeight::SEMIBOLD)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if !this.busy() {
                            this.choose_folder(exclusion, cx);
                        }
                    })),
                ),
            )
            .child(self.setting_divider())
            .when(paths.is_empty(), |e| {
                e.child(
                    div()
                        .px(px(style::SPACE_16))
                        .py(px(style::SPACE_12))
                        .type_style(TextRole::Callout)
                        .text_color(rgb(t.muted))
                        .child("尚未添加目录"),
                )
            })
            .children(paths.iter().enumerate().map(|(index, path)| {
                div()
                    .flex()
                    .items_center()
                    .gap(px(style::SPACE_12))
                    .px(px(style::SPACE_16))
                    .py(px(style::SPACE_12))
                    .when(index > 0, |e| {
                        e.border_t(px(0.5)).border_color(rgb(t.border))
                    })
                    .child(icon("folder", t.muted, 16.))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .type_style(TextRole::Callout)
                            .font_family(style::MONOSPACE)
                            .text_color(rgb(t.muted))
                            .overflow_hidden()
                            .child(elided_middle(
                                platform::display_path(path),
                                TextRole::Callout,
                            )),
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
                            .size(px(24.))
                            .rounded_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .hover(move |e| e.bg(rgb(t.fill_hover)))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.remove_directory(index, exclusion, cx)
                            }))
                            .child(icon("x", t.muted, 12.)),
                    )
            }));
        let label = if exclusion {
            "排除规则"
        } else {
            "项目扫描"
        };
        div()
            .flex()
            .flex_col()
            .gap(px(style::SPACE_6))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(style::SPACE_8))
                    .px(px(style::SPACE_4))
                    .child(
                        div()
                            .type_style(TextRole::MetadataEmphasis)
                            .text_color(rgb(t.muted))
                            .child(label),
                    )
                    .child(
                        div()
                            .type_style(TextRole::Micro)
                            .numeric()
                            .text_color(rgb(t.faint))
                            .child(paths.len().to_string()),
                    ),
            )
            .child(
                content
                    .rounded(px(style::CARD_RADIUS))
                    .bg(rgb(t.fill))
                    .border(px(0.5))
                    .border_color(rgb(t.border))
                    .overflow_hidden(),
            )
    }

    fn remove_directory(&mut self, index: usize, exclusion: bool, cx: &mut Context<Self>) {
        if self.busy() {
            return;
        }
        let scan = &mut self.controller.model.settings.scan;
        let paths = if exclusion {
            &mut scan.exclusions
        } else {
            &mut scan.project_roots
        };
        if index < paths.len() {
            paths.remove(index);
        }
        self.save();
        cx.notify();
    }

    pub(in crate::ui) fn settings_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme;
        let general =
            div()
                .child(self.setting_row(
                    "启动时扫描",
                    "启动后自动检查各分类，不会自动清理。",
                    toggle("auto-scan", self.controller.model.settings.scan_on_start, t).on_click(
                        cx.listener(|this, _, _, cx| {
                            this.controller.model.settings.scan_on_start =
                                !this.controller.model.settings.scan_on_start;
                            this.save();
                            cx.notify();
                        }),
                    ),
                ))
                .child(self.setting_divider())
                .child(
                    self.setting_row(
                        "最小化到托盘",
                        "最小化后在后台保留任务，可从系统托盘恢复或退出。",
                        toggle(
                            "minimize-to-tray",
                            self.controller.model.settings.minimize_to_tray,
                            t,
                        )
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.controller.model.settings.minimize_to_tray =
                                !this.controller.model.settings.minimize_to_tray;
                            this.save();
                            cx.notify();
                        })),
                    ),
                )
                .child(self.setting_divider())
                .child(self.setting_row(
                    "后台运行",
                    "托盘不可用时保留普通最小化，关闭窗口仍会退出应用。",
                    button("hide-to-tray", "收起窗口", "tray", t, false, true).on_click(
                        |_, _, cx| shell::request(purge::lifecycle::Request::HideToTray, cx),
                    ),
                ))
                .child(self.setting_divider())
                .child(self.appearance_picker(cx));
        div()
            .id("settings-scroll")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(style::SPACE_24))
                    .pb(px(style::SPACE_24))
                    .child(self.settings_section("通用", general))
                    .child(self.settings_section("扫描", self.scan_settings(cx)))
                    .child(self.directory_settings(false, cx))
                    .child(self.directory_settings(true, cx)),
            )
            .into_any_element()
    }
}
