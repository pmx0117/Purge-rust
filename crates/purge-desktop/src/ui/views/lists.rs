use super::*;
use purge::model::{FilterTab, SelectionSummary};
impl PurgeView {
    pub(in crate::ui) fn category(
        &self,
        category: Category,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let query = self.search.read(cx).content.to_string();
        let visible = self.controller.model.visible(&query);
        let groups = self.controller.model.grouped_visible(&query);
        let summary = self.controller.model.selection_summary(&visible);
        let show_search =
            matches!(category, Category::LargeFiles | Category::Applications) || !query.is_empty();
        let controls = div()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap(px(style::SPACE_12))
            .when(show_search, |e| {
                e.child(self.result_search(category, groups.len(), window, cx))
            })
            .when(category != Category::Applications, |e| {
                e.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(style::SPACE_12))
                        .child(self.result_filter_tabs(category, cx))
                        .when(category == Category::LargeFiles, |e| {
                            e.child(div().flex_1()).child(self.result_file_types(cx))
                        }),
                )
            })
            .child(self.result_selection(category, summary, query, cx));
        div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .child(controls)
            .child(self.result_list(category, groups, cx))
            .into_any_element()
    }
    fn result_search(
        &self,
        category: Category,
        count: usize,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Div {
        let t = self.theme;
        let focused = self.search.focus_handle(cx).is_focused(window);
        div()
            .flex()
            .items_center()
            .gap(px(style::SPACE_8))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .h(px(28.))
                    .px(px(10.))
                    .rounded(px(style::FIELD_RADIUS))
                    .border_1()
                    .border_color(rgb(t.border_strong))
                    .bg(rgb(t.fill))
                    .when(focused, |e| {
                        e.when(!t.dark, |e| e.border_color(rgb(t.accent)).bg(rgb(t.card)))
                    })
                    .flex()
                    .items_center()
                    .gap(px(style::SPACE_8))
                    .child(icon("search", t.faint, 13.))
                    .child(self.search.clone()),
            )
            .when(category == Category::LargeFiles, |e| {
                e.child(
                    button(
                        "duplicates",
                        "查找重复",
                        "copy",
                        t,
                        false,
                        !self.busy() && count > 0,
                    )
                    .on_click(cx.listener(|this, _, _, cx| this.duplicates(cx))),
                )
            })
    }
    fn result_filter_tabs(&self, category: Category, cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(style::SPACE_4))
            .children(
                self.controller
                    .model
                    .filter_tabs(category)
                    .into_iter()
                    .enumerate()
                    .map(|(index, tab)| self.result_filter_chip(index, tab, cx)),
            )
    }
    fn result_filter_chip(
        &self,
        index: usize,
        tab: FilterTab,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let t = self.theme;
        let filter = tab.filter;
        let (background, foreground) = if tab.selected {
            match filter {
                Filter::Safe => (t.safe_bg, t.safe),
                Filter::Review => (t.review_bg, t.review),
                _ => (t.raised, t.text),
            }
        } else {
            (t.fill, t.muted)
        };
        div()
            .id(("filter", index))
            .flex()
            .items_center()
            .gap(px(style::SPACE_6))
            .px(px(style::SPACE_12))
            .py(px(style::SPACE_6))
            .rounded_full()
            .border_1()
            .border_color(rgb(t.border))
            .bg(rgb(background))
            .type_style(TextRole::Body)
            .text_color(rgb(foreground))
            .when(tab.selected, |e| e.font_weight(FontWeight::SEMIBOLD))
            .when(!tab.selected && tab.count == 0, |e| e.opacity(0.45))
            .cursor_pointer()
            .hover(move |e| e.border_color(rgb(t.border_strong)))
            .on_click(cx.listener(move |this, _, _, cx| this.set_filter(filter, cx)))
            .child(icon(tab.glyph, foreground, 14.))
            .child(tab.label)
            .child(
                div()
                    .type_style(TextRole::RowTitle)
                    .numeric()
                    .text_color(rgb(if tab.selected { foreground } else { t.faint }))
                    .child(tab.count.to_string()),
            )
    }
    fn result_file_types(&self, cx: &mut Context<Self>) -> Div {
        div().child(
            value_control(
                "file-type",
                self.controller
                    .model
                    .kind_filter
                    .map(FileKind::label)
                    .unwrap_or("所有类型"),
                self.theme,
                true,
            )
            .on_click(cx.listener(|this, event: &ClickEvent, _, cx| {
                this.open_picker(Picker::FileType, event.position(), cx);
            })),
        )
    }
    fn result_selection(
        &self,
        category: Category,
        summary: SelectionSummary,
        query: String,
        cx: &mut Context<Self>,
    ) -> Div {
        let t = self.theme;
        let label = summary.label();
        let review_count = summary.review_add.len();
        div()
            .flex()
            .items_center()
            .gap(px(style::SPACE_8))
            .pt(px(style::SPACE_8))
            .pb(px(style::SPACE_12))
            .when(category != Category::Applications, |e| {
                e.child(
                    div()
                        .id("select-all")
                        .flex()
                        .items_center()
                        .gap(px(style::SPACE_8))
                        .cursor_pointer()
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.controller.toggle_visible(&query);
                            cx.notify();
                        }))
                        .child(
                            div()
                                .size(px(16.))
                                .when(summary.mixed, |e| {
                                    e.rounded(px(4.))
                                        .bg(rgb(t.action))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(div().w(px(8.)).h(px(2.)).bg(rgb(t.on_action)))
                                })
                                .when(!summary.mixed, |e| e.child(self.checkbox(summary.all))),
                        )
                        .child(label),
                )
            })
            .when(category == Category::Applications, |e| {
                e.child(
                    div()
                        .type_style(TextRole::Metadata)
                        .text_color(rgb(t.muted))
                        .child("卸载由系统应用管理器处理"),
                )
            })
            .when(summary.show_review_link, |e| {
                e.child(
                    div()
                        .id("also-review")
                        .type_style(TextRole::Callout)
                        .text_color(rgb(t.muted))
                        .cursor_pointer()
                        .hover(move |e| e.text_color(rgb(t.text)))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.controller.select_items(&summary.review_add);
                            cx.notify();
                        }))
                        .child(format!(
                            "另选 {review_count} 项需要确认 ({})",
                            format_bytes(summary.review_bytes)
                        )),
                )
            })
            .child(div().flex_1())
            .child(
                button(
                    "sort",
                    self.controller.model.sort.label(),
                    "sort",
                    t,
                    false,
                    true,
                )
                .on_click(cx.listener(|this, event: &ClickEvent, _, cx| {
                    this.open_picker(Picker::Sort, event.position(), cx);
                })),
            )
    }
    fn result_list(
        &self,
        category: Category,
        groups: Vec<ItemGroup>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if groups.is_empty() {
            return self.empty_results(category);
        }
        uniform_list(
            ("items", category as usize),
            groups.len(),
            cx.processor(move |this, range: std::ops::Range<usize>, _, cx| {
                range
                    .map(|index| this.item_row(groups[index].clone(), cx))
                    .collect()
            }),
        )
        .flex_1()
        .min_h_0()
        .into_any_element()
    }
    fn empty_results(&self, category: Category) -> AnyElement {
        let t = self.theme;
        div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(style::SPACE_12))
            .child(icon(category_icon(category), t.faint, 32.))
            .child(
                div()
                    .type_style(TextRole::Title)
                    .child(empty_title(self.busy())),
            )
            .child(
                div()
                    .type_style(TextRole::Callout)
                    .text_color(rgb(t.muted))
                    .child(empty_hint(category)),
            )
            .into_any_element()
    }
    fn item_row(&self, group: ItemGroup, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme;
        let item = &group.items[0];
        let id = group.id;
        let ids: Vec<_> = group.items.iter().map(|item| item.id).collect();
        let selected = ids
            .iter()
            .all(|id| self.controller.model.selected.contains(id));
        let is_app = item.is_application();
        let for_menu = group.clone();
        let context_active = self
            .context_item
            .as_ref()
            .is_some_and(|(item, _)| item.id == id);
        div()
            .id(("item", id))
            .h(px(style::SCAN_ROW_HEIGHT))
            .py(px(style::SPACE_4))
            .child(
                div()
                    .id(("row", id))
                    .size_full()
                    .flex()
                    .items_center()
                    .gap(px(style::SPACE_12))
                    .px(px(style::ROW_PADDING))
                    .py(px(style::SPACE_12))
                    .rounded(px(style::CARD_RADIUS))
                    .bg(rgb(if context_active { t.row_context } else { t.row }))
                    .when(context_active, |e| e.border_1().border_color(rgb(t.border)))
                    .when(!is_app && !self.busy(), |e| e.cursor_pointer())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.controller.toggle_group(&ids);
                        cx.notify();
                    }))
                    .on_mouse_down(
                        MouseButton::Right,
                        cx.listener(move |this, event: &MouseDownEvent, _, cx| {
                            this.context_item = Some((for_menu.clone(), event.position));
                            this.picker = None;
                            cx.notify();
                        }),
                    )
                    .when(!is_app, |e| e.child(self.checkbox(selected)))
                    .child(self.result_icon(&group))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .h(px(style::SCAN_TEXT_HEIGHT))
                            .flex()
                            .flex_col()
                            .justify_center()
                            .child(
                                div()
                                    .type_style(TextRole::Headline)
                                    .overflow_hidden()
                                    .child(elided(group.name.clone(), TextRole::Headline)),
                            )
                            .child(
                                div()
                                    .mt(px(style::SPACE_4))
                                    .type_style(TextRole::Metadata)
                                    .text_color(rgb(t.muted))
                                    .line_clamp(2)
                                    .child(item.description.clone()),
                            ),
                    )
                    .child(self.result_metadata(&group, cx)),
            )
            .into_any_element()
    }

    fn result_icon(&self, group: &ItemGroup) -> Div {
        let t = self.theme;
        let item = &group.items[0];
        let leading = div()
            .size(px(style::TILE_SIZE))
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_center();
        if let Some(brand) = brand_name(&group.name) {
            return leading.child(
                img(ImageSource::Resource(Resource::Embedded(
                    format!("brand/{}{}", brand, if t.dark { "" } else { "-light" }).into(),
                )))
                .size(px(style::TILE_SIZE)),
            );
        }
        let glyph = if item.is_application() {
            "apps"
        } else if item.category() == Category::LargeFiles {
            "file"
        } else {
            "folder"
        };
        leading.child(icon(glyph, t.muted, 18.))
    }

    fn result_metadata(&self, group: &ItemGroup, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let item = &group.items[0];
        let safe = item.safety == Safety::Safe;
        div()
            .flex_shrink_0()
            .min_w(px(72.))
            .flex()
            .flex_col()
            .items_end()
            .gap(px(style::SPACE_8))
            .child(
                div()
                    .type_style(TextRole::MetadataEmphasis)
                    .numeric()
                    .child(if group.bytes() > 0 {
                        format_bytes(group.bytes())
                    } else {
                        "大小未知".into()
                    }),
            )
            .when(!item.is_application(), |e| {
                e.child(tag(
                    if safe {
                        "可安全清理"
                    } else {
                        "需要确认"
                    },
                    if safe { t.safe } else { t.review },
                    if safe { t.safe_bg } else { t.review_bg },
                ))
            })
            .when(item.is_application(), |e| {
                e.child(
                    div()
                        .id(("uninstall", group.id))
                        .type_style(TextRole::Metadata)
                        .text_color(rgb(t.muted))
                        .cursor_pointer()
                        .hover(move |e| e.text_color(rgb(t.text)))
                        .on_click(cx.listener(|this, _, _, cx| {
                            cx.stop_propagation();
                            this.handle(platform::open_apps_settings(), cx);
                        }))
                        .child("管理 / 卸载"),
                )
            })
    }

    pub(in crate::ui) fn history_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme;
        let total = self
            .controller
            .model
            .history
            .iter()
            .map(|h| h.bytes)
            .sum::<u64>();
        let mut list = div().flex().flex_col().gap(px(style::SPACE_12)).child(
            div()
                .bg(rgb(t.card))
                .border_1()
                .border_color(rgb(t.border))
                .rounded(px(style::CARD_RADIUS))
                .p(px(style::SPACE_24))
                .flex()
                .items_center()
                .gap(px(style::SPACE_16))
                .child(icon("history", t.safe, 28.))
                .child(
                    div()
                        .flex_1()
                        .child(
                            div()
                                .type_style(TextRole::Total)
                                .numeric()
                                .child(format_bytes(total)),
                        )
                        .child(
                            div()
                                .type_style(TextRole::Callout)
                                .text_color(rgb(t.muted))
                                .child(format!(
                                    "累计移入回收站 · {} 次操作",
                                    self.controller.model.history.len()
                                )),
                        ),
                )
                .child(
                    button("history-bin", "打开回收站", "trash", t, false, true).on_click(
                        cx.listener(|this, _, _, cx| this.handle(platform::open_recycle_bin(), cx)),
                    ),
                ),
        );
        if self.controller.model.history.is_empty() {
            list = list.child(
                div()
                    .p(px(style::SPACE_32))
                    .text_color(rgb(t.muted))
                    .child("还没有清理记录。完成第一次清理后，记录会显示在这里。"),
            );
        }
        for (index, entry) in self.controller.model.history.iter().enumerate() {
            list = list.child(
                div()
                    .id(("history-entry", index))
                    .bg(rgb(t.card))
                    .rounded(px(style::CARD_RADIUS))
                    .p(px(style::SPACE_16))
                    .flex()
                    .items_center()
                    .gap(px(style::SPACE_16))
                    .child(icon("check", t.safe, 16.))
                    .child(
                        div()
                            .flex_1()
                            .child(format!("{} 项已移入回收站", entry.moved))
                            .child(
                                div()
                                    .mt(px(style::SPACE_4))
                                    .type_style(TextRole::Callout)
                                    .text_color(rgb(t.faint))
                                    .child(entry.time.clone()),
                            ),
                    )
                    .when(entry.failed > 0, |e| {
                        e.child(tag(
                            format!("{} 项跳过", entry.failed),
                            t.review,
                            t.review_bg,
                        ))
                    })
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(format_bytes(entry.bytes)),
                    ),
            );
        }
        div()
            .id("history-scroll")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .pb(px(style::SPACE_16))
            .child(list)
            .into_any_element()
    }
}

fn brand_name(name: &str) -> Option<&'static str> {
    let lower = name.to_lowercase();
    [
        ("chrome", "googlechrome"),
        ("brave", "brave"),
        ("cursor", "cursor"),
        ("discord", "discord"),
        ("slack", "slack"),
        ("notion", "notion"),
        ("pnpm", "pnpm"),
        ("pip", "python"),
        ("python", "python"),
        ("cargo", "rust"),
        ("gradle", "gradle"),
        ("uv", "uv"),
        ("yarn", "yarn"),
    ]
    .into_iter()
    .find(|(needle, _)| lower.contains(needle))
    .map(|(_, brand)| brand)
}

fn empty_title(busy: bool) -> &'static str {
    if busy { "正在扫描" } else { "暂无项目" }
}
fn empty_hint(category: Category) -> &'static str {
    if category == Category::Developer {
        "可在设置中添加项目目录，或调整筛选条件。"
    } else {
        "重新扫描，或调整筛选条件。"
    }
}
