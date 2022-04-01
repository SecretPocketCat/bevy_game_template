use bevy::prelude::*;
use bevy_input::*;

pub struct InputBindingPlugin;
impl Plugin for InputBindingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_action_input_systems::<UiAction>()
            .add_action_input_systems_with_axis::<PlayerAction, PlayerAxis>();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiAction {
    Confirm,
    Cancel,
    Down,
    Up,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerAction {
    Reset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerAxis {
    MoveX,
    MoveY,
}

pub type PlayerInput = ActionInput<PlayerAction, PlayerAxis>;
pub type UiInput = ActionInput<UiAction>;

pub fn get_menu_input_map() -> Result<ActionMap<UiAction>, BindingError> {
    let mut map = ActionMap::<UiAction>::new();
    map.bind_button_action(UiAction::Confirm, GamepadButtonType::South)?
        .bind_button_action(UiAction::Confirm, KeyCode::Return)?
        .bind_button_action(UiAction::Confirm, KeyCode::Space)?
        .bind_button_action(UiAction::Cancel, GamepadButtonType::West)?
        .bind_button_action(UiAction::Cancel, KeyCode::Escape)?
        .bind_button_action(UiAction::Up, KeyCode::W)?
        .bind_button_action(UiAction::Up, KeyCode::Up)?
        .bind_button_action(UiAction::Down, KeyCode::S)?
        .bind_button_action(UiAction::Down, KeyCode::Down)?
        // todo: bind stick as well?
        .bind_button_action(UiAction::Up, GamepadButtonType::DPadUp)?
        .bind_button_action(UiAction::Down, GamepadButtonType::DPadDown)?;

    Ok(map)
}
