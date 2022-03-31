// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(derive_default_enum)]
#![feature(if_let_guard)]
#![feature(drain_filter)]
#![allow(clippy::type_complexity, clippy::too_many_arguments)]

use bevy::{prelude::*, window::WindowResizeConstraints};
use bevy_kira_audio::AudioPlugin;
use bevy_prototype_lyon::plugin::ShapePlugin;
use bevy_time::TimePlugin;
use bevy_tweening::TweeningPlugin;
use debug::DebugPlugin;
use game_state::GameState;
use heron::PhysicsPlugin;
use loading::LoadingPlugin;
use menu::MenuPlugin;
use pause::PausePlugin;
use render::GameDimensions;
use sfx::SfxPlugin;

mod debug;
mod game_state;
mod loading;
mod menu;
mod pause;
mod render;
mod sfx;
mod tween;

// todo:
const NAME: &str = "Jam Game";

fn main() {
    let base_size = Vec2::new(1280., 720.);
    let dimensions = GameDimensions {
        base_size,
        min_size: base_size * 0.5,
    };

    let mut app = App::new();
    app.insert_resource(Msaa { samples: 4 })
        // resources needed before default plugins to take effect
        .insert_resource(WindowDescriptor {
            title: NAME.to_string(),
            width: dimensions.base_size.x,
            height: dimensions.base_size.y,
            resize_constraints: WindowResizeConstraints {
                min_width: dimensions.min_size.x,
                min_height: dimensions.min_size.y,
                ..Default::default()
            },
            position: Some(Vec2::splat(50.)),
            // mode: WindowMode::BorderlessFullscreen,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::GRAY));

    // game resources
    app.insert_resource(dimensions);

    // bevy plugins
    app.add_plugins(DefaultPlugins);

    if cfg!(not(feature = "gizmos")) {
        // heron 2d-debug adds lyon plugin as well, which would cause a panic
        app.add_plugin(ShapePlugin);
    }

    if cfg!(feature = "debug") {
        app.add_plugin(DebugPlugin);
    }

    // 3rd party plugins
    app.add_plugin(AudioPlugin)
        .add_plugin(PhysicsPlugin::default())
        .add_plugin(TweeningPlugin);

    // initial state
    app.add_state(GameState::Loading);

    // own external plugins
    app.add_plugin(TimePlugin);

    // game plugins
    app.add_plugin(LoadingPlugin)
        .add_plugin(MenuPlugin)
        .add_plugin(PausePlugin)
        .add_plugin(SfxPlugin);

    app.run();
}
