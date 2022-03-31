use bevy::prelude::*;
use bevy_time::*;

pub struct PausePlugin;
impl Plugin for PausePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(process_inactive);
    }
}

#[derive(Component)]
pub enum Inactive {
    Timed { timer: Timer },
    Permanent,
}

fn process_inactive(
    mut commands: Commands,
    mut inactive_q: Query<(Entity, &mut Inactive)>,
    time: ScaledTime,
) {
    for (inactive_e, mut inactive) in inactive_q.iter_mut() {
        match &mut *inactive {
            Inactive::Timed { timer } => {
                timer.tick(time.scaled_delta());
                if timer.just_finished() {
                    commands.entity(inactive_e).remove::<Inactive>();
                }
            }
            Inactive::Permanent => {}
        }
    }
}
