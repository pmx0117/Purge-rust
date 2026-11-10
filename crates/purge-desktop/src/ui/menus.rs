use super::*;

impl PurgeView {
    pub(super) fn popup(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme;
        let point = if self.sort_menu {
            self.menu_position
        } else {
            self.context_item
                .as_ref()
                .map(|(_, p)| *p)
                .unwrap_or_default()
        };
        let viewport = window.viewport_size();
        let mut menu = div()
            .id("popup-menu")
            .occlude()
            .absolute()
            .left(point.x.min(viewport.width - px(244.)).max(px(8.)))
            .top(point.y.min(viewport.height - px(210.)).max(px(8.)))
            .w(px(236.))
            .p(px(5.))
            .rounded(px(8.))
            .bg(rgb(t.raised))
            .border_1()
            .border_color(rgb(t.border_strong))
            .shadow_md()
            .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                this.context_item = None;
                this.sort_menu = false;
                cx.stop_propagation();
                cx.notify();
            }));
        if self.sort_menu {
            for (index, sort) in [
                Sort::Size,
                Sort::Smallest,
                Sort::Newest,
                Sort::Oldest,
                Sort::Name,
            ]
            .into_iter()
            .enumerate()
            {
                menu = menu.child(
                    div()
                        .id(("sort-option", index))
                        .h(px(30.))
                        .px(px(8.))
                        .rounded(px(4.))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .cursor_pointer()
                        .hover(move |e| e.bg(rgb(t.fill_hover)))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.set_sort(sort, cx);
                            this.sort_menu = false;
                            cx.notify();
                        }))
                        .child(
                            div()
                                .size(px(12.))
                                .when(self.controller.model.sort == sort, |e| {
                                    e.child(icon("check", t.text, 12.))
                                }),
                        )
                        .child(sort.label()),
                );
            }
        } else if let Some((group, _)) = &self.context_item {
            let item = &group.items[0];
            menu = menu.child(
                div()
                    .px(px(8.))
                    .py(px(6.))
                    .text_size(px(11.))
                    .text_color(rgb(t.muted))
                    .text_ellipsis()
                    .child(group.name.clone()),
            );
            for (index, (title, glyph)) in [
                ("在资源管理器中显示", "external"),
                ("复制完整路径", "copy"),
                ("不再扫描此项目", "x"),
            ]
            .into_iter()
            .enumerate()
            {
                if index == 2 && item.is_application() {
                    continue;
                }
                let selected = group.clone();
                menu = menu.child(
                    div()
                        .id(("context-option", index))
                        .h(px(30.))
                        .px(px(8.))
                        .rounded(px(4.))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .cursor_pointer()
                        .hover(move |e| e.bg(rgb(t.fill_hover)))
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
                                        .map(|i| platform::display_path(&i.path))
                                        .collect::<Vec<_>>()
                                        .join("\n"),
                                )),
                                _ => this.controller.exclude_group(&selected),
                            }
                            cx.notify();
                        }))
                        .child(icon(glyph, t.muted, 12.))
                        .child(title),
                );
            }
        }
        menu.into_any_element()
    }
}
