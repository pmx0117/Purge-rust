mod assets;
mod bootstrap;
mod components;
mod input;
mod menus;
mod style;
mod theme;
mod views;
use anyhow::Result;
pub use bootstrap::run;
use components::*;
use gpui::{prelude::*, *};
use input::TextInput;
use purge::{
    controller::AppController,
    format::*,
    launch::LaunchOptions,
    model::Page,
    platform,
    preferences::{Filter, Sort},
};
use purge_core::*;
use purge_engine::{jobs::Operation, storage::LocalRepository};
use std::{borrow::Cow, time::Duration};
use theme::Theme;
actions!(
    purge,
    [
        Rescan,
        ScanAll,
        Search,
        Overview,
        Caches,
        Developer,
        LargeFiles,
        Applications,
        Escape,
        AllItems,
        SafeItems,
        ReviewItems,
        NextControl,
        PreviousControl
    ]
);

struct PurgeView {
    controller: AppController,
    theme: Theme,
    native_dark: Option<bool>,
    highlighted: Option<Category>,
    sort_menu: bool,
    context_item: Option<(ItemGroup, Point<Pixels>)>,
    menu_position: Point<Pixels>,
    search: Entity<TextInput>,
    focus: FocusHandle,
    review_focus: [FocusHandle; 3],
    review_focused: bool,
    _tick: Task<()>,
    _search_subscription: Subscription,
}
impl PurgeView {
    fn new(cx: &mut Context<Self>) -> Self {
        let controller = AppController::new(
            LocalRepository::default(),
            LaunchOptions::parse(std::env::args()),
        );
        let theme = Theme::new(controller.appearance());
        let search = cx.new(TextInput::new);
        let subscription = cx.observe(&search, |_, _, cx| cx.notify());
        let tick = cx.spawn(async move |this, cx| {
            loop {
                smol::Timer::after(Duration::from_millis(120)).await;
                if this.update(cx, |this, cx| this.pump(cx)).is_err() {
                    break;
                }
            }
        });
        let mut view = Self {
            controller,
            theme,
            native_dark: None,
            highlighted: None,
            sort_menu: false,
            context_item: None,
            menu_position: Point::default(),
            search,
            focus: cx.focus_handle(),
            review_focus: [cx.focus_handle(), cx.focus_handle(), cx.focus_handle()],
            review_focused: false,
            _tick: tick,
            _search_subscription: subscription,
        };
        if view.controller.model.settings.scan_on_start {
            view.controller.start_scan(Category::ALL.to_vec());
        }
        view
    }
    fn busy(&self) -> bool {
        self.controller.busy()
    }
    fn total(&self, category: Category) -> u64 {
        self.controller.model.total(category)
    }
    fn safe_bytes(&self) -> u64 {
        self.controller.model.safe_bytes()
    }
    fn save(&mut self) {
        self.controller.save();
    }
    fn set_filter(&mut self, filter: Filter, cx: &mut Context<Self>) {
        self.controller.set_filter(filter);
        cx.notify();
    }
    fn set_sort(&mut self, sort: Sort, cx: &mut Context<Self>) {
        self.controller.set_sort(sort);
        cx.notify();
    }
    fn navigate(&mut self, page: Page, window: &mut Window, cx: &mut Context<Self>) {
        if !self.controller.navigate(page) {
            return;
        }
        self.context_item = None;
        self.sort_menu = false;
        self.highlighted = None;
        self.search.update(cx, |input, cx| input.clear(cx));
        self.focus.focus(window);
        cx.notify();
    }
    fn start_scan(&mut self, categories: Vec<Category>, cx: &mut Context<Self>) {
        self.controller.start_scan(categories);
        cx.notify();
    }
    fn rescan(&mut self, _: &Rescan, _: &mut Window, cx: &mut Context<Self>) {
        let categories = self
            .controller
            .model
            .page
            .scan_category()
            .map(|category| vec![category])
            .unwrap_or_else(|| Category::ALL.to_vec());
        self.start_scan(categories, cx);
    }
    fn scan_all(&mut self, _: &ScanAll, _: &mut Window, cx: &mut Context<Self>) {
        self.start_scan(Category::ALL.to_vec(), cx);
    }
    fn pump(&mut self, cx: &mut Context<Self>) {
        if self.controller.poll() {
            cx.notify();
        }
    }
    fn review_items(&mut self, items: Vec<Item>, cx: &mut Context<Self>) {
        self.controller.review_items(items);
        cx.notify();
    }
    fn review_safe(&mut self, cx: &mut Context<Self>) {
        self.controller.review_safe();
        cx.notify();
    }
    fn confirm_clean(&mut self, cx: &mut Context<Self>) {
        self.controller.confirm_cleanup();
        cx.notify();
    }
    fn duplicates(&mut self, cx: &mut Context<Self>) {
        self.controller.start_duplicates();
        cx.notify();
    }
    fn handle(&mut self, result: Result<()>, cx: &mut Context<Self>) {
        if let Err(error) = result {
            self.controller.model.notice = Some(error.to_string());
            cx.notify();
        }
    }
    fn choose_folder(&mut self, exclusion: bool, cx: &mut Context<Self>) {
        let picker = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: true,
            prompt: Some(
                if exclusion {
                    "选择不扫描的文件夹"
                } else {
                    "选择开发项目所在目录"
                }
                .into(),
            ),
        });
        cx.spawn(async move |this, cx| match picker.await {
            Ok(Ok(Some(paths))) => {
                let _ = this.update(cx, |this, cx| {
                    this.controller.add_directories(paths, exclusion);
                    cx.notify();
                });
            }
            Ok(Err(error)) => {
                let _ = this.update(cx, |this, cx| {
                    this.controller.model.notice = Some(error.to_string());
                    cx.notify();
                });
            }
            _ => {}
        })
        .detach();
    }
}
impl Render for PurgeView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.controller.model.review.is_some() && !self.review_focused {
            self.review_focus[0].focus(window);
            self.review_focused = true;
        } else if self.controller.model.review.is_none() {
            self.review_focused = false;
        }
        if self.native_dark != Some(self.theme.dark) {
            #[cfg(windows)]
            if let Ok(handle) = raw_window_handle::HasWindowHandle::window_handle(window)
                && let raw_window_handle::RawWindowHandle::Win32(handle) = handle.as_raw()
            {
                platform::apply_caption_theme(handle.hwnd.get(), self.theme.dark);
            }
            self.native_dark = Some(self.theme.dark);
        }
        let t = self.theme;
        self.search.update(cx, |input, _| {
            input.set_colors(
                if t.dark { 0x85827c } else { t.faint },
                if t.dark { 0x6cc3f0 } else { t.accent },
                if t.dark { 0x6cc3f045 } else { 0x85a9d44d },
            )
        });
        let body = match self.controller.model.page {
            Page::Overview => self.overview(cx),
            Page::Category(c) => self.category(c, window, cx),
            Page::Settings => self.settings_view(cx),
            Page::History => self.history_view(cx),
            Page::About => self.about(cx),
        };
        window.set_window_title("Purge · Windows");
        div()
            .size_full()
            .relative()
            .flex()
            .bg(rgb(t.base))
            .text_color(rgb(t.text))
            .font_family("Onest")
            .text_size(px(13.))
            .track_focus(&self.focus)
            .key_context("Purge")
            .on_action(cx.listener(Self::rescan))
            .on_action(cx.listener(Self::scan_all))
            .on_action(cx.listener(|this, _: &Search, w, cx| {
                if this.controller.model.review.is_none()
                    && matches!(
                        this.controller.model.page,
                        Page::Category(Category::LargeFiles | Category::Applications)
                    )
                {
                    this.search.focus_handle(cx).focus(w);
                }
            }))
            .on_action(cx.listener(|this, _: &AllItems, _, cx| this.set_filter(Filter::All, cx)))
            .on_action(cx.listener(|this, _: &SafeItems, _, cx| this.set_filter(Filter::Safe, cx)))
            .on_action(
                cx.listener(|this, _: &ReviewItems, _, cx| this.set_filter(Filter::Review, cx)),
            )
            .on_action(cx.listener(|this, _: &NextControl, w, _| {
                if this.controller.model.review.is_some() {
                    let index = if this.review_focus[0].is_focused(w) {
                        1
                    } else if this.review_focus[1].is_focused(w)
                        && this.controller.model.acknowledged()
                    {
                        2
                    } else {
                        0
                    };
                    this.review_focus[index].focus(w);
                } else {
                    w.focus_next();
                }
            }))
            .on_action(cx.listener(|this, _: &PreviousControl, w, _| {
                if this.controller.model.review.is_some() {
                    let index = if this.review_focus[2].is_focused(w) {
                        1
                    } else if this.review_focus[1].is_focused(w) {
                        0
                    } else if this.controller.model.acknowledged() {
                        2
                    } else {
                        1
                    };
                    this.review_focus[index].focus(w);
                } else {
                    w.focus_prev();
                }
            }))
            .on_action(
                cx.listener(|this, _: &Overview, w, cx| this.navigate(Page::Overview, w, cx)),
            )
            .on_action(cx.listener(|this, _: &Caches, w, cx| {
                this.navigate(Page::Category(Category::Caches), w, cx)
            }))
            .on_action(cx.listener(|this, _: &Developer, w, cx| {
                this.navigate(Page::Category(Category::Developer), w, cx)
            }))
            .on_action(cx.listener(|this, _: &LargeFiles, w, cx| {
                this.navigate(Page::Category(Category::LargeFiles), w, cx)
            }))
            .on_action(cx.listener(|this, _: &Applications, w, cx| {
                this.navigate(Page::Category(Category::Applications), w, cx)
            }))
            .on_action(cx.listener(|this, _: &Escape, w, cx| {
                let dismissing = this.controller.model.review.is_some()
                    || this.context_item.is_some()
                    || this.sort_menu;
                this.controller.model.review = None;
                this.context_item = None;
                this.sort_menu = false;
                if !dismissing && this.controller.operation() == Some(Operation::Scan) {
                    this.controller.cancel();
                }
                this.focus.focus(w);
                cx.notify();
            }))
            .child(self.sidebar(cx))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .max_w(px(style::PAGE_MAX_WIDTH))
                    .h_full()
                    .flex()
                    .flex_col()
                    .px(px(style::PAGE_INSET))
                    .pt(px(style::PAGE_TOP))
                    .child(self.header(cx))
                    .when_some(self.controller.model.notice.clone(), |e, s| {
                        e.child(
                            div()
                                .mb_3()
                                .p_3()
                                .bg(rgb(t.raised))
                                .rounded(px(4.))
                                .flex()
                                .items_start()
                                .gap_3()
                                .child(icon("info", t.muted, 14.))
                                .child(
                                    div()
                                        .id("notice-body")
                                        .flex_1()
                                        .max_h(px(120.))
                                        .overflow_y_scroll()
                                        .text_size(px(11.))
                                        .child(s),
                                )
                                .child(
                                    div()
                                        .id("dismiss-notice")
                                        .cursor_pointer()
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.controller.model.notice = None;
                                            cx.notify();
                                        }))
                                        .child(icon("x", t.faint, 10.)),
                                ),
                        )
                    })
                    .when(
                        !self.controller.model.warnings.is_empty()
                            && self.controller.model.notice.is_none(),
                        |e| {
                            e.child(
                                div()
                                    .id("scan-warning")
                                    .mb(px(12.))
                                    .p(px(12.))
                                    .rounded(px(14.))
                                    .bg(rgb(t.review_bg))
                                    .text_color(rgb(t.review))
                                    .text_size(px(12.))
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        let mut details = this
                                            .controller
                                            .model
                                            .warnings
                                            .iter()
                                            .take(20)
                                            .cloned()
                                            .collect::<Vec<_>>()
                                            .join("\n");
                                        if this.controller.model.warnings.len() > 20 {
                                            details.push_str(&format!(
                                                "\n另有 {} 项提示。",
                                                this.controller.model.warnings.len() - 20
                                            ));
                                        }
                                        this.controller.model.notice = Some(details);
                                        cx.notify();
                                    }))
                                    .child(format!(
                                        "{} 项未完成，查看详情",
                                        self.controller.model.warnings.len()
                                    )),
                            )
                        },
                    )
                    .child(body),
            )
            .when(self.context_item.is_some() || self.sort_menu, |e| {
                e.child(self.popup(window, cx))
            })
            .when_some(
                self.controller
                    .model
                    .review
                    .as_ref()
                    .map(|plan| plan.items.clone()),
                |e, items| e.child(self.review_modal(&items, cx)),
            )
    }
}
