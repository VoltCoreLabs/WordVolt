use gpui_kit::rgb;

use super::Theme;

impl Theme {
    pub fn light() -> Self {
        Self {
            bg: rgb(0xF3F4F6),
            surface: rgb(0xFFFFFF),
            surface_alt: rgb(0xF9FAFB),
            border: rgb(0xD1D5DB),
            border_strong: rgb(0x9CA3AF),
            text: rgb(0x111827),
            text_muted: rgb(0x6B7280),
            accent: rgb(0x2563EB),
            accent_hover: rgb(0x1D4ED8),
            accent_subtle: rgb(0xDBEAFE),
            on_accent: rgb(0xFFFFFF),
            danger: rgb(0xDC2626),
            page: rgb(0xFFFFFF),
        }
    }
}
