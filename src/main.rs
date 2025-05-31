use bevy::prelude::*;
use turborand::prelude::*;

mod cam;

use cam::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CameraControllerPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, set_targets)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let rand = Rng::new();
    let count = 10;
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        CameraController::default(),
    ));

    
    for i in 0..count {
        let dir = Vec3::new(rand.f32_normalized() * 5.0, rand.f32_normalized() * 5.0, rand.f32_normalized() * 5.0);
        
        commands.spawn((
            Mesh3d(meshes.add(Tetrahedron::default())),
            MeshMaterial3d(materials.add(StandardMaterial{
                base_color: Color::srgb(0.8, 0.7, 0.7),
                ..Default::default()
            })),
            Transform::from_rotation(Quat::from_scaled_axis(dir)).with_translation(dir.normalize() * 4.0),
            AverageTarget::default(),
            Name::new("target"),
        ));
    }
    
    
}

#[derive(Component, Copy, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct AverageTarget{
    pub rotation: Vec3,
}

fn set_targets(
    mut targets: Query<(&mut AverageTarget, &GlobalTransform), Or<(Added<AverageTarget>, Added<GlobalTransform>, Changed<GlobalTransform>)>>
) {
    for (mut target, gt) in targets.iter_mut() {
        let dir = gt.rotation().to_scaled_axis();
        target.rotation = dir;
    }
}
