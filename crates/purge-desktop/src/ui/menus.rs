use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Picker {
    Sort,
    FileType,
    Threshold,
    StaleDays,
}

#[derive(Clone, Copy)]
enum Choice {
    Sort(Sort),
    FileType(Option<FileKind>),
    Threshold(u64),
    StaleDays(u64),
}

impl PurgeView {
    pub(super) fn open_picker(
        &mut self,
        picker: Picker,
        point: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        if self.busy() && matches!(picker, Picker::Threshold | Picker::StaleDays) {
            return;
        }
        self.context_item = None;
        self.menu_position = point + gpui::point(px(0.), px(12.));
        self.picker = (self.picker != Some(picker)).then_some(picker);
        cx.notify();
    }

    fn picker_choices(&self, picker: Picker) -> Vec<(String, Choice)> {
        match picker {
            Picker::Sort => Sort::all()
                .map(|sort| (sort.label().into(), Choice::Sort(sort)))
                .collect(),
            Picker::FileType => std::iter::once(None)
                .chain(FileKind::all().map(Some))
                .map(|kind| {
                    (
                        kind.map(FileKind::label).unwrap_or("所有类型").into(),
                        Choice::FileType(kind),
                    )
                })
                .collect(),
            Picker::Threshold => [5, 25, 100, 500, 1024]
                .into_iter()
                .map(|size| (format!("{size} MB"), Choice::Threshold(size)))
                .collect(),
            Picker::StaleDays => [0, 30, 90, 180, 365]
                .into_iter()
                .map(|days| (stale_days_label(days), Choice::StaleDays(days)))
                .collect(),
        }
    }

    fn choice_selected(&self, choice: Choice) -> bool {
        let model = &self.controller.model;
        match choice {
            Choice::Sort(sort) => model.sort == sort,
            Choice::FileType(kind) => model.kind_filter == kind,
            Choice::Threshold(size) => model.settings.scan.minimum_file_mb == size,
            Choice::StaleDays(days) => model.settings.scan.stale_project_days == days,
        }
    }

    fn choose_value(&mut self, choice: Choice, cx: &mut Context<Self>) {
        self.picker = None;
        match choice {
            Choice::Sort(sort) => self.set_sort(sort, cx),
            Choice::FileType(kind) => self.controller.model.kind_filter = kind,
            Choice::Threshold(size) => {
                if !self.busy() {
                    self.controller.model.settings.scan.minimum_file_mb = size;
                    self.save();
                }
            }
            Choice::StaleDays(days) => {
                if !self.busy() {
                    self.controller.model.settings.scan.stale_project_days = days;
                    self.save();
                }
            }
        }
        cx.notify();
    }

    pub(super) fn popup(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme;
        let choices = self.picker.map(|picker| self.picker_choices(picker));
        let position = self.picker.map(|_| self.menu_position).unwrap_or_else(|| {
            self.context_item
                .as_ref()
                .map(|(_, point)| *point)
                .unwrap_or_default()
        });
        let height = choices
            .as_ref()
            .map(|choices| choices.len() as f32 * 28. + 8.)
            .unwrap_or(134.);
        let viewport = window.viewport_size();
        let menu = div()
            .id("popup-menu")
            .occlude()
            .absolute()
            .left(position.x.min(viewport.width - px(228.)).max(px(8.)))
            .top(
                position
                    .y
                    .min(viewport.height - px(height + 8.))
                    .max(px(8.)),
            )
            .w(px(220.))
            .max_h(viewport.height - px(16.))
            .overflow_y_scroll()
            .p(px(style::SPACE_4))
            .rounded(px(style::FIELD_RADIUS))
            .bg(rgb(t.raised))
            .border(px(0.5))
            .border_color(rgb(t.border_strong))
            .shadow_md()
            .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                this.context_item = None;
                this.picker = None;
                cx.stop_propagation();
                cx.notify();
            }));
        let menu = match choices {
            Some(choices) => menu.children(choices.into_iter().enumerate().map(
                |(index, (label, choice))| {
                    self.menu_row(("picker-option", index), cx)
                        .on_click(cx.listener(move |this, _, _, cx| this.choose_value(choice, cx)))
                        .child(div().size(px(14.)).when(self.choice_selected(choice), |e| {
                            e.child(icon("check", t.text, 12.))
                        }))
                        .child(label)
                },
            )),
            None => self.context_menu(menu, cx),
        };
        menu.into_any_element()
    }

    fn menu_row(&self, id: impl Into<ElementId>, _cx: &mut Context<Self>) -> Stateful<Div> {
        let t = self.theme;
        div()
            .id(id)
            .h(px(28.))
            .px(px(style::SPACE_8))
            .rounded(px(6.))
            .type_style(TextRole::RowTitle)
            .text_color(rgb(t.text))
            .flex()
            .items_center()
            .gap(px(style::SPACE_8))
            .cursor_pointer()
            .hover(move |e| e.bg(rgb(t.fill_hover)))
    }

    fn context_menu(&self, mut menu: Stateful<Div>, cx: &mut Context<Self>) -> Stateful<Div> {
        let t = self.theme;
        let Some((group, _)) = &self.context_item else {
            return menu;
        };
        menu = menu.child(
            div()
                .px(px(style::SPACE_8))
                .py(px(style::SPACE_6))
                .type_style(TextRole::Metadata)
                .text_color(rgb(t.muted))
                .overflow_hidden()
                .child(elided(group.name.clone(), TextRole::Metadata)),
        );
        for (index, (title, glyph)) in [
            ("在资源管理器中显示", "external"),
            ("复制完整路径", "copy"),
            ("不再扫描此项目", "x"),
        ]
        .into_iter()
        .enumerate()
        {
            if index == 2 && group.items[0].is_application() {
                continue;
            }
            let selected = group.clone();
            menu = menu.child(
                self.menu_row(("context-option", index), cx)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.context_item = None;
                        match index {
                            0 => this.handle(
                                platform::reveal(
                                    &selected
                                        .items
                                        .iter()
                                        .max_by_key(|item| item.bytes())
                                        .expect("nonempty cache group")
                                        .path,
                                ),
                                cx,
                            ),
                            1 => cx.write_to_clipboard(ClipboardItem::new_string(
                                selected
                                    .items
                                    .iter()
                                    .map(|item| platform::display_path(&item.path))
                                    .collect::<Vec<_>>()
                                    .join("\n"),
                            )),
                            _ => this.controller.exclude_group(&selected),
                        }
                        cx.notify();
                    }))
                    .child(icon(glyph, t.muted, 14.))
                    .child(title),
            );
        }
        menu
    }
}

pub(super) fn stale_days_label(days: u64) -> String {
    if days == 0 {
        "显示全部".into()
    } else {
        format!("{days} 天")
    }
}
