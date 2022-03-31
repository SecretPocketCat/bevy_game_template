use std::time::Duration;

use crate::pause::Inactive;
use crate::GameState;
use crate::{loading::FontAssets, tween::UiColorLens};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy_extensions::Vec2Conversion;
use bevy_tweening::lens::{TransformPositionLens, TransformScaleLens, UiPositionLens};
use bevy_tweening::{component_animator_system, Animator, EaseFunction, Lens, Tracks, Tween};

// todo:
// use palette
// different btn colors
// tabindex (Resource?)
// todo: img logo

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ButtonStyle>()
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

struct ButtonInteractionStyle {
    color: Color,
    text_color: Color,
    scale: Vec2,
    position: Rect<Val>,
}

impl Default for ButtonInteractionStyle {
    fn default() -> Self {
        Self {
            scale: Vec2::ONE,
            color: Color::BLACK,
            text_color: Color::WHITE,
            position: Rect::all(Val::Px(0.)),
        }
    }
}

struct ButtonStyle {
    normal: ButtonInteractionStyle,
    hover: ButtonInteractionStyle,
    active: ButtonInteractionStyle,
    // todo
    // focus: UiColor,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        let mut active_pos = Rect::all(Val::Px(0.));
        active_pos.top = Val::Px(10.);

        ButtonStyle {
            normal: ButtonInteractionStyle {
                color: Color::rgb(0.15, 0.15, 0.15),
                ..Default::default()
            },
            hover: ButtonInteractionStyle {
                color: Color::rgb(0.25, 0.25, 0.25),
                scale: Vec2::new(1.1, 0.9),
                ..Default::default()
            },
            active: ButtonInteractionStyle {
                color: Color::rgb(0.65, 0.65, 0.65),
                scale: Vec2::new(1.25, 0.75),
                position: active_pos,
                ..Default::default()
            },
        }
    }
}

impl ButtonStyle {
    pub fn get_interaction_style(&self, interaction: Interaction) -> &ButtonInteractionStyle {
        match interaction {
            Interaction::Clicked => &self.active,
            Interaction::Hovered => &self.hover,
            Interaction::None => &self.normal,
        }
    }
}

pub struct ButtonClickEvt {
    button_e: Entity,
}

fn setup_menu(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    button_colors: Res<ButtonStyle>,
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

                for (text, action, margin_btm, size_mult) in [
                    (
                        "Play",
                        ButtonAction::ChangeState(GameState::Game),
                        base_btn_margin * 3.,
                        1.5,
                    ),
                    (
                        "Tutorial",
                        ButtonAction::ChangeState(GameState::Tutorial),
                        base_btn_margin,
                        1.,
                    ),
                    (
                        "Settings",
                        ButtonAction::ChangeState(GameState::Settings),
                        base_btn_margin * 2.,
                        1.,
                    ),
                    ("Quit", ButtonAction::ChangeState(GameState::Quit), 0., 1.),
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
                        b.spawn_bundle(ButtonBundle {
                            style: Style {
                                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                                position_type: PositionType::Absolute,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..Default::default()
                            },
                            color: button_colors.normal.color.into(),
                            ..Default::default()
                        })
                        .insert(*action);

                        b.spawn_bundle(TextBundle {
                            text: Text {
                                sections: vec![TextSection {
                                    value: text.to_string(),
                                    style: TextStyle {
                                        font: font_assets.fira_sans.clone(),
                                        font_size: 40.0 * *size_mult,
                                        color: Color::rgb(0.9, 0.9, 0.9),
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
    button_style: Res<ButtonStyle>,
    mut interaction_q: Query<
        (Entity, &Interaction, &UiColor, &Transform, &Parent),
        (Changed<Interaction>, Without<Inactive>),
    >,
    style_q: Query<&Style>,
    mut click_evw: EventWriter<ButtonClickEvt>,
) {
    for (button_e, interaction, ui_col, t, parent) in interaction_q.iter_mut() {
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
                button_style.get_interaction_style(*interaction),
            );
        }
    }
}

fn reactivate_button(
    mut commands: Commands,
    button_colors: Res<ButtonStyle>,
    removed: RemovedComponents<Inactive>,
    interaction_q: Query<(&Interaction, &UiColor, &Transform, &Parent), With<Button>>,
    style_q: Query<&Style>,
) {
    for inactive_e in removed.iter() {
        if let Ok((interaction, ui_col, t, parent)) = interaction_q.get(inactive_e) {
            if let Ok(style) = style_q.get(parent.0) {
                tween_button(
                    &mut commands,
                    inactive_e,
                    parent.0,
                    ui_col.0,
                    t.scale,
                    style.position,
                    button_colors.get_interaction_style(*interaction),
                );
            }
        }
    }
}

fn tween_button(
    commands: &mut Commands,
    button_e: Entity,
    button_root_e: Entity,
    start_color: Color,
    start_scale: Vec3,
    start_ui_pos: Rect<Val>,
    style: &ButtonInteractionStyle,
) {
    commands
        .entity(button_e)
        .insert(Animator::new(Tween::new(
            EaseFunction::QuadraticInOut,
            bevy_tweening::TweeningType::Once,
            Duration::from_millis(350),
            UiColorLens {
                start: start_color,
                end: style.color,
            },
        )))
        .insert(Animator::new(Tween::new(
            EaseFunction::BackOut,
            bevy_tweening::TweeningType::Once,
            Duration::from_millis(350),
            TransformScaleLens {
                start: start_scale,
                end: style.scale.extend(start_scale.z),
            },
        )));

    commands
        .entity(button_root_e)
        .insert(Animator::new(Tween::new(
            EaseFunction::BackOut,
            bevy_tweening::TweeningType::Once,
            Duration::from_millis(350),
            UiPositionLens {
                start: start_ui_pos,
                end: style.position,
            },
        )));
}
