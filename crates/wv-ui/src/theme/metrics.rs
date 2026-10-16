use gpui_kit::{Pixels, px};

#[allow(dead_code)]
pub mod space {
    use super::*;
    pub fn xs() -> Pixels {
        px(4.)
    }
    pub fn sm() -> Pixels {
        px(8.)
    }
    pub fn md() -> Pixels {
        px(12.)
    }
    pub fn lg() -> Pixels {
        px(16.)
    }
    pub fn xl() -> Pixels {
        px(24.)
    }
}

#[allow(dead_code)]
pub mod radius {
    use super::*;
    pub fn sm() -> Pixels {
        px(8.)
    }
    pub fn md() -> Pixels {
        px(12.)
    }
    pub fn lg() -> Pixels {
        px(16.)
    }
}
