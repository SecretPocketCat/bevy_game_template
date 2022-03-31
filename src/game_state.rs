use bevy::prelude::SystemLabel;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum GameState {
    Loading,
    Menu,
    Settings,
    Tutorial,
    Game,
    Reset,
    Quit,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, SystemLabel)]
pub enum UpdatePhase {
    Input,
    Physics,
    Movement,
    Animation,
    Audio,
    Render,
}
