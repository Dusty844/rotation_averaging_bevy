use bevy::prelude::*;
use turborand::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod cam;

use cam::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CameraControllerPlugin))
        .add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, setup)
        .add_systems(Update, (set_targets, debug_direction))
        .register_type::<(AverageIn, AverageOut, AverageVariant, AverageVariantResource)>()
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let rand = Rng::new();
    let count = 3;
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

    commands.spawn((
        Mesh3d(meshes.add(Cone::default())),
        MeshMaterial3d(materials.add(StandardMaterial{
            base_color: Color::srgb(0.5, 0.75, 0.7),
            ..Default::default()
        })),
        Transform::default(),
        AverageOut,
        Name::new("out"),
        ));

    
    for i in 0..count {
        let dir = Vec3::new(rand.f32_normalized() * 5.0, rand.f32_normalized() * 5.0, rand.f32_normalized() * 5.0);
        
        commands.spawn((
            Mesh3d(meshes.add(Cone::default())),
            MeshMaterial3d(materials.add(StandardMaterial{
                base_color: Color::srgb(0.8, 0.7, 0.7),
                ..Default::default()
            })),
            Transform::from_rotation(Quat::from_scaled_axis(dir)).with_translation(dir.normalize() * 4.0),
            AverageIn::default(),
            Name::new("target"),
        ));
    }
    
    
}

#[derive(Component, Copy, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct AverageIn{
    pub rotation: Vec3,
}

#[derive(Component, Copy, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct AverageOut;

#[derive(Resource, Copy, Clone, Reflect, Default)]
#[reflect(Resource)]
pub struct AverageVariantResource{
    pub current: AverageVariant,
}

#[derive(Default, Clone, Copy, PartialEq, Reflect)]
pub enum AverageVariant{
    #[default]
    Regular,
    Eigen,
    
}

fn set_targets(
    mut targets: Query<(&mut AverageIn, &GlobalTransform), Or<(Added<AverageIn>, Added<GlobalTransform>, Changed<GlobalTransform>)>>,
    mut out: Single<(&mut Transform, &GlobalTransform, &AverageOut), Without<AverageIn>>
) {
    let mut rots = Vec::new();
    for (mut target, gt) in targets.iter_mut() {
        rots.push(gt.rotation());
        let dir = gt.rotation().to_scaled_axis();
        target.rotation = dir;
    }
    out.0.rotation = average_regular(&rots).normalize();
    

}

fn average_regular(
    vec: &Vec<Quat>,
) -> Quat{
    let mut average = Vec3::ZERO;
    let mut d = 0.0;
    for quat in vec{
        average += quat.to_scaled_axis();
        d += 1.0;
    }
    //average /= d;
    Quat::from_scaled_axis(average)
    
}

fn debug_direction(
    mut gizmos: Gizmos,
    targets: Query<(&GlobalTransform), Or<(With<AverageIn>, With<AverageOut>)>>
){
    for gt in targets.iter(){
        let pos = gt.translation();
        let dir = gt.rotation().normalize() * Vec3::Y;
        gizmos.arrow(pos, pos + (dir * 2.0), Color::WHITE);
    }
    
}
