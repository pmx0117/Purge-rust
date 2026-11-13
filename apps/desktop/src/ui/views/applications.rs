use super::*;

const CARD_HEIGHT: f32 = 130.;
const MIN_CARD_WIDTH: f32 = 146.;
const LIST_ROW_HEIGHT: f32 = 68.;

impl PurgeView {
    pub(in crate::ui) fn applications_view(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let items = self.controller.model.visible(&self.search.read(cx).content);
        let layout = self.controller.model.settings.app_layout;
        let content = if items.is_empty() {
            self.empty_applications()
        } else {
            match layout {
                AppLayout::Cards => self.application_grid(items, window, cx),
                AppLayout::List => self.application_list(items, cx),
            }
        };
        div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .child(self.application_toolbar(window, cx))
            .child(content)
            .child(
                div()
                    .flex_shrink_0()
                    .pt(px(style::SPACE_8))
                    .type_style(TextRole::Callout)
                    .bg(rgb(self.theme.base))
                    .text_color(rgb(self.theme.muted))
                    .child("管理与卸载将在系统应用管理器中完成。"),
            )
            .into_any_element()
    }

    fn application_toolbar(&self, window: &Window, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let focused = self.search.focus_handle(cx).is_focused(window);
        div()
            .flex_shrink_0()
            .flex()
            .items_center()
            .gap(px(style::SPACE_12))
            .pb(px(style::SPACE_12))
            .child(
                button(
                    "apps-sort",
                    self.controller.model.sort.label(),
                    "sort",
                    t,
                    false,
                    true,
                )
                .on_click(cx.listener(|this, event: &ClickEvent, w, cx| {
                    this.open_picker(Picker::Sort, event.position(), w, cx)
                })),
            )
            .child(div().flex_1())
            .child(
                div()
                    .w(px(220.))
                    .min_w_0()
                    .h(px(28.))
                    .px(px(style::SPACE_12))
                    .rounded(px(style::FIELD_RADIUS))
                    .bg(rgb(t.fill))
                    .border(px(0.5))
                    .border_color(rgb(if focused { t.accent } else { t.border }))
                    .flex()
                    .items_center()
                    .gap(px(style::SPACE_8))
                    .child(icon("search", t.muted, 14.))
                    .child(self.search.clone()),
            )
            .child(self.application_layout_switcher(cx))
    }

    fn application_layout_switcher(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        div()
            .h(px(30.))
            .p(px(style::SPACE_2))
            .rounded(px(style::FIELD_RADIUS))
            .bg(rgb(t.fill))
            .border(px(0.5))
            .border_color(rgb(t.border))
            .flex()
            .items_center()
            .children(
                [
                    (AppLayout::List, "list", "列表"),
                    (AppLayout::Cards, "overview", "卡片"),
                ]
                .into_iter()
                .enumerate()
                .map(|(index, (layout, glyph, label))| {
                    let selected = self.controller.model.settings.app_layout == layout;
                    let color = if selected { t.text } else { t.muted };
                    div()
                        .id(("app-layout", index))
                        .focusable()
                        .tab_index(0)
                        .focus(move |e| e.border_1().border_color(rgb(t.accent)))
                        .h(px(24.))
                        .px(px(style::SPACE_8))
                        .rounded(px(6.))
                        .flex()
                        .items_center()
                        .gap(px(style::SPACE_6))
                        .type_style(TextRole::Callout)
                        .text_color(rgb(color))
                        .when(selected, |e| {
                            e.bg(rgb(t.raised)).font_weight(FontWeight::SEMIBOLD)
                        })
                        .cursor_pointer()
                        .hover(move |e| e.bg(rgb(t.fill_hover)))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.controller.set_app_layout(layout);
                            cx.notify();
                        }))
                        .child(icon(glyph, color, 14.))
                        .child(label)
                }),
            )
    }

    fn application_grid(
        &self,
        items: Vec<Item>,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let pane_width = (f32::from(window.viewport_size().width)
            - style::SIDEBAR_WIDTH
            - style::PAGE_INSET * 2.)
            .max(0.);
        let columns = ((pane_width + style::SPACE_12) / (MIN_CARD_WIDTH + style::SPACE_12))
            .floor()
            .max(1.) as usize;
        let width =
            ((pane_width - (columns - 1) as f32 * style::SPACE_12) / columns as f32).max(1.);
        let list = uniform_list(
            ("applications-cards", columns),
            items.len().div_ceil(columns),
            cx.processor(move |this, range: std::ops::Range<usize>, _, cx| {
                range
                    .map(|row| {
                        div()
                            .h(px(CARD_HEIGHT + style::SPACE_12))
                            .pb(px(style::SPACE_12))
                            .flex()
                            .gap(px(style::SPACE_12))
                            .children(
                                items
                                    .iter()
                                    .skip(row * columns)
                                    .take(columns)
                                    .map(|item| this.application_card(item, width, cx)),
                            )
                    })
                    .collect()
            }),
        )
        .flex_1()
        .min_h_0()
        .into_any_element();
        application_viewport(list)
    }

    fn application_card(&self, item: &Item, width: f32, cx: &mut Context<Self>) -> Stateful<Div> {
        let t = self.theme;
        let context_active = self
            .context_item
            .as_ref()
            .is_some_and(|(group, _)| group.id == item.id);
        let for_menu = item.clone();
        let name = item.name.clone();
        div()
            .id(("application-card", item.id))
            .focusable()
            .tab_index(0)
            .focus(move |e| e.border_color(rgb(t.accent)))
            .w(px(width))
            .h(px(CARD_HEIGHT))
            .p(px(style::SPACE_16))
            .rounded(px(style::CARD_RADIUS))
            .border(px(0.5))
            .border_color(rgb(t.border))
            .bg(rgb(if context_active {
                t.row_context
            } else {
                t.fill
            }))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(style::SPACE_8))
            .cursor_pointer()
            .hover(move |e| e.bg(rgb(t.raised)))
            .active(move |e| e.bg(rgb(t.fill_pressed)))
            .on_click(cx.listener(move |this, _, _, cx| this.manage_application(&name, cx)))
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(move |this, event: &MouseDownEvent, w, cx| {
                    this.open_application_context(for_menu.clone(), event.position, w, cx);
                }),
            )
            .child(self.application_icon(item, 56.))
            .child(
                div()
                    .w_full()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(style::SPACE_2))
                    .child(
                        div()
                            .w_full()
                            .type_style(TextRole::RowTitle)
                            .text_center()
                            .overflow_hidden()
                            .child(elided_middle(item.name.clone(), TextRole::RowTitle)),
                    )
                    .child(
                        div()
                            .type_style(TextRole::Metadata)
                            .numeric()
                            .text_color(rgb(t.muted))
                            .child(application_size(item)),
                    ),
            )
    }

    fn application_list(&self, items: Vec<Item>, cx: &mut Context<Self>) -> AnyElement {
        let list = uniform_list(
            "applications-list",
            items.len(),
            cx.processor(move |this, range: std::ops::Range<usize>, _, cx| {
                range
                    .map(|index| this.application_list_row(&items[index], cx))
                    .collect()
            }),
        )
        .flex_1()
        .min_h_0()
        .into_any_element();
        application_viewport(list)
    }

    fn application_list_row(&self, item: &Item, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let for_menu = item.clone();
        let name = item.name.clone();
        div().h(px(LIST_ROW_HEIGHT)).pb(px(style::SPACE_8)).child(
            div()
                .id(("application-row", item.id))
                .focusable()
                .tab_index(0)
                .focus(move |e| e.border_color(rgb(t.accent)))
                .size_full()
                .px(px(style::ROW_PADDING))
                .py(px(style::SPACE_12))
                .rounded(px(style::CARD_RADIUS))
                .bg(rgb(t.card))
                .border(px(0.5))
                .border_color(rgb(t.border))
                .flex()
                .items_center()
                .gap(px(style::SPACE_12))
                .cursor_pointer()
                .hover(move |e| e.bg(rgb(t.row_hover)))
                .on_click(cx.listener(move |this, _, _, cx| this.manage_application(&name, cx)))
                .on_mouse_down(
                    MouseButton::Right,
                    cx.listener(move |this, event: &MouseDownEvent, w, cx| {
                        this.open_application_context(for_menu.clone(), event.position, w, cx);
                    }),
                )
                .child(self.application_icon(item, 32.))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(
                            div()
                                .type_style(TextRole::Headline)
                                .overflow_hidden()
                                .child(elided(item.name.clone(), TextRole::Headline)),
                        )
                        .when(!item.description.is_empty(), |e| {
                            e.child(
                                div()
                                    .mt(px(style::SPACE_2))
                                    .type_style(TextRole::Metadata)
                                    .text_color(rgb(t.muted))
                                    .overflow_hidden()
                                    .child(elided(item.description.clone(), TextRole::Metadata)),
                            )
                        }),
                )
                .child(
                    div()
                        .type_style(TextRole::MetadataEmphasis)
                        .numeric()
                        .text_color(rgb(t.muted))
                        .child(application_size(item)),
                ),
        )
    }

    fn open_application_context(
        &mut self,
        item: Item,
        point: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.context_item = Some((
            ItemGroup {
                id: item.id,
                name: item.name.clone(),
                items: vec![item],
            },
            point,
        ));
        self.picker = None;
        self.focus_menu(3, window, cx);
        cx.notify();
    }

    fn manage_application(&mut self, name: &str, cx: &mut Context<Self>) {
        match platform::open_apps_settings() {
            Ok(()) => {
                self.controller.model.notice = Some(format!(
                    "已打开系统应用管理器，请搜索“{name}”。右键应用可复制名称。"
                ))
            }
            Err(error) => self.controller.model.notice = Some(error.to_string()),
        }
        cx.notify();
    }

    fn empty_applications(&self) -> AnyElement {
        let t = self.theme;
        div()
            .flex_1()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(style::SPACE_8))
            .child(icon("apps", t.faint, 36.))
            .child(div().type_style(TextRole::Title).child(if self.busy() {
                "正在读取应用"
            } else {
                "未找到应用"
            }))
            .child(
                div()
                    .type_style(TextRole::Callout)
                    .text_color(rgb(t.muted))
                    .child("调整搜索条件，或重新扫描。"),
            )
            .into_any_element()
    }
}

fn application_size(item: &Item) -> String {
    if item.bytes() == 0 {
        "大小未知".into()
    } else {
        format_bytes(item.bytes())
    }
}

/// Keep the list's own overflow-y scroll state intact; clipping belongs to
/// its viewport. Replacing UniformList overflow with Hidden clears that state.
fn application_viewport(list: AnyElement) -> AnyElement {
    div()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .overflow_hidden()
        .child(list)
        .into_any_element()
}
