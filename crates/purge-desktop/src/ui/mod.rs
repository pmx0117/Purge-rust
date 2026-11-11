mod app_icons;
mod assets;
mod bootstrap;
mod chrome;
mod components;
mod input;
mod label;
mod menus;
mod shell;
mod style;
mod theme;
mod tray;
mod views;
use anyhow::Result;
pub use bootstrap::run;
use components::*;
use gpui::{prelude::*, *};
use input::TextInput;
use label::{elided, elided_middle};
use menus::Picker;
use purge::{
    controller::AppController,
    format::*,
    launch::LaunchOptions,
    model::Page,
    platform,
    preferences::{AppLayout, Filter, Sort},
};
use purge_core::*;
use purge_engine::{jobs::Operation, storage::LocalRepository};
use std::{borrow::Cow, time::Duration};
use style::{TextRole, TypographyExt};
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
        PreviousControl,
        MinimizeToTray,
        QuitApplication
    ]
);

struct PurgeView {
    controller: AppController,
    theme: Theme,
    native_dark: Option<bool>,
    #[cfg(windows)]
    native_icon_scale: Option<f32>,
    highlighted: Option<Category>,
    picker: Option<Picker>,
    context_item: Option<(ItemGroup, Point<Pixels>)>,
    menu_position: Point<Pixels>,
    search: Entity<TextInput>,
    focus: FocusHandle,
    review_focus: [FocusHandle; 3],
    review_focused: bool,
    app_icons: std::collections::HashMap<u64, ImageSource>,
    _icon_task: Option<Task<()>>,
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
            #[cfg(windows)]
            native_icon_scale: None,
            highlighted: None,
            picker: None,
            context_item: None,
            menu_position: Point::default(),
            search,
            focus: cx.focus_handle(),
            review_focus: [cx.focus_handle(), cx.focus_handle(), cx.focus_handle()],
            review_focused: false,
            app_icons: Default::default(),
            _icon_task: None,
            _tick: tick,
            _search_subscription: subscription,
        };
        view.load_app_icons(cx);
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
        self.picker = None;
        self.highlighted = None;
        self.search.update(cx, |input, cx| input.clear(cx));
        self.focus.focus(window, cx);
        cx.notify();
    }
    fn start_scan(&mut self, categories: Vec<Category>, cx: &mut Context<Self>) {
        let includes_apps = categories.contains(&Category::Applications);
        if self.controller.start_scan(categories) && includes_apps {
            self.load_app_icons(cx);
        }
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
            self.review_focus[0].focus(window, cx);
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
        #[cfg(windows)]
        if self.native_icon_scale != Some(window.scale_factor())
            && let Ok(handle) = raw_window_handle::HasWindowHandle::window_handle(window)
            && let raw_window_handle::RawWindowHandle::Win32(handle) = handle.as_raw()
        {
            if let Err(error) = platform::apply_window_icons(handle.hwnd.get()) {
                self.controller.model.notice = Some(format!("系统图标加载失败：{error}"));
            }
            self.native_icon_scale = Some(window.scale_factor());
        }
        let t = self.theme;
        let placeholder = if self.controller.model.page == Page::Category(Category::Applications) {
            "搜索应用"
        } else {
            "搜索名称、路径或类型…"
        };
        self.search.update(cx, |input, _| {
            input.set_placeholder(placeholder);
            input.set_colors(t.faint, t.accent, t.text_selection)
        });
        let body = match self.controller.model.page {
            Page::Overview => self.overview(cx),
            Page::Category(Category::Applications) => self.applications_view(window, cx),
            Page::Category(c) => self.category(c, window, cx),
            Page::Settings => self.settings_view(cx),
            Page::History => self.history_view(cx),
            Page::About => self.about(cx),
        };
        let show_chrome = purge::lifecycle::custom_caption(
            purge_engine::platform::OperatingSystem::current(),
            window.is_fullscreen(),
            matches!(window.window_decorations(), Decorations::Client { .. }),
        );
        div()
            .size_full()
            .relative()
            .flex()
            .when(show_chrome, |e| e.pt(px(chrome::HEIGHT)))
            .bg(rgb(t.base))
            .text_color(rgb(t.text))
            .type_style(TextRole::Body)
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
                    this.search.focus_handle(cx).focus(w, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &AllItems, _, cx| this.set_filter(Filter::All, cx)))
            .on_action(cx.listener(|this, _: &SafeItems, _, cx| this.set_filter(Filter::Safe, cx)))
            .on_action(
                cx.listener(|this, _: &ReviewItems, _, cx| this.set_filter(Filter::Review, cx)),
            )
            .on_action(cx.listener(|this, _: &NextControl, w, cx| {
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
                    this.review_focus[index].focus(w, cx);
                } else {
                    w.focus_next(cx);
                }
            }))
            .on_action(cx.listener(|this, _: &PreviousControl, w, cx| {
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
                    this.review_focus[index].focus(w, cx);
                } else {
                    w.focus_prev(cx);
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
                    || this.picker.is_some();
                this.controller.model.review = None;
                this.context_item = None;
                this.picker = None;
                if !dismissing && this.controller.operation() == Some(Operation::Scan) {
                    this.controller.cancel();
                }
                this.focus.focus(w, cx);
                cx.notify();
            }))
            .child(self.sidebar(chrome::hairline(window), !show_chrome, cx))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .flex()
                    .flex_col()
                    .px(px(style::PAGE_INSET))
                    .pt(px(style::PAGE_TOP))
                    .child(self.header(cx))
                    .when_some(self.controller.model.notice.clone(), |e, s| {
                        e.child(
                            div()
                                .mb(px(style::SPACE_12))
                                .p(px(style::SPACE_12))
                                .bg(rgb(t.raised))
                                .rounded(px(style::CARD_RADIUS))
                                .flex()
                                .items_start()
                                .gap(px(style::SPACE_12))
                                .child(icon("info", t.muted, 14.))
                                .child(
                                    div()
                                        .id("notice-body")
                                        .flex_1()
                                        .max_h(px(120.))
                                        .overflow_y_scroll()
                                        .type_style(TextRole::Callout)
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
                                    .mb(px(style::SPACE_12))
                                    .p(px(style::SPACE_12))
                                    .rounded(px(style::CARD_RADIUS))
                                    .bg(rgb(t.review_bg))
                                    .text_color(rgb(t.review))
                                    .type_style(TextRole::Callout)
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
            .when(self.context_item.is_some() || self.picker.is_some(), |e| {
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
            .when(show_chrome, |e| e.child(chrome::titlebar(t, window, cx)))
            .when(
                cfg!(target_os = "linux") && show_chrome && !window.is_maximized(),
                |e| e.child(chrome::resize_handles(t, window)),
            )
    }
}
