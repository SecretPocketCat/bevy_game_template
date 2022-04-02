use crate::{
    game_state::DelayedState,
    tween::{get_fade_out_sprite_anim, get_fade_out_sprite_tween, TweenDoneAction, UiColorLens},
    GameState,
};
use bevy::{
    asset::{Asset, LoadState},
    prelude::*,
};
use bevy_kira_audio::AudioSource;
use bevy_time::*;
use bevy_tweening::{
    lens::SpriteColorLens, Animator, Delay, EaseFunction, Tween, TweenCompleted, TweeningType,
};
use dyn_fmt::AsStrFormatExt;
use std::{ops::RangeInclusive, time::Duration};

pub struct AssetsPlugin;
impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(start_loading_assets).add_system_set(
            SystemSet::on_update(GameState::Loading).with_system(check_assets_progress),
        );
    }
}

pub struct Fonts {
    pub ui: Handle<Font>,
}

pub struct Sfx {
    pub click: Vec<Handle<AudioSource>>,
}

pub struct Sprites {
    pub bevy_logo: Handle<Image>,
}

struct LoadingAssets {
    all_handles: Vec<HandleUntyped>,
    splash_timer: Timer,
    splash_entities: Vec<Entity>,
    done: bool,
}

fn start_loading_assets(mut cmd: Commands, ass: Res<AssetServer>) {
    // splash screen imgs
    let mut splash_entities = Vec::new();

    if cfg!(not(feature = "dev")) {
        for (img_path, size) in [("sprites/bevy_logo.png", Vec2::splat(300.))].iter() {
            splash_entities.push(
                cmd.spawn_bundle(SpriteBundle {
                    texture: ass.load(*img_path).into(),
                    sprite: Sprite {
                        color: Color::NONE.into(),
                        custom_size: Some(*size),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Animator::new(Delay::new(Duration::from_millis(350)).then(
                    Tween::new(
                        EaseFunction::QuadraticInOut,
                        TweeningType::Once,
                        Duration::from_millis(500),
                        SpriteColorLens {
                            start: Color::NONE,
                            end: Color::WHITE,
                        },
                    ),
                )))
                .id(),
            );
        }
    }

    let mut loading_ass = LoadingAssets {
        all_handles: Vec::new(),
        splash_timer: Timer::from_seconds(2.5, false),
        done: false,
        splash_entities,
    };

    cmd.insert_resource(Sprites {
        bevy_logo: load_asset("sprites/bevy_logo.png", &ass, &mut loading_ass),
    });

    cmd.insert_resource(Fonts {
        ui: load_asset("fonts/FiraSans-Bold.ttf", &ass, &mut loading_ass),
    });

    cmd.insert_resource(Sfx {
        click: load_asset_variants("audio/sfx/click{}.ogg", 1..=13, &ass, &mut loading_ass),
    });

    cmd.insert_resource(loading_ass);
}

fn load_asset<T: Asset>(
    path: &str,
    ass: &AssetServer,
    loading_ass: &mut LoadingAssets,
) -> Handle<T> {
    let handle = ass.load(path);
    loading_ass.all_handles.push(handle.clone_untyped());
    handle
}

fn load_asset_variants<T: Asset>(
    path_format: &str,
    range: RangeInclusive<usize>,
    ass: &AssetServer,
    loading_ass: &mut LoadingAssets,
) -> Vec<Handle<T>> {
    let mut handles = Vec::new();

    for i in range {
        handles.push(load_asset(&path_format.format(&[i]), ass, loading_ass));
    }

    handles
}

fn check_assets_progress(
    mut cmd: Commands,
    mut state: ResMut<State<GameState>>,
    mut delayed_state: ResMut<DelayedState<GameState>>,
    mut loading_ass: ResMut<LoadingAssets>,
    server: Res<AssetServer>,
    time: ScaledTime,
) {
    if loading_ass.done {
        return;
    }

    loading_ass.splash_timer.tick(time.delta());

    match server.get_group_load_state(loading_ass.all_handles.iter().map(|h| h.id)) {
        LoadState::Failed => {
            // todo: what to do here?
        }
        LoadState::Loaded => {
            if loading_ass.splash_timer.finished() {
                loading_ass.done = true;

                if cfg!(feature = "dev") {
                    state.overwrite_set(GameState::Game).unwrap();
                } else {
                    info!("queue state");
                    delayed_state.queue_state(GameState::Menu, 1.);

                    for e in loading_ass.splash_entities.iter() {
                        cmd.entity(*e).insert(get_fade_out_sprite_anim(
                            Color::WHITE,
                            500,
                            Some(TweenDoneAction::DespawnRecursive),
                        ));
                    }
                }
            }
        }
        _ => {} // still loading
    }
}
