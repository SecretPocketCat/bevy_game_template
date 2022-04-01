use crate::assets::Sprites;
use crate::input_binding::{get_menu_input_map, UiAction, UiInput};
use crate::palette::{Palette, PaletteColor};
use crate::pause::Inactive;
use crate::tween::delay_tween;
use crate::GameState;
use crate::{assets::Fonts, tween::UiColorLens};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy::utils::{HashMap, HashSet};
use bevy_tweening::lens::{
    TextColorLens, TransformPositionLens, TransformScaleLens, UiPositionLens,
};
use bevy_tweening::{component_animator_system, Animator, EaseFunction, Lens, Tracks, Tween};
use indexmap::IndexSet;
use std::time::Duration;

// todo:
// actual action handler

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ButtonInteractionStyles>()
            .add_event::<ButtonClickEvt>()
            .add_system(component_animator_system::<UiColor>)
            .add_system_set(SystemSet::on_enter(GameState::Menu).with_system(setup_menu))
            .add_system(handle_button_interaction)
            .add_system_to_stage(CoreStage::PostUpdate, reactivate_button)
            .add_system(on_btn_added)
            .add_system(handle_ui_input);
    }
}

#[derive(Component, Clone, Copy)]
enum ButtonAction {
    ChangeState(GameState),
}

#[derive(Component)]
struct ButtonTextEntity(Entity);

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
    pub fn get_interaction_style(
        &self,
        interaction: Interaction,
        focus_state: &FocusState,
    ) -> &ButtonStyle {
        match focus_state {
            FocusState::None => match interaction {
                Interaction::Clicked => &self.active,
                Interaction::Hovered => &self.focus,
                Interaction::None => &self.normal,
            },
            FocusState::Focus => match interaction {
                Interaction::Clicked => &self.active,
                _ => &self.focus,
            },
            FocusState::Active => &self.active,
        }
    }

    pub fn get_override_or_self<'a>(&'a self, style_override: Option<&'a Self>) -> &Self {
        style_override.unwrap_or(self)
    }
}

pub struct ButtonClickEvt {
    button_e: Entity,
}

#[derive(Component)]
struct UiFocus {
    focusable_entities: IndexSet<Entity>,
    current_focus_index: usize,
}

#[derive(Component, Clone, Copy)]
enum FocusState {
    None,
    Focus,
    Active,
}

fn setup_menu(
    mut commands: Commands,
    font_assets: Res<Fonts>,
    button_colors: Res<ButtonInteractionStyles>,
    palette: Res<Palette>,
    sprites: Res<Sprites>,
) {
    commands.spawn_bundle(UiCameraBundle::default());

    let mut focusable_entities = IndexSet::new();

    commands
        // menu root
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .with_children(|b| {
            b.spawn_bundle(ImageBundle {
                image: sprites.bevy_logo.clone().into(),
                style: Style {
                    max_size: Size::new(Val::Percent(100.), Val::Percent(30.)),
                    margin: Rect::all(Val::Auto),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                ..Default::default()
            });

            // btns root
            b.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.), Val::Auto),
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
                let base_btn_margin = 15.;

                for (i, (text, action, margin_btm, size_mult, is_accent)) in [
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
                        50.,
                        1.,
                        false,
                    ),
                ]
                .iter()
                .enumerate()
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

                        if i == 0 {
                            btn.insert(FocusState::Focus);
                        } else {
                            btn.insert(FocusState::None);
                        }

                        focusable_entities.insert(btn.id());

                        b.spawn_bundle(TextBundle {
                            text: Text {
                                sections: vec![TextSection {
                                    value: text.to_string().to_uppercase(),
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
        })
        .insert(get_menu_input_map().unwrap())
        .insert(UiFocus {
            focusable_entities,
            current_focus_index: 0,
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
            &FocusState,
            &ButtonTextEntity,
            &Parent,
        ),
        (
            Or<(
                Changed<Interaction>,
                Changed<FocusState>,
                Added<ButtonTextEntity>,
            )>,
            Without<Inactive>,
        ),
    >,
    style_q: Query<&Style>,
    text_q: Query<&Text>,
    mut click_evw: EventWriter<ButtonClickEvt>,
    palette: Res<Palette>,
) {
    for (button_e, interaction, ui_col, t, btn_style_override, focus_state, btn_txt_e, parent) in
        interaction_q.iter_mut()
    {
        if matches!(focus_state, FocusState::Active) || matches!(interaction, Interaction::Clicked)
        {
            commands.entity(button_e).insert(Inactive::Timed {
                timer: Timer::from_seconds(0.4, false),
            });
            click_evw.send(ButtonClickEvt { button_e });
        }

        if let Ok(style) = style_q.get(parent.0) {
            if let Ok(text) = text_q.get(btn_txt_e.0) {
                tween_button(
                    &mut commands,
                    button_e,
                    btn_txt_e.0,
                    parent.0,
                    ui_col.0,
                    text.sections[0].style.color,
                    t.scale,
                    style.position,
                    button_style
                        .get_override_or_self(btn_style_override)
                        .get_interaction_style(*interaction, focus_state),
                    &palette,
                );
            }
        }
    }
}

fn reactivate_button(
    mut commands: Commands,
    button_style: Res<ButtonInteractionStyles>,
    removed: RemovedComponents<Inactive>,
    ui_focus_q: Query<&UiFocus>,
    mut interaction_q: Query<
        (
            &Interaction,
            &UiColor,
            &Transform,
            Option<&ButtonInteractionStyles>,
            &mut FocusState,
            &ButtonTextEntity,
            &Parent,
        ),
        With<Button>,
    >,
    style_q: Query<&Style>,
    text_q: Query<&Text>,
    palette: Res<Palette>,
) {
    for inactive_e in removed.iter() {
        for ui_focus in ui_focus_q.iter() {
            if ui_focus.focusable_entities.contains(&inactive_e) {
                if let Ok((
                    interaction,
                    ui_col,
                    t,
                    btn_style_override,
                    mut focus_state,
                    btn_txt_e,
                    parent,
                )) = interaction_q.get_mut(inactive_e)
                {
                    *focus_state = if ui_focus.current_focus_index
                        == ui_focus
                            .focusable_entities
                            .get_index_of(&inactive_e)
                            .unwrap()
                    {
                        FocusState::Focus
                    } else {
                        FocusState::None
                    };

                    if let Ok(style) = style_q.get(parent.0) {
                        if let Ok(text) = text_q.get(btn_txt_e.0) {
                            tween_button(
                                &mut commands,
                                inactive_e,
                                btn_txt_e.0,
                                parent.0,
                                ui_col.0,
                                text.sections[0].style.color,
                                t.scale,
                                style.position,
                                button_style
                                    .get_override_or_self(btn_style_override)
                                    .get_interaction_style(*interaction, &focus_state),
                                &palette,
                            );
                        }
                    }
                }
            }
        }
    }
}

fn handle_ui_input(
    mut input_q: Query<(&UiInput, &mut UiFocus)>,
    mut focusable_q: Query<&mut FocusState>,
) {
    for (input, mut focusable) in input_q.iter_mut() {
        if input.just_pressed(UiAction::Confirm) {
            let active_e = focusable.focusable_entities[focusable.current_focus_index];
            if let Ok(mut focus_state) = focusable_q.get_mut(active_e) {
                *focus_state = FocusState::Active;
            }
        } else {
            let mut focus_offset: i8 = 0;
            if input.just_pressed(UiAction::Down) {
                focus_offset = 1;
            }
            if input.just_pressed(UiAction::Up) {
                focus_offset = -1;
            }

            if focus_offset != 0 {
                focusable.current_focus_index = (focusable.current_focus_index as i8 + focus_offset)
                    .rem_euclid(focusable.focusable_entities.len() as i8)
                    as usize;

                for (i, focus_state_e) in focusable.focusable_entities.iter().enumerate() {
                    if let Ok(mut focus_state) = focusable_q.get_mut(*focus_state_e) {
                        *focus_state = if i == focusable.current_focus_index {
                            FocusState::Focus
                        } else {
                            FocusState::None
                        };
                    }
                }
            }
        }
    }
}

fn on_btn_added(
    mut commands: Commands,
    btn_q: Query<(Entity, &Parent), (Added<FocusState>, With<Button>)>,
    children_q: Query<&Children>,
    text_q: Query<(), With<Text>>,
) {
    for (btn_e, btn_p) in btn_q.iter() {
        if let Ok(children) = children_q.get(btn_p.0) {
            for child in children.iter() {
                if let Ok(_) = text_q.get(*child) {
                    commands.entity(btn_e).insert(ButtonTextEntity(*child));
                }
            }
        }
    }
}

fn tween_button(
    commands: &mut Commands,
    button_e: Entity,
    button_text_e: Entity,
    button_root_e: Entity,
    start_color: Color,
    start_text_color: Color,
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
        .entity(button_text_e)
        .insert(Animator::new(delay_tween(
            Tween::new(
                EaseFunction::BackOut,
                bevy_tweening::TweeningType::Once,
                Duration::from_millis(200),
                TextColorLens {
                    start: start_text_color,
                    end: palette.get_color(&style.text_color),
                    section: 0,
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
