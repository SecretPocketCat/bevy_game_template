use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_inspector_egui::{InspectorPlugin, RegisterInspectable, WorldInspectorPlugin};
use bevy_prototype_lyon::prelude::Path;
use bevy_time::ScaledTime;

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(WorldInspectorPlugin::new())
            // res inspector example
            // .add_plugin(InspectorPlugin::<>::new())
            // inspectable example
            // .register_inspectable::<>()
            .add_startup_system(test_setup)
            .add_system(test_system);
    }
}

fn test_setup(_commands: Commands) {}

fn test_system(_path_q: Query<&mut Path>, _time: ScaledTime) {}
