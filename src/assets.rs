use crate::GameState;
use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};
use bevy_kira_audio::AudioSource;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        AssetLoader::new(GameState::Loading)
            .with_collection::<Fonts>()
            .with_collection::<Audio>()
            .with_collection::<Sprites>()
            .continue_to_state(GameState::Menu)
            .build(app);
    }
}

#[derive(AssetCollection)]
pub struct Fonts {
    #[asset(path = "fonts/FiraSans-Bold.ttf")]
    pub fira_sans: Handle<Font>,
}

#[derive(AssetCollection)]
pub struct Audio {
    #[asset(path = "audio/sfx", folder)]
    _sfx: Vec<HandleUntyped>,
}

#[derive(AssetCollection)]
pub struct Sprites {
    #[asset(path = "sprites/bevy_logo.png")]
    pub bevy_logo: Handle<Image>,
}
