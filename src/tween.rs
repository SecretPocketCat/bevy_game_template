use bevy::prelude::*;
use bevy_tweening::lens::{SpriteColorLens, TransformScaleLens};
use bevy_tweening::*;
use std::time::Duration;

pub struct TweenPlugin;
impl Plugin for TweenPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(on_tween_completed);
    }
}

#[repr(u64)]
pub enum TweenDoneAction {
    None = 0,
    DespawnRecursive = 1,
}

impl From<u64> for TweenDoneAction {
    fn from(val: u64) -> Self {
        unsafe { ::std::mem::transmute(val) }
    }
}

impl From<TweenDoneAction> for u64 {
    fn from(val: TweenDoneAction) -> Self {
        val as u64
    }
}

fn on_tween_completed(
    mut commands: Commands,
    mut ev_reader: EventReader<TweenCompleted>,
    entity_q: Query<Entity>,
) {
    for ev in ev_reader.iter() {
        match TweenDoneAction::from(ev.user_data) {
            TweenDoneAction::None => {}
            TweenDoneAction::DespawnRecursive => {
                if entity_q.get(ev.entity).is_ok() {
                    commands.entity(ev.entity).despawn_recursive();
                }
            }
        }
    }
}

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
    pub start: Rect<Val>,
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

pub fn get_scale_out_tween(
    start_scale: Vec3,
    duration_ms: u64,
    on_completed: Option<TweenDoneAction>,
) -> Tween<Transform> {
    get_scale_tween(
        start_scale,
        Vec3::ZERO,
        EaseFunction::QuadraticIn,
        duration_ms,
        on_completed,
    )
}

pub fn get_scale_in_tween(
    end_scale: Vec3,
    duration_ms: u64,
    on_completed: Option<TweenDoneAction>,
) -> Tween<Transform> {
    get_scale_tween(
        Vec3::ZERO,
        end_scale,
        EaseFunction::BackOut,
        duration_ms,
        on_completed,
    )
}

pub fn get_scale_tween(
    start_scale: Vec3,
    end_scale: Vec3,
    ease: EaseFunction,
    duration_ms: u64,
    on_completed: Option<TweenDoneAction>,
) -> Tween<Transform> {
    let mut tween = Tween::new(
        ease,
        TweeningType::Once,
        Duration::from_millis(duration_ms),
        TransformScaleLens {
            start: start_scale,
            end: end_scale,
        },
    );

    if let Some(on_completed) = on_completed {
        tween = tween.with_completed_event(true, on_completed.into());
    }

    tween
}

pub fn get_scale_out_anim(
    start_scale: Vec3,
    duration_ms: u64,
    on_completed: Option<TweenDoneAction>,
) -> Animator<Transform> {
    get_scale_anim(
        start_scale,
        Vec3::ZERO,
        EaseFunction::QuadraticIn,
        duration_ms,
        on_completed,
    )
}

pub fn get_scale_in_anim(
    end_scale: Vec3,
    duration_ms: u64,
    on_completed: Option<TweenDoneAction>,
) -> Animator<Transform> {
    get_scale_anim(
        Vec3::ZERO,
        end_scale,
        EaseFunction::BackOut,
        duration_ms,
        on_completed,
    )
}

pub fn get_scale_anim(
    start_scale: Vec3,
    end_scale: Vec3,
    ease: EaseFunction,
    duration_ms: u64,
    on_completed: Option<TweenDoneAction>,
) -> Animator<Transform> {
    Animator::new(get_scale_tween(
        start_scale,
        end_scale,
        ease,
        duration_ms,
        on_completed,
    ))
}

pub fn get_fade_out_sprite_anim(
    start_col: Color,
    duration_ms: u64,
    on_completed: Option<TweenDoneAction>,
) -> Animator<Sprite> {
    Animator::new(get_fade_out_sprite_tween(
        start_col,
        duration_ms,
        on_completed,
    ))
}

pub fn get_fade_out_sprite_tween(
    start_col: Color,
    duration_ms: u64,
    on_completed: Option<TweenDoneAction>,
) -> Tween<Sprite> {
    let mut tween = Tween::new(
        EaseFunction::QuadraticInOut,
        TweeningType::Once,
        Duration::from_millis(duration_ms),
        SpriteColorLens {
            start: start_col,
            end: Color::NONE,
        },
    );

    if let Some(on_completed) = on_completed {
        tween = tween.with_completed_event(true, on_completed.into());
    }

    tween
}
