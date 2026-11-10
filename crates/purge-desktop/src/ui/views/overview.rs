use super::*;
impl PurgeView {
    pub(crate) fn overview(&self, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme;
        let total = self.controller.model.disk.total;
        let used = self.controller.model.disk.used();
        let mut bar = div()
            .flex()
            .w_full()
            .h(px(style::BAR_HEIGHT))
            .gap(px(2.))
            .rounded_full()
            .overflow_hidden()
            .bg(rgb(t.chart_free));
        let mut accounted = 0;
        for (index, category) in [Category::Caches, Category::Developer, Category::LargeFiles]
            .into_iter()
            .enumerate()
        {
            let bytes = self
                .controller
                .model
                .items
                .iter()
                .filter(|i| {
                    i.category() == category
                        && i.path
                            .to_string_lossy()
                            .to_lowercase()
                            .starts_with(&self.controller.model.disk.name.to_lowercase())
                })
                .map(Item::bytes)
                .sum::<u64>()
                .min(used.saturating_sub(accounted));
            accounted += bytes;
            if total > 0 && bytes > 0 {
                bar = bar.child(
                    div()
                        .id(("disk-segment", index))
                        .h_full()
                        .w(relative(bytes as f32 / total as f32))
                        .bg(rgb(category.color()))
                        .cursor_pointer()
                        .on_hover(cx.listener(move |this, hovered, _, cx| {
                            this.highlighted = if *hovered { Some(category) } else { None };
                            cx.notify();
                        }))
                        .on_click(cx.listener(move |this, _, w, cx| {
                            this.navigate(Page::Category(category), w, cx)
                        })),
                );
            }
        }
        if total > 0 {
            bar = bar.child(
                div()
                    .h_full()
                    .w(relative(
                        used.saturating_sub(accounted) as f32 / total as f32,
                    ))
                    .bg(rgb(t.chart_used)),
            );
        }
        let mut categories = div()
            .rounded(px(style::CARD_RADIUS))
            .border_1()
            .border_color(rgb(t.border))
            .bg(rgb(t.card))
            .overflow_hidden();
        for (index, category) in Category::ALL.into_iter().enumerate() {
            if index > 0 {
                categories = categories.child(div().mx(px(16.)).h(px(0.5)).bg(rgb(t.border)));
            }
            let count = self
                .controller
                .model
                .items
                .iter()
                .filter(|i| i.category() == category)
                .count();
            let bytes = self.total(category);
            let cross_volume = category != Category::Applications
                && self
                    .controller
                    .model
                    .items
                    .iter()
                    .filter(|item| item.category() == category)
                    .any(|item| {
                        !platform::within(
                            &item.path,
                            std::path::Path::new(&self.controller.model.disk.name),
                        )
                    });
            let safe = self
                .controller
                .model
                .items
                .iter()
                .filter(|i| i.category() == category && i.safety == Safety::Safe)
                .map(Item::bytes)
                .sum::<u64>();
            let detail = if safe > 0 {
                format!("{} 可安全清理", format_bytes(safe))
            } else if category == Category::Applications {
                format!("{count} 个应用 · 安装程序估算")
            } else if category == Category::LargeFiles {
                format!(
                    "{count} 个文件，大于 {} MB",
                    self.controller.model.settings.scan.minimum_file_mb
                )
            } else {
                format!("{count} 个项目")
            };
            let emphasized = self.highlighted == Some(category);
            let detail = if cross_volume {
                format!("{detail} · 含其他磁盘")
            } else {
                detail
            };
            categories = categories.child(
                div()
                    .id(("summary", index))
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .px(px(14.))
                    .py(px(12.))
                    .min_h(px(58.))
                    .bg(rgb(if emphasized { t.row_hover } else { t.card }))
                    .cursor_pointer()
                    .hover(move |e| e.bg(rgb(t.row_hover)))
                    .opacity(if self.highlighted.is_some() && !emphasized {
                        0.55
                    } else {
                        1.
                    })
                    .on_click(cx.listener(move |this, _, w, cx| {
                        this.navigate(Page::Category(category), w, cx)
                    }))
                    .child(self.overview_tile(category))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.))
                                    .child(
                                        div()
                                            .text_size(px(13.))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child(category.label()),
                                    )
                                    .when(
                                        matches!(
                                            category,
                                            Category::LargeFiles | Category::Applications
                                        ),
                                        |e| e.child(tag("需要确认", t.review, t.review_bg)),
                                    ),
                            )
                            .child(
                                div()
                                    .mt(px(2.))
                                    .text_size(px(11.))
                                    .text_color(rgb(t.muted))
                                    .text_ellipsis()
                                    .child(detail),
                            ),
                    )
                    .child(
                        div()
                            .w(px(54.))
                            .text_right()
                            .font_family("Segoe UI")
                            .text_size(px(11.))
                            .text_color(rgb(t.faint))
                            .child(
                                if total > 0 && category != Category::Applications && !cross_volume
                                {
                                    format!("{:.1}%", bytes as f64 / total as f64 * 100.)
                                } else {
                                    "—".into()
                                },
                            ),
                    )
                    .child(
                        div()
                            .min_w(px(74.))
                            .text_right()
                            .font_family("Segoe UI")
                            .text_size(px(style::SECTION_SIZE))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(if bytes > 0 {
                                format_bytes(bytes)
                            } else if self.controller.operation() == Some(Operation::Scan) {
                                "扫描中".into()
                            } else {
                                "—".into()
                            }),
                    ),
            );
        }
        let other = used.saturating_sub(accounted);
        let rest = div()
            .rounded(px(14.))
            .border_1()
            .border_color(rgb(t.border))
            .bg(rgb(t.card))
            .overflow_hidden()
            .child(
                div()
                    .id("other-space")
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .p(px(14.))
                    .cursor_pointer()
                    .hover(move |e| e.bg(rgb(t.row_hover)))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.handle(platform::open_storage_settings(), cx)
                    }))
                    .child(self.neutral_tile("info"))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(div().font_weight(FontWeight::SEMIBOLD).child("其他空间"))
                            .child(
                                div()
                                    .mt(px(2.))
                                    .text_size(px(11.))
                                    .text_color(rgb(t.muted))
                                    .child("Windows、个人文件和 Purge 未分类的内容"),
                            ),
                    )
                    .child(
                        div()
                            .w(px(54.))
                            .text_right()
                            .font_family("Segoe UI")
                            .text_size(px(11.))
                            .text_color(rgb(t.faint))
                            .child(if total > 0 {
                                format!("{:.1}%", other as f64 / total as f64 * 100.)
                            } else {
                                "—".into()
                            }),
                    )
                    .child(
                        div()
                            .min_w(px(74.))
                            .text_right()
                            .font_family("Segoe UI")
                            .text_size(px(style::SECTION_SIZE))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(format_bytes(other)),
                    ),
            )
            .child(div().mx(px(16.)).h(px(0.5)).bg(rgb(t.border)))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .p(px(14.))
                    .child(
                        div()
                            .size(px(28.))
                            .rounded(px(style::TILE_RADIUS))
                            .border_1()
                            .border_color(rgb(t.faint)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .child(div().font_weight(FontWeight::SEMIBOLD).child("可用空间"))
                            .child(
                                div()
                                    .mt(px(2.))
                                    .text_size(px(11.))
                                    .text_color(rgb(t.muted))
                                    .child("可用于存储新文件"),
                            ),
                    )
                    .child(
                        div()
                            .min_w(px(74.))
                            .text_right()
                            .font_family("Segoe UI")
                            .text_size(px(style::SECTION_SIZE))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(format_bytes(self.controller.model.disk.free)),
                    ),
            );
        // A single block inside the scroll viewport prevents flex-shrinking or clipping
        // the summary when the window is short or scan results arrive incrementally.
        div()
            .id("overview-scroll")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(24.))
                    .pb(px(24.))
                    .child(
                        div()
                            .child(
                                div()
                                    .flex()
                                    .items_end()
                                    .gap(px(8.))
                                    .child(
                                        div()
                                            .font_family("Segoe UI")
                                            .text_size(px(style::TOTAL_SIZE))
                                            .font_weight(FontWeight::BOLD)
                                            .child(format!("{} 已用", format_storage_bytes(used))),
                                    )
                                    .child(
                                        div()
                                            .pb(px(3.))
                                            .text_size(px(13.))
                                            .text_color(rgb(t.muted))
                                            .child(format!("/ {}", format_storage_bytes(total))),
                                    )
                                    .child(div().flex_1())
                                    .child(
                                        div()
                                            .pb(px(3.))
                                            .text_size(px(13.))
                                            .text_color(rgb(t.muted))
                                            .child(format!(
                                                "{} 可用",
                                                format_bytes(self.controller.model.disk.free)
                                            )),
                                    ),
                            )
                            .child(div().mt(px(12.)).child(bar)),
                    )
                    .child(categories)
                    .child(rest)
                    .child(div().text_size(px(11.)).text_color(rgb(t.faint)).child(
                        "文件默认移入回收站。手动清空回收站后释放空间；Purge 不会永久删除文件。",
                    )),
            )
            .into_any_element()
    }

    fn overview_tile(&self, category: Category) -> Div {
        let color = if self.theme.dark {
            match category {
                Category::Caches => 0x3a9bd5,
                Category::Developer => 0x1e78ba,
                Category::LargeFiles => 0xe69f00,
                Category::Applications => 0xd55e00,
            }
        } else {
            match category {
                Category::Caches => 0x4b83a5,
                Category::Developer => 0x526f9c,
                Category::LargeFiles => 0xa9812f,
                Category::Applications => 0xad734f,
            }
        };
        self.glossy_tile(category_icon(category), color)
    }
    fn neutral_tile(&self, glyph: &'static str) -> Div {
        self.glossy_tile(glyph, self.theme.neutral_tile)
    }
    fn glossy_tile(&self, glyph: &'static str, color: u32) -> Div {
        let mix = |overlay: u32, amount: f32| {
            let channel = |shift: u32| {
                let a = (color >> shift) & 255;
                let b = (overlay >> shift) & 255;
                ((a as f32 * (1. - amount) + b as f32 * amount).round() as u32) << shift
            };
            channel(16) | channel(8) | channel(0)
        };
        div()
            .size(px(style::TILE_SIZE))
            .flex_shrink_0()
            .rounded(px(style::TILE_RADIUS))
            .border(px(0.5))
            .border_color(rgba(0xffffff55))
            .bg(linear_gradient(
                180.,
                linear_color_stop(rgb(mix(0xffffff, 0.22)), 0.),
                linear_color_stop(rgb(mix(0, 0.12)), 1.),
            ))
            .flex()
            .items_center()
            .justify_center()
            .child(icon(glyph, 0xffffff, 15.))
    }
}
