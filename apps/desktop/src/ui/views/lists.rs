use super::*;
use purge::model::{FilterTab, SelectionSummary};
use std::collections::HashMap;

enum ResultRow {
    Group(u64, usize),
    Item(ItemGroup),
}
impl PurgeView {
    pub(in crate::ui) fn category(
        &mut self,
        category: Category,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let query = self.search.read(cx).content.to_string();
        let visible = self.controller.model.visible(&query);
        let groups = self.controller.model.grouped_visible(&query);
        let group_ids: std::collections::HashSet<_> = groups.iter().map(|group| group.id).collect();
        self.result_focus.retain(|id, _| group_ids.contains(id));
        for group in &groups {
            self.result_focus
                .entry(group.id)
                .or_insert_with(|| cx.focus_handle());
        }
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
            .child(self.result_selection(category, summary, query, cx))
            .child(self.selected_results_bar(&visible, cx));
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
        _count: usize,
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
                        if self.controller.operation() == Some(Operation::Duplicates) {
                            "停止比较"
                        } else {
                            "查找重复"
                        },
                        "copy",
                        t,
                        false,
                        self.controller.operation() == Some(Operation::Duplicates)
                            || (!self.busy() && self.controller.model.can_find_duplicates()),
                    )
                    .on_click(cx.listener(|this, _, _, cx| {
                        if this.controller.operation() == Some(Operation::Duplicates) {
                            this.controller.cancel();
                            cx.notify();
                        } else {
                            this.duplicates(cx);
                        }
                    })),
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
            .focusable()
            .tab_index(0)
            .focus(move |e| e.border_color(rgb(t.accent)))
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
            .on_click(cx.listener(|this, event: &ClickEvent, w, cx| {
                this.open_picker(Picker::FileType, event.position(), w, cx);
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
        let label = if self.controller.model.filter == Filter::Duplicates {
            "选择多余副本"
        } else {
            summary.label()
        };
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
                        .focusable()
                        .tab_index(0)
                        .focus(move |e| e.rounded(px(4.)).border_1().border_color(rgb(t.accent)))
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
                .on_click(cx.listener(|this, event: &ClickEvent, w, cx| {
                    this.open_picker(Picker::Sort, event.position(), w, cx);
                })),
            )
    }
    fn result_list(
        &mut self,
        category: Category,
        groups: Vec<ItemGroup>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if groups.is_empty() {
            self.result_controls.clear();
            return self.empty_results(category, cx);
        }
        let mut rows = Vec::new();
        if self.controller.model.filter == Filter::Duplicates {
            let mut buckets: HashMap<u64, Vec<ItemGroup>> = HashMap::new();
            let mut order = Vec::new();
            for group in groups {
                let key = group.items[0].duplicate_group.unwrap_or(group.id);
                if !buckets.contains_key(&key) {
                    order.push(key);
                }
                buckets.entry(key).or_default().push(group);
            }
            for (index, key) in order.into_iter().enumerate() {
                rows.push(ResultRow::Group(key, index + 1));
                rows.extend(
                    buckets
                        .remove(&key)
                        .unwrap_or_default()
                        .into_iter()
                        .map(ResultRow::Item),
                );
            }
        } else {
            rows.extend(groups.into_iter().map(ResultRow::Item));
        }
        self.result_controls = rows
            .iter()
            .enumerate()
            .filter_map(|(row, item)| {
                if let ResultRow::Item(group) = item {
                    Some(ResultControl {
                        id: group.id,
                        row,
                        items: group.items.iter().map(|item| item.id).collect(),
                    })
                } else {
                    None
                }
            })
            .collect();
        uniform_list(
            ("items", category as usize),
            rows.len(),
            cx.processor(move |this, range: std::ops::Range<usize>, _, cx| {
                range
                    .map(|index| match &rows[index] {
                        ResultRow::Group(group, number) => {
                            this.duplicate_heading(*group, *number).into_any_element()
                        }
                        ResultRow::Item(group) => this.item_row(group.clone(), cx),
                    })
                    .collect()
            }),
        )
        .track_scroll(&self.result_scroll)
        .flex_1()
        .min_h_0()
        .into_any_element()
    }
    fn empty_results(&self, category: Category, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme;
        let state = self
            .controller
            .model
            .empty_state(&self.search.read(cx).content, self.busy());
        div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(style::SPACE_12))
            .child(icon(category_icon(category), t.faint, 32.))
            .child(div().type_style(TextRole::Title).child(state.title))
            .child(
                div()
                    .type_style(TextRole::Callout)
                    .text_color(rgb(t.muted))
                    .child(state.hint),
            )
            .when_some(state.action, |e, action| {
                use purge::model::EmptyAction;
                e.child(
                    button(
                        "empty-next-step",
                        match action {
                            EmptyAction::Scan => "开始扫描",
                            EmptyAction::ResetFilters => "查看全部结果",
                            EmptyAction::Review => "查看需要确认的项目",
                            EmptyAction::Settings => "打开设置",
                        },
                        "external",
                        t,
                        false,
                        true,
                    )
                    .on_click(cx.listener(
                        move |this, _, w, cx| match action {
                            EmptyAction::Scan => this.rescan(&Rescan, w, cx),
                            EmptyAction::Settings => this.navigate(Page::Settings, w, cx),
                            EmptyAction::Review => this.set_filter(Filter::Review, cx),
                            EmptyAction::ResetFilters => {
                                this.controller.model.reset_result_filters();
                                this.search.update(cx, |input, cx| input.clear(cx));
                                cx.notify();
                            }
                        },
                    )),
                )
            })
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
        let personal = item.category() == Category::LargeFiles;
        let description = if personal {
            let modified = item
                .modified
                .map(|modified| {
                    chrono::DateTime::<chrono::Local>::from(modified)
                        .format("%Y-%m-%d %H:%M")
                        .to_string()
                })
                .unwrap_or_else(|| "时间未知".into());
            format!("{} · {}", platform::display_path(&item.path), modified)
        } else {
            item.description.clone()
        };
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
                    .focusable()
                    .tab_index(0)
                    .track_focus(&self.result_focus[&id])
                    .focus(move |e| e.border_1().border_color(rgb(t.accent)))
                    .on_key_down(cx.listener(move |this, event: &KeyDownEvent, w, cx| {
                        match event.keystroke.key.as_str() {
                            "down" => {
                                this.move_result_focus(
                                    id,
                                    true,
                                    event.keystroke.modifiers.shift,
                                    w,
                                    cx,
                                );
                                cx.stop_propagation();
                            }
                            "up" => {
                                this.move_result_focus(
                                    id,
                                    false,
                                    event.keystroke.modifiers.shift,
                                    w,
                                    cx,
                                );
                                cx.stop_propagation();
                            }
                            _ => (),
                        }
                    }))
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
                        cx.listener(move |this, event: &MouseDownEvent, w, cx| {
                            this.context_item = Some((for_menu.clone(), event.position));
                            this.picker = None;
                            this.focus_menu(3, w, cx);
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
                                    .child(description),
                            ),
                    )
                    .child(self.result_metadata(&group, cx)),
            )
            .into_any_element()
    }

    fn move_result_focus(
        &mut self,
        id: u64,
        forward: bool,
        extend: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(current) = self
            .result_controls
            .iter()
            .position(|control| control.id == id)
        else {
            return;
        };
        let next = if forward {
            (current + 1).min(self.result_controls.len() - 1)
        } else {
            current.saturating_sub(1)
        };
        if extend {
            let ids = self.result_controls[current]
                .items
                .iter()
                .chain(&self.result_controls[next].items)
                .copied()
                .collect::<Vec<_>>();
            self.controller.select_items(&ids);
        }
        let control = &self.result_controls[next];
        self.result_scroll
            .scroll_to_item(control.row, ScrollStrategy::Nearest);
        self.result_focus[&control.id].focus(window, cx);
        cx.notify();
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
        let keeper = item.id;
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
            .when(item.duplicate_group.is_some(), |e| {
                e.child(
                    button(
                        ("keep-copy", keeper),
                        "保留此份",
                        "check",
                        t,
                        false,
                        !self.busy(),
                    )
                    .h(px(24.))
                    .type_style(TextRole::Metadata)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        cx.stop_propagation();
                        if !this.busy() {
                            this.controller.model.select_duplicate_keeper(keeper);
                        }
                        cx.notify();
                    })),
                )
            })
            .when(
                !item.is_application() && item.duplicate_group.is_none(),
                |e| {
                    e.child(tag(
                        if safe {
                            "可安全清理"
                        } else {
                            "需要确认"
                        },
                        if safe { t.safe } else { t.review },
                        if safe { t.safe_bg } else { t.review_bg },
                    ))
                },
            )
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
                                    "最近 {} 次操作移入回收站",
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
                    .child(icon(
                        if entry.failed > 0 { "info" } else { "check" },
                        if entry.failed > 0 { t.review } else { t.safe },
                        16.,
                    ))
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
                    )
                    .child(
                        button(
                            ("history-details", index),
                            if self.history_details == Some(index) {
                                "收起详情"
                            } else {
                                "文件与原因"
                            },
                            "list",
                            t,
                            false,
                            true,
                        )
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.history_details = if this.history_details == Some(index) {
                                None
                            } else {
                                Some(index)
                            };
                            cx.notify();
                        })),
                    ),
            );
            if self.history_details == Some(index) {
                list = list.child(self.cleanup_history_details(index, cx));
            }
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
