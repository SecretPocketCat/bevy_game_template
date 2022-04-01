use std::time::Duration;

use bevy::prelude::*;
use bevy_tweening::{Delay, Lens, Sequence, Tween};

pub struct UiColorLens {
    pub start: Color,
    pub end: Color,
}

// todo: impl macro
impl Lens<UiColor> for UiColorLens {
    fn lerp(&mut self, target: &mut UiColor, ratio: f32) {
        target.0 = lerp_color(self.start, self.end, ratio);
    }
}

pub fn lerp_color(from: Color, to: Color, ratio: f32) -> Color {
    let start: Vec4 = from.into();
    let end: Vec4 = to.into();
    start.lerp(end, ratio).into()
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct UiMarginLens {
    /// Start position.
    pub start: Rect<Val>,
    /// End position.
    pub end: Rect<Val>,
}

fn lerp_val(start: &Val, end: &Val, ratio: f32) -> Val {
    match (start, end) {
        (Val::Percent(start), Val::Percent(end)) => Val::Percent(start + (end - start) * ratio),
        (Val::Px(start), Val::Px(end)) => Val::Px(start + (end - start) * ratio),
        _ => *start,
    }
}

impl Lens<Style> for UiMarginLens {
    fn lerp(&mut self, target: &mut Style, ratio: f32) {
        target.margin = Rect {
            left: lerp_val(&self.start.left, &self.end.left, ratio),
            right: lerp_val(&self.start.right, &self.end.right, ratio),
            top: lerp_val(&self.start.top, &self.end.top, ratio),
            bottom: lerp_val(&self.start.bottom, &self.end.bottom, ratio),
        };
    }
}

pub fn delay_tween<T: 'static>(tween: Tween<T>, delay_ms: u64) -> Sequence<T> {
    if delay_ms > 0 {
        Delay::new(Duration::from_millis(delay_ms)).then(tween)
    } else {
        Sequence::new([tween])
    }
}
