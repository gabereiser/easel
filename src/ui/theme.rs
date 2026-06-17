/// DaVinci Resolve / ZBrush inspired dark theme.
pub struct Theme {
    pub background: [f32; 4],
    pub panel_bg: [f32; 4],
    pub panel_border: [f32; 4],
    pub widget_bg: [f32; 4],
    pub widget_hover: [f32; 4],
    pub widget_active: [f32; 4],
    pub accent: [f32; 4],
    pub accent_hover: [f32; 4],
    pub text_primary: [f32; 4],
    pub text_secondary: [f32; 4],
    pub text_disabled: [f32; 4],
    pub canvas_bg: [f32; 4],

    // Measurements
    pub corner_radius: f32,
    pub border_width: f32,
    pub toolbar_height: f32,
    pub palette_width: f32,
    pub drawer_width: f32,
    pub title_bar_height: f32,
    pub spacing: f32,
    pub padding: f32,

    // Title bar colors
    pub title_bg: [f32; 4],
    pub title_text: [f32; 4],
    pub title_button_hover: [f32; 4],
    pub close_button_hover: [f32; 4],
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            background: hex(0x16181E, 1.0),
            panel_bg: hex(0x1E2028, 1.0),
            panel_border: hex(0x2A2D3A, 1.0),
            widget_bg: hex(0x262833, 1.0),
            widget_hover: hex(0x303348, 1.0),
            widget_active: hex(0x3A3E55, 1.0),
            accent: hex(0x4F7CFF, 1.0),
            accent_hover: hex(0x6B92FF, 1.0),
            text_primary: hex(0xC8CAD4, 1.0),
            text_secondary: hex(0x8B8FA3, 1.0),
            text_disabled: hex(0x4A4D5E, 1.0),
            canvas_bg: hex(0x242631, 1.0),

            corner_radius: 6.0,
            border_width: 1.0,
            toolbar_height: 40.0,
            palette_width: 48.0,
            drawer_width: 260.0,
            title_bar_height: 32.0,
            spacing: 4.0,
            padding: 8.0,

            title_bg: hex(0x121317, 1.0),
            title_text: hex(0xC8CAD4, 1.0),
            title_button_hover: hex(0x303348, 1.0),
            close_button_hover: hex(0xE04949, 1.0),
        }
    }
}

const fn hex(rgb: u32, a: f32) -> [f32; 4] {
    let r = ((rgb >> 16) & 0xFF) as f32 / 255.0;
    let g = ((rgb >> 8) & 0xFF) as f32 / 255.0;
    let b = (rgb & 0xFF) as f32 / 255.0;
    [r, g, b, a]
}
