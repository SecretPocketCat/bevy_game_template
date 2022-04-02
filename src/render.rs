use bevy::{prelude::*, render::render_resource::FilterMode};

pub struct RenderPlugin;
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system(set_img_sampler_filter);
    }
}

pub struct GameDimensions {
    pub base_size: Vec2,
    pub min_size: Vec2,
}

#[derive(Component)]
pub struct MainCamera;

fn setup(mut cmd: Commands) {
    cmd.spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);
}

fn set_img_sampler_filter(
    mut ev_asset: EventReader<AssetEvent<Image>>,
    mut assets: ResMut<Assets<Image>>,
) {
    for ev in ev_asset.iter() {
        match ev {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                if let Some(mut texture) = assets.get_mut(handle) {
                    // set sampler filtering to add some AA (quite fuzzy though)
                    texture.sampler_descriptor.mag_filter = FilterMode::Linear;
                    texture.sampler_descriptor.min_filter = FilterMode::Linear;
                }
            }
            _ => {}
        }
    }
}
