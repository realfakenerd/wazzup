use gpui::rgb;

pub struct ThemeColors;

impl ThemeColors {
    // Base backgrounds
    pub const BASE: u32 = 0x181825;
    pub const MANTLE: u32 = 0x1e1e2e;
    pub const CRUST: u32 = 0x11111b;
    pub const SURFACE: u32 = 0x313244;
    pub const SURFACE0: u32 = 0x313244;
    pub const SURFACE1: u32 = 0x45475a;
    pub const OVERLAY: u32 = 0x45475a;

    // WhatsApp Emerald green accents
    pub const EMERALD: u32 = 0x25D366;
    pub const GREEN_DARK: u32 = 0x005c4b;
    pub const GREEN_MUTED: u32 = 0x128C7E;

    // Message status ticks
    pub const TICK_CYAN: u32 = 0x34B7F1;
    pub const TICK_GRAY: u32 = 0x6c7086;

    // Typography
    pub const TEXT_PRIMARY: u32 = 0xcdd6f4;
    pub const TEXT_MUTED: u32 = 0xa6adc8;
    pub const TEXT_DIM: u32 = 0x6c7086;

    // Borders & Focus rings
    pub const BORDER: u32 = 0x313244;
    pub const FOCUS_RING: u32 = 0x25D366;

    // Helper functions returning gpui::Hsla / Rgba
    pub fn base() -> gpui::Rgba {
        rgb(Self::BASE)
    }

    pub fn mantle() -> gpui::Rgba {
        rgb(Self::MANTLE)
    }

    pub fn surface() -> gpui::Rgba {
        rgb(Self::SURFACE)
    }

    pub fn emerald() -> gpui::Rgba {
        rgb(Self::EMERALD)
    }

    pub fn text_primary() -> gpui::Rgba {
        rgb(Self::TEXT_PRIMARY)
    }

    pub fn text_muted() -> gpui::Rgba {
        rgb(Self::TEXT_MUTED)
    }
}
