use bevy::{ecs::schedule::StateData, prelude::*};
use bevy_time::*;

pub struct GameStatePlugin;
impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DelayedState<GameState>>()
            .add_system(set_delayed_state::<GameState>);
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum GameState {
    Loading,
    Menu,
    Game,
    Reset,
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

pub struct DelayedState<T: StateData> {
    timer: Timer,
    queued_state: Option<T>,
}

impl<T: StateData> Default for DelayedState<T> {
    fn default() -> Self {
        Self {
            timer: Default::default(),
            queued_state: Default::default(),
        }
    }
}

impl<T: StateData + Copy> DelayedState<T> {
    pub fn queue_state(&mut self, state: T, delay_sec: f32) {
        if let Some(queued) = self.queued_state {
            if queued == state {
                return;
            }
        }

        self.queued_state = Some(state);
        self.timer = Timer::from_seconds(delay_sec, false);
    }
}

pub fn set_delayed_state<T: StateData + Copy>(
    mut delayed_state: ResMut<DelayedState<T>>,
    mut state: ResMut<State<T>>,
    time: ScaledTime,
) {
    if let Some(next_state) = delayed_state.queued_state {
        delayed_state.timer.tick(time.scaled_delta());

        if delayed_state.timer.finished() {
            info!("setting queued state");
            state.overwrite_set(next_state).unwrap();
            delayed_state.queued_state = None;
        }
    }
}
