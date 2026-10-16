use gpui_kit::*;

/// Phosphor icons we use. Files live in `assets/icons/phosphor/` (add new ones there).
#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)] // catalogue: icons are added ahead of use
pub enum Icon {
    TextB,
    TextItalic,
    TextUnderline,
    TextAa,
    AlignLeft,
    AlignCenter,
    AlignRight,
    AlignJustify,
    ListBullets,
    ListNumbers,
    FloppyDisk,
    FolderOpen,
    FilePlus,
    Undo,
    Redo,
    Search,
    CaretDown,
    CaretUp,
    Image,
    Table,
    Link,
    Printer,
    Highlighter,
    PaintBucket,
    Minimize,
    Maximize,
    Restore,
    Close,
    Strikethrough,
    Superscript,
    Subscript,
    Code,
    EyeSlash,
    Quotes,
    TextColor,
}

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum Weight {
    Regular,
    Bold,
    Fill,
}

impl Icon {
    fn file(self) -> &'static str {
        match self {
            Icon::TextB => "text-b",
            Icon::TextItalic => "text-italic",
            Icon::TextUnderline => "text-underline",
            Icon::TextAa => "text-aa",
            Icon::AlignLeft => "text-align-left",
            Icon::AlignCenter => "text-align-center",
            Icon::AlignRight => "text-align-right",
            Icon::AlignJustify => "text-align-justify",
            Icon::ListBullets => "list-bullets",
            Icon::ListNumbers => "list-numbers",
            Icon::FloppyDisk => "floppy-disk",
            Icon::FolderOpen => "folder-open",
            Icon::FilePlus => "file-plus",
            Icon::Undo => "arrow-counter-clockwise",
            Icon::Redo => "arrow-clockwise",
            Icon::Search => "magnifying-glass",
            Icon::CaretDown => "caret-down",
            Icon::CaretUp => "caret-up",
            Icon::Image => "image",
            Icon::Table => "table",
            Icon::Link => "link",
            Icon::Printer => "printer",
            Icon::Highlighter => "highlighter",
            Icon::PaintBucket => "paint-bucket",
            Icon::Minimize => "minus",
            Icon::Maximize => "corners-out",
            Icon::Restore => "corners-in",
            Icon::Close => "x",
            Icon::Strikethrough => "text-strikethrough",
            Icon::Superscript => "text-superscript",
            Icon::Subscript => "text-subscript",
            Icon::Code => "code",
            Icon::EyeSlash => "eye-slash",
            Icon::Quotes => "quotes",
            Icon::TextColor => "text-a-underline",
        }
    }
}

/// A Phosphor icon. GPUI paints an svg only if the colour is set on the svg itself.
pub fn icon(icon: Icon, weight: Weight, size: Pixels, color: Rgba) -> Svg {
    let suffix = match weight {
        Weight::Regular => "",
        Weight::Bold => "-bold",
        Weight::Fill => "-fill",
    };
    let name = icon.file();
    svg()
        .path(format!("icons/phosphor/{name}{suffix}.svg"))
        .size(size)
        .flex_none()
        .text_color(color)
}
