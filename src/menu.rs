use std::time::Duration;

use crate::palette::{Palette, PaletteColor};
use crate::pause::Inactive;
use crate::tween::delay_tween;
use crate::GameState;
use crate::{loading::FontAssets, tween::UiColorLens};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy_extensions::Vec2Conversion;
use bevy_tweening::lens::{TransformPositionLens, TransformScaleLens, UiPositionLens};
use bevy_tweening::{component_animator_system, Animator, EaseFunction, Lens, Tracks, Tween};

// todo:
// tabindex (Resource?)
// todo: img logo

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ButtonInteractionStyles>()
            .add_event::<ButtonClickEvt>()
            .add_system(component_animator_system::<UiColor>)
            .add_system_set(SystemSet::on_enter(GameState::Menu).with_system(setup_menu))
            .add_system(handle_button_interaction)
            .add_system_to_stage(CoreStage::PostUpdate, reactivate_button);
    }
}

#[derive(Component, Clone, Copy)]
enum ButtonAction {
    ChangeState(GameState),
}

#[derive(Clone, Copy)]
struct ButtonStyle {
    color: PaletteColor,
    text_color: PaletteColor,
    scale: Vec2,
    position: Rect<Val>,
    delay_ms: u64,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            scale: Vec2::ONE,
            color: PaletteColor::Button,
            text_color: PaletteColor::ButtonText,
            position: Rect::all(Val::Px(0.)),
            delay_ms: 0,
        }
    }
}

#[derive(Component, Clone, Copy)]
struct ButtonInteractionStyles {
    normal: ButtonStyle,
    focus: ButtonStyle,
    active: ButtonStyle,
}

impl Default for ButtonInteractionStyles {
    fn default() -> Self {
        let mut active_pos = Rect::all(Val::Px(0.));
        active_pos.top = Val::Px(10.);

        ButtonInteractionStyles {
            normal: ButtonStyle {
                ..Default::default()
            },
            focus: ButtonStyle {
                color: PaletteColor::ButtonFocus,
                text_color: PaletteColor::ButtonTextFocus,
                scale: Vec2::new(1.1, 0.9),
                delay_ms: 50,
                ..Default::default()
            },
            active: ButtonStyle {
                color: PaletteColor::ButtonActive,
                text_color: PaletteColor::ButtonTextActive,
                scale: Vec2::new(1.25, 0.75),
                position: active_pos,
                ..Default::default()
            },
        }
    }
}

impl ButtonInteractionStyles {
    pub fn get_interaction_style(&self, interaction: Interaction) -> &ButtonStyle {
        match interaction {
            Interaction::Clicked => &self.active,
            Interaction::Hovered => &self.focus,
            Interaction::None => &self.normal,
        }
    }

    pub fn get_override_or_self<'a>(&'a self, style_override: Option<&'a Self>) -> &Self {
        style_override.unwrap_or(self)
    }
}

pub struct ButtonClickEvt {
    button_e: Entity,
}

fn setup_menu(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    button_colors: Res<ButtonInteractionStyles>,
    palette: Res<Palette>,
) {
    commands.spawn_bundle(UiCameraBundle::default());

    commands
        // menu root
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .with_children(|b| {
            // btns root
            b.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                    margin: Rect::all(Val::Auto),
                    flex_direction: FlexDirection::ColumnReverse,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                color: Color::NONE.into(),
                ..Default::default()
            })
            .with_children(|b| {
                // btns
                // todo: different sizes & colors just to make play more prominent?
                // pick an accent col?
                let base_btn_margin = 15.;

                for (text, action, margin_btm, size_mult, is_accent) in [
                    (
                        "Play",
                        ButtonAction::ChangeState(GameState::Game),
                        base_btn_margin * 3.,
                        1.5,
                        true,
                    ),
                    (
                        "Tutorial",
                        ButtonAction::ChangeState(GameState::Tutorial),
                        base_btn_margin,
                        1.,
                        false,
                    ),
                    (
                        "Settings",
                        ButtonAction::ChangeState(GameState::Settings),
                        base_btn_margin * 2.,
                        1.,
                        false,
                    ),
                    (
                        "Quit",
                        ButtonAction::ChangeState(GameState::Quit),
                        0.,
                        1.,
                        false,
                    ),
                ]
                .iter()
                {
                    b.spawn_bundle(NodeBundle {
                        style: Style {
                            size: Size::new(
                                Val::Percent(25. * *size_mult),
                                Val::Percent(10. * *size_mult),
                            ),
                            min_size: Size::new(
                                Val::Px(150.0 * *size_mult),
                                Val::Px(65.0 * *size_mult),
                            ),
                            position: button_colors.normal.position,
                            margin: Rect {
                                bottom: Val::Px(*margin_btm),
                                ..Default::default()
                            },
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..Default::default()
                        },
                        color: Color::NONE.into(),
                        ..Default::default()
                    })
                    .with_children(|b| {
                        let mut btn = b.spawn_bundle(ButtonBundle {
                            style: Style {
                                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                                position_type: PositionType::Absolute,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..Default::default()
                            },
                            color: palette.get_color(&button_colors.normal.color).into(),
                            ..Default::default()
                        });

                        btn.insert(*action);

                        if *is_accent {
                            btn.insert(ButtonInteractionStyles {
                                normal: ButtonStyle {
                                    color: PaletteColor::ButtonAccent,
                                    text_color: PaletteColor::ButtonTextAccent,
                                    ..Default::default()
                                },
                                focus: ButtonStyle {
                                    color: PaletteColor::ButtonAccentFocus,
                                    text_color: PaletteColor::ButtonTextAccentFocus,
                                    scale: Vec2::new(1.1, 0.9),
                                    delay_ms: 50,
                                    ..Default::default()
                                },
                                active: ButtonStyle {
                                    color: PaletteColor::ButtonAccentActive,
                                    text_color: PaletteColor::ButtonTextAccentActive,
                                    scale: Vec2::new(1.25, 0.75),
                                    position: Rect {
                                        top: Val::Px(20.),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                            });
                        }

                        b.spawn_bundle(TextBundle {
                            text: Text {
                                sections: vec![TextSection {
                                    value: text.to_string(),
                                    style: TextStyle {
                                        font: font_assets.fira_sans.clone(),
                                        font_size: 40.0 * *size_mult,
                                        color: palette.get_color(&button_colors.normal.text_color),
                                    },
                                }],
                                alignment: Default::default(),
                            },
                            focus_policy: FocusPolicy::Pass,
                            ..Default::default()
                        });
                    });
                }
            });
        });
}

fn handle_button_interaction(
    mut commands: Commands,
    button_style: Res<ButtonInteractionStyles>,
    mut interaction_q: Query<
        (
            Entity,
            &Interaction,
            &UiColor,
            &Transform,
            Option<&ButtonInteractionStyles>,
            &Parent,
        ),
        (Changed<Interaction>, Without<Inactive>),
    >,
    style_q: Query<&Style>,
    mut click_evw: EventWriter<ButtonClickEvt>,
    palette: Res<Palette>,
) {
    for (button_e, interaction, ui_col, t, btn_style_override, parent) in interaction_q.iter_mut() {
        if let Interaction::Clicked = interaction {
            commands.entity(button_e).insert(Inactive::Timed {
                timer: Timer::from_seconds(0.4, false),
            });
            click_evw.send(ButtonClickEvt { button_e });
        }

        if let Ok(style) = style_q.get(parent.0) {
            tween_button(
                &mut commands,
                button_e,
                parent.0,
                ui_col.0,
                t.scale,
                style.position,
                button_style
                    .get_override_or_self(btn_style_override)
                    .get_interaction_style(*interaction),
                &palette,
            );
        }
    }
}

fn reactivate_button(
    mut commands: Commands,
    button_style: Res<ButtonInteractionStyles>,
    removed: RemovedComponents<Inactive>,
    interaction_q: Query<
        (
            &Interaction,
            &UiColor,
            &Transform,
            Option<&ButtonInteractionStyles>,
            &Parent,
        ),
        With<Button>,
    >,
    style_q: Query<&Style>,
    palette: Res<Palette>,
) {
    for inactive_e in removed.iter() {
        if let Ok((interaction, ui_col, t, btn_style_override, parent)) =
            interaction_q.get(inactive_e)
        {
            if let Ok(style) = style_q.get(parent.0) {
                tween_button(
                    &mut commands,
                    inactive_e,
                    parent.0,
                    ui_col.0,
                    t.scale,
                    style.position,
                    button_style
                        .get_override_or_self(btn_style_override)
                        .get_interaction_style(*interaction),
                    &palette,
                );
            }
        }
    }
}

// todo: tween text
fn tween_button(
    commands: &mut Commands,
    button_e: Entity,
    button_root_e: Entity,
    start_color: Color,
    start_scale: Vec3,
    start_ui_pos: Rect<Val>,
    style: &ButtonStyle,
    palette: &Palette,
) {
    commands
        .entity(button_e)
        .insert(Animator::new(delay_tween(
            Tween::new(
                EaseFunction::QuadraticInOut,
                bevy_tweening::TweeningType::Once,
                Duration::from_millis(350),
                UiColorLens {
                    start: start_color,
                    end: palette.get_color(&style.color),
                },
            ),
            style.delay_ms,
        )))
        .insert(Animator::new(delay_tween(
            Tween::new(
                EaseFunction::BackOut,
                bevy_tweening::TweeningType::Once,
                Duration::from_millis(350),
                TransformScaleLens {
                    start: start_scale,
                    end: style.scale.extend(start_scale.z),
                },
            ),
            style.delay_ms,
        )));

    commands
        .entity(button_root_e)
        .insert(Animator::new(delay_tween(
            Tween::new(
                EaseFunction::BackOut,
                bevy_tweening::TweeningType::Once,
                Duration::from_millis(350),
                UiPositionLens {
                    start: start_ui_pos,
                    end: style.position,
                },
            ),
            style.delay_ms,
        )));
}
