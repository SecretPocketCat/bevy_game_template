pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system_to_stage(CoreStage::PostUpdate, rotate)
            .add_system_to_stage(CoreStage::Last, follow_scale)
            .add_system_to_stage(CoreStage::Last, follow_position);
    }
}

#[derive(Bundle, Default)]
pub struct TransformBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl TransformBundle {
    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self {
            transform: Transform::from_xyz(x, y, z),
            ..Default::default()
        }
    }
}

#[derive(Default, Component, Inspectable)]
pub struct TransformRotation {
    pub rotation_rad: f32,
    pub rotation_max_rad: f32,
}

impl TransformRotation {
    pub fn new(rotation_rad: f32) -> Self {
        Self {
            rotation_rad,
            rotation_max_rad: rotation_rad,
        }
    }
}

#[derive(Component, Inspectable)]
pub struct FollowScale {
    followed_e: Entity,
    scale_multiplier: Vec3,
}

#[derive(Component, Inspectable)]
pub struct FollowPosition {
    followed_e: Entity,
    offset: Vec3,
}

fn rotate(mut q: Query<(&TransformRotation, &mut Transform)>, time: ScaledTime) {
    for (r, mut t) in q.iter_mut() {
        t.rotate(Quat::from_rotation_z(
            r.rotation_rad * time.scaled_delta_seconds(),
        ));
    }
}

fn follow_scale(follow_q: Query<(Entity, &FollowScale)>, mut transform_q: Query<&mut Transform>) {
    for (following_e, follow) in follow_q.iter() {
        if let Ok(followed_t) = transform_q.get(follow.followed_e) {
            if let Ok(mut following_t) = transform_q.get_mut(following_e) {
                following_t.scale = followed_t.scale * follow.scale_multiplier;
            }
        }
    }
}

fn follow_position(
    follow_q: Query<(Entity, &FollowPosition)>,
    mut transform_q: Query<&mut Transform>,
) {
    for (following_e, follow) in follow_q.iter() {
        if let Ok(followed_t) = transform_q.get(follow.followed_e) {
            if let Ok(mut following_t) = transform_q.get_mut(following_e) {
                following_t.translation = Vec3::new(
                    followed_t.translation.x,
                    followed_t.translation.y,
                    following_t.translation.z,
                ) + follow.offset;
            }
        }
    }
}
