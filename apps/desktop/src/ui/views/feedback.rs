use super::*;

impl PurgeView {
    pub(super) fn selected_results_bar(&self, visible: &[Item], cx: &mut Context<Self>) -> Div {
        let selected = self.controller.model.selected_items();
        if selected.is_empty() {
            return div();
        }
        let t = self.theme;
        let hidden = self.controller.model.hidden_selection_count(visible);
        let only_selected = self.controller.model.only_selected;
        let ids: Vec<_> = selected.iter().map(|item| item.id).collect();
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(style::SPACE_8))
            .pb(px(style::SPACE_8))
            .child(
                div()
                    .flex_1()
                    .type_style(TextRole::Callout)
                    .text_color(rgb(t.muted))
                    .child(format!(
                        "已选 {} 项 · {}{}",
                        selected.len(),
                        format_bytes(selected.iter().map(Item::bytes).sum()),
                        if hidden > 0 {
                            format!("，其中 {hidden} 项被当前筛选隐藏")
                        } else {
                            String::new()
                        }
                    )),
            )
            .child(
                button(
                    "show-selected",
                    if only_selected {
                        "返回全部"
                    } else {
                        "查看已选"
                    },
                    "list",
                    t,
                    false,
                    true,
                )
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.controller.model.reset_result_filters();
                    this.controller.model.only_selected = !only_selected;
                    this.search.update(cx, |input, cx| input.clear(cx));
                    cx.notify();
                })),
            )
            .child(
                button("clear-selected", "清空选择", "x", t, false, !self.busy()).on_click(
                    cx.listener(move |this, _, _, cx| {
                        if !this.busy() {
                            this.controller
                                .model
                                .selected
                                .retain(|id| !ids.contains(id));
                            this.controller.model.only_selected = false;
                        }
                        cx.notify();
                    }),
                ),
            )
    }

    pub(super) fn duplicate_heading(&self, group: u64, number: usize) -> Div {
        let t = self.theme;
        let members: Vec<_> = self
            .controller
            .model
            .items
            .iter()
            .filter(|item| item.duplicate_group == Some(group))
            .collect();
        let selected = members
            .iter()
            .filter(|item| self.controller.model.selected.contains(&item.id))
            .count();
        div()
            .h(px(style::SCAN_ROW_HEIGHT))
            .px(px(style::SPACE_12))
            .flex()
            .items_center()
            .gap(px(style::SPACE_12))
            .child(icon("copy", t.muted, 20.))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(style::SPACE_4))
                    .child(
                        div()
                            .type_style(TextRole::Headline)
                            .child(format!("重复组 {number} · {} 份相同内容", members.len())),
                    )
                    .child(
                        div()
                            .type_style(TextRole::Callout)
                            .text_color(rgb(t.muted))
                            .child(format!(
                                "保留 {} 份 · 待清理 {selected} 份。点击“保留此份”可选择其他副本。",
                                members.len().saturating_sub(selected)
                            )),
                    ),
            )
    }

    pub(in crate::ui) fn scan_scope_description(&self) -> String {
        let mut sections = vec![
            "扫描全部会检查所有分类，不会遍历全部磁盘。未列出的位置不会参与扫描。".to_string(),
        ];
        let category = self.controller.model.page.scan_category();
        if category.is_none() || category == Some(Category::LargeFiles) {
            sections.push(format!(
                "大文件目录：\n{}",
                purge_engine::platform::personal_roots()
                    .iter()
                    .map(|path| platform::display_path(path))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }
        if category.is_none() || matches!(category, Some(Category::Caches | Category::Developer)) {
            sections.push(format!(
                "应用与开发缓存目录：\n{}",
                purge_engine::scanner::cache_rules()
                    .iter()
                    .filter(|rule| category.is_none_or(|category| category == rule.category))
                    .map(|rule| platform::display_path(&rule.path))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }
        if category.is_none() || category == Some(Category::Developer) {
            let projects = &self.controller.model.settings.scan.project_roots;
            sections.push(format!(
                "开发项目目录（仅检查依赖和构建产物）：\n{}",
                if projects.is_empty() {
                    "尚未添加，可在设置中添加。".into()
                } else {
                    projects
                        .iter()
                        .map(|path| platform::display_path(path))
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            ));
        }
        if category.is_none() || category == Some(Category::Applications) {
            sections
                .push("已安装应用：读取系统应用登记信息，管理与卸载交给系统应用管理器。".into());
        }
        let exclusions = &self.controller.model.settings.scan.exclusions;
        if !exclusions.is_empty() {
            sections.push(format!(
                "以下目录及子目录已排除：\n{}",
                exclusions
                    .iter()
                    .map(|path| platform::display_path(path))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }
        sections.push(
            "扫描中的访问异常会显示提示；不存在或受保护的位置可能跳过。扫描范围修改后请重新扫描。"
                .into(),
        );
        sections.join("\n\n")
    }

    pub(in crate::ui) fn stale_results_banner(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        div()
            .mb(px(style::SPACE_12))
            .p(px(style::SPACE_12))
            .rounded(px(style::CARD_RADIUS))
            .bg(rgb(t.review_bg))
            .flex()
            .items_center()
            .gap(px(style::SPACE_12))
            .child(
                div()
                    .flex_1()
                    .type_style(TextRole::Callout)
                    .text_color(rgb(t.review))
                    .child("扫描设置已更改，当前结果尚未更新。"),
            )
            .child(
                button(
                    "refresh-stale-results",
                    "重新扫描",
                    "refresh",
                    t,
                    false,
                    !self.busy(),
                )
                .on_click(cx.listener(|this, _, w, cx| this.rescan(&Rescan, w, cx))),
            )
    }

    pub(super) fn cleanup_history_details(
        &self,
        index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let t = self.theme;
        let entry = self.controller.model.history[index].clone();
        if entry.items.is_empty() {
            return div()
                .p(px(style::SPACE_12))
                .type_style(TextRole::Callout)
                .text_color(rgb(t.muted))
                .child(if entry.failures.is_empty() {
                    "此记录来自旧版本，未保存逐项详情。请到系统回收站查找。".into()
                } else {
                    entry.failures.join("\n")
                })
                .into_any_element();
        }
        let items = entry.items;
        uniform_list(("history-files", index), items.len(), cx.processor(
            move |_, range: std::ops::Range<usize>, _, cx| range.map(|row| {
                let item = &items[row];
                let path = platform::display_path(&item.path);
                let clipboard = item.error.as_ref()
                    .map(|reason| format!("{}\n{path}\n{reason}", item.name))
                    .unwrap_or_else(|| path.clone());
                div().h(px(100.)).p(px(style::SPACE_12)).flex().items_center().gap(px(style::SPACE_12))
                    .child(icon(if item.error.is_some() { "info" } else { "check" },
                        if item.error.is_some() { t.review } else { t.safe }, 16.))
                    .child(div().flex_1().min_w_0().flex().flex_col().gap(px(style::SPACE_4))
                        .child(div().type_style(TextRole::Headline).child(item.name.clone()))
                        .child(div().type_style(TextRole::Metadata).text_color(rgb(t.muted))
                            .child(elided_middle(path, TextRole::Metadata)))
                        .child(div().type_style(TextRole::Callout).text_color(rgb(t.muted))
                            .overflow_hidden().child(elided(item.error.clone()
                                .map(|reason| format!("已跳过：{reason}"))
                                .unwrap_or_else(|| "已移入系统回收站；需要恢复时请在回收站按名称查找。".into()), TextRole::Callout))))
                    .child(button(("copy-history-path", row),
                        if item.error.is_some() { "复制详情" } else { "复制路径" }, "copy", t, false, true)
                        .on_click(cx.listener(move |_, _, _, cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(clipboard.clone()));
                        })))
                    .when_some(item.error.clone(), |e, reason| {
                        e.child(button(("history-full-error", row), "完整原因", "info", t, false, true)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.controller.model.notice = Some(reason.clone());
                                cx.notify();
                            })))
                    })
            }).collect()
        )).h(px((entry.moved.saturating_add(entry.failed).max(1) as f32 * 100.).min(300.)))
            .into_any_element()
    }
}
