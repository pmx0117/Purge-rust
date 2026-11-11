use purge_core::Category;

#[derive(Clone, Copy)]
pub(crate) struct Theme {
    pub(crate) dark: bool,
    pub(crate) accent: u32,
    pub(crate) brand: u32,
    pub(crate) base: u32,
    pub(crate) sidebar: u32,
    pub(crate) card: u32,
    pub(crate) raised: u32,
    pub(crate) fill: u32,
    pub(crate) fill_hover: u32,
    pub(crate) fill_pressed: u32,
    pub(crate) secondary: u32,
    pub(crate) selection: u32,
    pub(crate) selection_hover: u32,
    pub(crate) selection_text: u32,
    pub(crate) nav_hover: u32,
    pub(crate) row: u32,
    pub(crate) row_hover: u32,
    pub(crate) row_context: u32,
    pub(crate) toggle_on: u32,
    pub(crate) toggle_off: u32,
    pub(crate) toggle_thumb: u32,
    pub(crate) chart_used: u32,
    pub(crate) chart_free: u32,
    pub(crate) neutral_tile: u32,
    pub(crate) overlay: u32,
    pub(crate) text_selection: u32,
    pub(crate) border_strong: u32,
    pub(crate) border: u32,
    pub(crate) chrome_edge: u32,
    pub(crate) text: u32,
    pub(crate) muted: u32,
    pub(crate) faint: u32,
    pub(crate) action: u32,
    pub(crate) action_hover: u32,
    pub(crate) action_pressed: u32,
    pub(crate) on_action: u32,
    pub(crate) safe: u32,
    pub(crate) safe_bg: u32,
    pub(crate) review: u32,
    pub(crate) review_bg: u32,
}
impl Theme {
    pub(crate) fn chart_color(self, category: Category) -> u32 {
        match category {
            Category::Caches => {
                if self.dark {
                    0x6cc3f0
                } else {
                    0x56b4e9
                }
            }
            Category::Developer => {
                if self.dark {
                    0x3a94d0
                } else {
                    0x0072b2
                }
            }
            Category::LargeFiles => {
                if self.dark {
                    0xf0b020
                } else {
                    0xe69f00
                }
            }
            Category::Applications => {
                if self.dark {
                    0xe8772a
                } else {
                    0xd55e00
                }
            }
        }
    }

    pub(crate) fn tile_color(self, category: Category) -> u32 {
        match category {
            Category::Caches => {
                if self.dark {
                    0x3a9bd5
                } else {
                    0x3399d3
                }
            }
            Category::Developer => {
                if self.dark {
                    0x1e78ba
                } else {
                    0x0072b2
                }
            }
            Category::LargeFiles => 0xe69f00,
            Category::Applications => 0xd55e00,
        }
    }

    pub(crate) fn destructive(mut self) -> Self {
        self.action = if self.dark { 0xd9372d } else { 0xd1312a };
        self.action_hover = if self.dark { 0xe2453b } else { 0xbc2b25 };
        self.action_pressed = if self.dark { 0xc4302a } else { 0xa92620 };
        self.on_action = 0xffffff;
        self
    }

    pub(crate) fn new(dark: bool) -> Self {
        if dark {
            Self {
                dark: true,
                accent: 0xeceae6,
                brand: purge::brand::COLOR,
                base: 0x171615,
                sidebar: 0x1e1d1c,
                card: 0x1e1d1c,
                raised: 0x2b2a28,
                fill: 0x282725,
                fill_hover: 0x2f2e2c,
                fill_pressed: 0x373533,
                secondary: 0x282725,
                selection: 0x282725,
                selection_hover: 0x282725,
                selection_text: 0xeceae6,
                nav_hover: 0x2b2a28,
                row: 0x282725,
                row_hover: 0x2a2927,
                row_context: 0x373533,
                toggle_on: 0x5a5853,
                toggle_off: 0x41403d,
                toggle_thumb: 0xeceae6,
                chart_used: 0xa8a49d,
                chart_free: 0x4e4b47,
                neutral_tile: 0x8c8983,
                overlay: 0x00000088,
                text_selection: 0xeceae638,
                border_strong: 0x3d3b38,
                border: 0x302f2d,
                chrome_edge: 0x2d2b29,
                text: 0xeceae6,
                muted: 0xa6a39d,
                faint: 0x807d78,
                action: 0xeceae6,
                action_hover: 0xf6f4f1,
                action_pressed: 0xd8d5d0,
                on_action: 0x171615,
                safe: 0x5fd36b,
                safe_bg: 0x1b2e22,
                review: 0xf2b84b,
                review_bg: 0x332910,
            }
        } else {
            Self {
                dark: false,
                accent: 0x353431,
                brand: purge::brand::COLOR,
                base: 0xf6f5f3,
                sidebar: 0xffffff,
                card: 0xffffff,
                raised: 0xffffff,
                fill: 0xefedea,
                fill_hover: 0xe8e6e2,
                fill_pressed: 0xdfdcd8,
                secondary: 0xefedea,
                selection: 0xefedea,
                selection_hover: 0xefedea,
                selection_text: 0x2e2d2b,
                nav_hover: 0xf4f2ef,
                row: 0xefedea,
                row_hover: 0xf4f2ef,
                row_context: 0xdfdcd8,
                toggle_on: 0x353431,
                toggle_off: 0xd5d2cd,
                toggle_thumb: 0xffffff,
                chart_used: 0x8c8983,
                chart_free: 0xdad7d2,
                neutral_tile: 0x8c8983,
                overlay: 0x17161566,
                text_selection: 0x35343128,
                border_strong: 0xd5d2cd,
                border: 0xe6e3df,
                chrome_edge: 0xe7e4df,
                text: 0x2e2d2b,
                muted: 0x64625e,
                faint: 0x85827c,
                action: 0x353431,
                action_hover: 0x42403d,
                action_pressed: 0x2a2927,
                on_action: 0xffffff,
                safe: 0x1f7a35,
                safe_bg: 0xe6f4ea,
                review: 0x8a5300,
                review_bg: 0xfaeeda,
            }
        }
    }
}
