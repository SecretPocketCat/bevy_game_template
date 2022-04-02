use crate::assets::Sprites;
use crate::input_binding::{get_menu_input_map, UiAction, UiInput};
use crate::palette::{Palette, PaletteColor};
use crate::pause::Inactive;
use crate::tween::{delay_tween, TweenDoneAction};
use crate::GameState;
use crate::{assets::Fonts, tween::UiColorLens};
use bevy::app::AppExit;
use bevy::ecs::system::Command;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy::utils::{HashMap, HashSet};
use bevy_tweening::lens::{
    TextColorLens, TransformPositionLens, TransformScaleLens, UiPositionLens,
};
use bevy_tweening::{
    component_animator_system, Animator, EaseFunction, Lens, Tracks, Tween, TweeningType,
};
use indexmap::IndexSet;
use std::time::Duration;

// todo:
// actual action handler

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ButtonInteractionStyles>()
            .add_event::<ButtonActiveEvt>()
            .add_startup_system(setup_ui)
            .add_system(component_animator_system::<UiColor>)
            .add_system_set(SystemSet::on_enter(GameState::Menu).with_system(setup_main_menu))
            .add_system_set(SystemSet::on_exit(GameState::Menu).with_system(despawn_panels))
            .add_system_set(
                SystemSet::on_update(GameState::Menu)
                    .with_system(handle_button_interaction)
                    .with_system(reactivate_button)
                    .with_system(on_btn_added)
                    .with_system(handle_ui_input)
                    .with_system(handle_button_action),
            )
            .add_system_to_stage(CoreStage::PostUpdate, reactivate_button);
    }
}

struct Ui {
    root_e: Entity,
    active_menu_e: Option<Entity>,
}

#[derive(Component, Clone, Copy)]
enum ButtonAction {
    ChangeState(GameState),
    ShowSubmenu {
        submenu: Submenu,
        parent_panel_e: Entity,
    },
    Cancel,
    Quit,
}

#[derive(Component, Clone, Copy)]
enum Submenu {
    Settings,
    Tutorial,
}

#[derive(Component)]
struct Cancelable {
    previous_panel_e: Entity,
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

pub struct ButtonActiveEvt {
    action: ButtonAction,
}

#[derive(Clone, Copy)]
struct SpawnBtnData<'a> {
    text: &'a str,
    is_accent: bool,
    is_focused: bool,
    action: ButtonAction,
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

fn setup_ui(mut commands: Commands) {
    commands.spawn_bundle(UiCameraBundle::default());

    let root_e = commands
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
        .id();

    commands.insert_resource(Ui {
        root_e,
        active_menu_e: None,
    });
}

fn setup_main_menu(
    mut commands: Commands,
    fonts: Res<Fonts>,
    btn_style: Res<ButtonInteractionStyles>,
    palette: Res<Palette>,
    sprites: Res<Sprites>,
    ui: Res<Ui>,
) {
    commands.entity(ui.root_e).with_children(|b| {
        let mut focusable_entities = IndexSet::new();

        b // menu root
            .spawn_bundle(get_wrapper_node_bundle())
            .insert(Animator::new(get_panel_tween(true)))
            .with_children(|b| {
                let main_panel_e = b.parent_entity();

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
                b.spawn_bundle(get_btns_wrapper_node()).with_children(|b| {
                    let base_btn_margin = 15.;

                    let mut btns = Vec::from([
                        (
                            "Play",
                            ButtonAction::ChangeState(GameState::Game),
                            base_btn_margin * 3.,
                            1.5,
                            true,
                        ),
                        (
                            "Tutorial",
                            ButtonAction::ShowSubmenu {
                                submenu: Submenu::Tutorial,
                                parent_panel_e: main_panel_e,
                            },
                            base_btn_margin,
                            1.,
                            false,
                        ),
                        (
                            "Settings",
                            ButtonAction::ShowSubmenu {
                                submenu: Submenu::Settings,
                                parent_panel_e: main_panel_e,
                            },
                            base_btn_margin * 2.5,
                            1.,
                            false,
                        ),
                    ]);

                    if cfg!(not(target_arch = "wasm32")) {
                        btns.push(("Quit", ButtonAction::Quit, 50., 1., false));
                    }

                    for (i, (text, action, margin_btm, size_mult, is_accent)) in
                        btns.iter().enumerate()
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
                                position: btn_style.normal.position,
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
                            focusable_entities.insert(spawn_btn(
                                SpawnBtnData {
                                    text: text,
                                    is_accent: *is_accent,
                                    is_focused: i == 0,
                                    action: *action,
                                },
                                b,
                                &fonts,
                                &palette,
                                &btn_style,
                            ));
                        });
                    }
                });
            })
            .insert(get_menu_input_map().unwrap())
            .insert(UiFocus {
                focusable_entities,
                current_focus_index: 0,
            });
    });
}

fn handle_button_interaction(
    mut commands: Commands,
    button_style: Res<ButtonInteractionStyles>,
    ui_focus_q: Query<(Entity, &UiFocus), Without<Inactive>>,
    mut interaction_q: Query<
        (
            Entity,
            &Interaction,
            &UiColor,
            &Transform,
            Option<&ButtonInteractionStyles>,
            &FocusState,
            &ButtonTextEntity,
            &ButtonAction,
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
    mut click_evw: EventWriter<ButtonActiveEvt>,
    palette: Res<Palette>,
) {
    for (focusable_e, focusable) in ui_focus_q.iter() {
        for focus_e in focusable.focusable_entities.iter() {
            if let Ok((
                button_e,
                interaction,
                ui_col,
                t,
                btn_style_override,
                focus_state,
                btn_txt_e,
                btn_action,
                parent,
            )) = interaction_q.get_mut(*focus_e)
            {
                if matches!(focus_state, FocusState::Active)
                    || matches!(interaction, Interaction::Clicked)
                {
                    commands.entity(focusable_e).insert(Inactive::Permanent);
                    commands.entity(button_e).insert(Inactive::Timed {
                        timer: Timer::from_seconds(0.4, false),
                    });
                    click_evw.send(ButtonActiveEvt {
                        action: *btn_action,
                    });
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
    mut commands: Commands,
    mut panel_q: Query<
        (Entity, &UiInput, &mut UiFocus, Option<&Cancelable>, &Parent),
        Without<Inactive>,
    >,
    mut focusable_q: Query<(&mut FocusState, &ButtonAction)>,
) {
    for (panel_e, input, mut focusable, cancelable, parent) in panel_q.iter_mut() {
        let any_focusables = focusable.focusable_entities.len() > 0;

        if any_focusables && input.just_pressed(UiAction::Confirm) {
            let active_e = focusable.focusable_entities[focusable.current_focus_index];
            if let Ok((mut focus_state, action)) = focusable_q.get_mut(active_e) {
                *focus_state = FocusState::Active;

                if matches!(action, ButtonAction::Cancel) {
                    if let Some(cancelable) = cancelable {
                        despawn_panel(parent.0, cancelable.previous_panel_e.into(), &mut commands);
                    }
                }
            }
        } else if cancelable.is_some() && input.just_pressed(UiAction::Cancel) {
            if let Some(cancelable) = cancelable {
                despawn_panel(parent.0, cancelable.previous_panel_e.into(), &mut commands);
            }
        } else if any_focusables {
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
                    if let Ok((mut focus_state, _)) = focusable_q.get_mut(*focus_state_e) {
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

fn handle_button_action(
    mut commands: Commands,
    ui: Res<Ui>,
    palette: Res<Palette>,
    fonts: Res<Fonts>,
    btn_style: Res<ButtonInteractionStyles>,
    mut btn_action_evr: EventReader<ButtonActiveEvt>,
    mut state: ResMut<State<GameState>>,
    mut exit: EventWriter<AppExit>,
) {
    for ev in btn_action_evr.iter() {
        match ev.action {
            ButtonAction::ChangeState(game_state) => {
                state.overwrite_set(game_state).unwrap();
            }
            ButtonAction::ShowSubmenu {
                submenu,
                parent_panel_e,
            } => match submenu {
                Submenu::Settings => spawn_settings(
                    ui.root_e,
                    parent_panel_e,
                    &mut commands,
                    &palette,
                    &fonts,
                    &btn_style,
                ),
                Submenu::Tutorial => spawn_tutorial(
                    ui.root_e,
                    parent_panel_e,
                    &mut commands,
                    &palette,
                    &fonts,
                    &btn_style,
                ),
            },
            ButtonAction::Cancel => {
                trace!("cancelling");
            }
            ButtonAction::Quit => exit.send(AppExit),
        }
    }
}

fn despawn_panels(mut commands: Commands, panel_q: Query<Entity, With<UiFocus>>) {
    for panel_e in panel_q.iter() {
        despawn_panel(panel_e, None, &mut commands);
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

fn spawn_settings(
    root_e: Entity,
    previous_panel_e: Entity,
    commands: &mut Commands,
    palette: &Palette,
    fonts: &Fonts,
    btn_style: &ButtonInteractionStyles,
) {
    // root
    let submenu_root_e = spawn_panel(
        root_e,
        Some(previous_panel_e),
        vec![SpawnBtnData {
            action: ButtonAction::Cancel,
            is_accent: false,
            is_focused: true,
            text: "Back",
        }],
        commands,
        palette,
        fonts,
        btn_style,
    );
}

fn spawn_tutorial(
    root_e: Entity,
    previous_panel_e: Entity,
    commands: &mut Commands,
    palette: &Palette,
    fonts: &Fonts,
    btn_style: &ButtonInteractionStyles,
) {
    // root
    let submenu_root_e = spawn_panel(
        root_e,
        Some(previous_panel_e),
        vec![SpawnBtnData {
            action: ButtonAction::Cancel,
            is_accent: false,
            is_focused: true,
            text: "Back",
        }],
        commands,
        palette,
        fonts,
        btn_style,
    );

    commands.entity(submenu_root_e).with_children(|b| {
        b.spawn_bundle(TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value:
"Tutorial thingy goes here.
Make it good...
Maybe just use an img or better yet, do it in game

Eiusmod consectetur sunt enim duis consequat ullamco pariatur magna. Culpa fugiat excepteur pariatur dolore tempor ad nisi aliqua est aute. Non anim ad esse sit aute ex laboris nulla nostrud sint labore sunt laboris ullamco. Ut cillum eu aute nulla incididunt voluptate.

Aute id incididunt consequat incididunt irure sint eu labore enim. Pariatur ipsum veniam adipisicing amet adipisicing sint consequat exercitation officia nostrud ullamco. Labore id exercitation do consequat cupidatat sit. Eu ipsum anim ullamco magna reprehenderit anim aliqua laborum nisi est qui magna eu dolore. Deserunt veniam cillum id culpa eiusmod magna esse voluptate ad reprehenderit.

Officia nisi non nostrud occaecat amet. Voluptate aute excepteur quis irure sit incididunt. Laboris ex ullamco non excepteur excepteur. Cillum pariatur dolore occaecat proident proident in fugiat excepteur nulla incididunt exercitation proident ex Lorem. Officia duis consectetur ea quis nulla. Enim culpa culpa cillum excepteur sint commodo.".to_string(),
                    style: TextStyle {
                        font: fonts.ui.clone(),
                        font_size: 25.,
                        color: palette.get_color(&btn_style.normal.text_color),
                    },
                }],
                alignment: Default::default(),
            },
            focus_policy: FocusPolicy::Pass,
            ..Default::default()
        });
    });
}

fn spawn_panel(
    root_e: Entity,
    previous_panel_e: Option<Entity>,
    btns: Vec<SpawnBtnData>,
    commands: &mut Commands,
    palette: &Palette,
    fonts: &Fonts,
    btn_style: &ButtonInteractionStyles,
) -> Entity {
    let mut submenu = None;

    commands.entity(root_e).with_children(|b| {
        // root
        b.spawn_bundle(get_wrapper_node_bundle())
            .insert(Animator::new(get_panel_tween(true)))
            .with_children(|b| {
                let margin = 50.;

                let mut panel = b.spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(85.), Val::Percent(100.)),
                        margin: Rect::all(Val::Px(margin)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::ColumnReverse,
                        ..Default::default()
                    },
                    color: palette.get_color(&PaletteColor::Background).into(),
                    ..Default::default()
                });

                let mut focusable_entities = IndexSet::new();

                panel
                    .with_children(|b| {
                        b.spawn_bundle(get_btns_wrapper_node()).with_children(|b| {
                            for btn in btns.iter() {
                                focusable_entities
                                    .insert(spawn_btn(*btn, b, fonts, palette, btn_style));
                            }
                        });
                    })
                    .insert(get_menu_input_map().unwrap())
                    .insert(UiFocus {
                        focusable_entities,
                        current_focus_index: 0,
                    });

                if let Some(previous_panel_e) = previous_panel_e {
                    panel.insert(Cancelable { previous_panel_e });
                }

                submenu = Some(panel.id());
            });
    });

    submenu.unwrap()
}

fn despawn_panel(panel_e: Entity, previous_panel_e: Option<Entity>, commands: &mut Commands) {
    if let Some(previous_panel_e) = previous_panel_e {
        commands.entity(previous_panel_e).remove::<Inactive>();
    }

    commands
        .entity(panel_e)
        .insert(Animator::new(get_panel_tween(false).with_completed_event(
            true,
            TweenDoneAction::DespawnRecursive.into(),
        )))
        .insert(Inactive::Permanent);
}

fn spawn_btn(
    btn_data: SpawnBtnData,
    child_builder: &mut ChildBuilder,
    fonts: &Fonts,
    palette: &Palette,
    btn_style: &ButtonInteractionStyles,
) -> Entity {
    let mut btn = child_builder.spawn_bundle(ButtonBundle {
        style: Style {
            size: Size::new(Val::Percent(100.), Val::Percent(100.)),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        color: palette.get_color(&btn_style.normal.color).into(),
        ..Default::default()
    });

    btn.insert(btn_data.action);

    if btn_data.is_accent {
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

    if btn_data.is_focused {
        btn.insert(FocusState::Focus);
    } else {
        btn.insert(FocusState::None);
    }

    let btn_e = btn.id();

    child_builder.spawn_bundle(TextBundle {
        text: Text {
            sections: vec![TextSection {
                value: btn_data.text.to_uppercase(),
                style: TextStyle {
                    font: fonts.ui.clone(),
                    font_size: 40. * if btn_data.is_accent { 1.5 } else { 1. },
                    color: palette.get_color(&btn_style.normal.text_color),
                },
            }],
            alignment: Default::default(),
        },
        focus_policy: FocusPolicy::Pass,
        ..Default::default()
    });

    btn_e
}

pub fn get_wrapper_node_bundle() -> NodeBundle {
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.), Val::Percent(100.)),
            margin: Rect::all(Val::Auto),
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Percent(100.),
                ..Default::default()
            },
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::ColumnReverse,
            ..Default::default()
        },
        color: Color::NONE.into(),
        ..Default::default()
    }
}

fn get_panel_tween(is_in: bool) -> Tween<Style> {
    Tween::new(
        if is_in {
            EaseFunction::CircularOut
        } else {
            EaseFunction::CircularIn
        },
        TweeningType::Once,
        Duration::from_millis(650),
        UiPositionLens {
            start: Rect {
                top: Val::Percent(if is_in { 100. } else { 0. }),
                ..Default::default()
            },
            end: Rect {
                top: Val::Percent(if is_in { 0. } else { 100. }),
                ..Default::default()
            },
        },
    )
}

fn get_btns_wrapper_node() -> NodeBundle {
    NodeBundle {
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
    }
}
