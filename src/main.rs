use std::ops::{Deref, Sub};

use bevy::prelude::*;
use turborand::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::{inspector_options::InspectorOptionsType, quick::WorldInspectorPlugin};

mod cam;

use cam::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CameraControllerPlugin))
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, setup)
        .add_systems(Update, (set_targets, debug_direction))
        .insert_resource(AverageVariantResource::default())
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let rand = Rng::new();
    let count = 2;
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
        Transform::from_xyz(-2.0, 0.0, 0.0),
        AverageOut{
            variant: AverageVariant::Regular,
        },
        Name::new("out regular"),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cone::default())),
        MeshMaterial3d(materials.add(StandardMaterial{
            base_color: Color::srgb(0.2, 0.9, 0.25),
            ..Default::default()
        })),
        Transform::from_xyz(-1.0, 0.0, 0.0),
        AverageOut{
            variant: AverageVariant::Euler,
        },
        Name::new("out euler"),
    ));
    
    commands.spawn((
        Mesh3d(meshes.add(Cone::default())),
        MeshMaterial3d(materials.add(StandardMaterial{
            base_color: Color::srgb(0.9, 0.25, 0.25),
            ..Default::default()
        })),
        Transform::default(),
        AverageOut{
            variant: AverageVariant::Covariance(5),
        },
        Name::new("out covariance 1"),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cone::default())),
        MeshMaterial3d(materials.add(StandardMaterial{
            base_color: Color::srgb(0.9, 0.8, 0.6),
            ..Default::default()
        })),
        Transform::from_xyz(-3.0, 0.0, 0.0),
        AverageOut{
            variant: AverageVariant::Covariance2(5),
        },
        Name::new("out covariance 2"),
    ));
    

    let mut i_f = 1.0;
    for i in 0..count {
        let dir = Vec3::new(rand.f32_normalized() * 5.0, rand.f32_normalized() * 5.0, rand.f32_normalized() * 5.0);
        
        
        commands.spawn((
            Mesh3d(meshes.add(Cone::default())),
            MeshMaterial3d(materials.add(StandardMaterial{
                base_color: Color::srgb(0.8, 0.7, 0.7),                ..Default::default()
            })),
            Transform::from_xyz(i_f, 0.0, 0.0),
            AverageIn::default(),
            Name::new("target"),
        ));
        i_f += 1.0;
    }
    
    
}

#[derive(Component, Copy, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct AverageIn{
    pub rotation: Vec3,
}

#[derive(Component, Copy, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct AverageOut{
    variant: AverageVariant,
}

#[derive(Resource, Copy, Clone, Reflect, Default)]
#[reflect(Resource)]
pub struct AverageVariantResource{
    pub current: AverageVariant,
}

#[derive(Default, Clone, Copy, PartialEq, Reflect)]
pub enum AverageVariant{
    #[default]
    Regular,
    Euler,
    Covariance(usize),
    Covariance2(usize)
    
}

fn set_targets(
    mut targets: Query<(&mut AverageIn, &GlobalTransform)>,
    mut out: Query<(&mut Transform, &AverageOut), Without<AverageIn>>
) {
    let mut rots = Vec::new();
    for (mut target, gt) in targets.iter_mut() {
        rots.push(gt.rotation());
        let dir = gt.rotation().to_scaled_axis();
        target.rotation = dir;
    }
    for (mut t, ao) in out.iter_mut(){
        match ao.variant{
            AverageVariant::Regular => {
                t.rotation = average_regular(&rots);
            }
            AverageVariant::Euler => {
                t.rotation = average_euler(&rots);
            }
            AverageVariant::Covariance(x) => {
                t.rotation = average_covaraince(&rots, x);
            }
            AverageVariant::Covariance2(x) => {
                t.rotation = accurate_quat_average(&rots, x, Quat::IDENTITY)
            }
        }
    }
    
    

}

fn average_euler(
    vec: &Vec<Quat>,
) -> Quat{
    let mut average = Vec3::ZERO;
    let mut d = 0.0;
    for quat in vec {
        let euler: Vec3 = quat.to_euler(EulerRot::XYZ).into();
        average += euler;
        d += 1.0;
    }
    if average.length() > 0.001 {
        average /= d;
    }
    Quat::from_euler(EulerRot::XYZ,average.x, average.y, average.z).normalize()
    // Quat::IDENTITY
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
    average /= d;
    Quat::from_scaled_axis(average).normalize()
    
}

// based off of the following paper: https://users.cecs.anu.edu.au/~hartley/Papers/PDF/Hartley-Trumpf:Rotation-averaging:IJCV.pdf
// did my best to copy what is there.
fn average_covaraince(
    vec: &Vec<Quat>,
    x: usize,
) -> Quat{
    let mut a = [[0.0; 4]; 4];
    for quat in vec {
        for i in 0..4 {
            for j in 0..4 {
                a[i][j] += quat.to_array()[i] * quat.to_array()[j];
            }
        }
    }

    let mut eigenvec = Quat::from_array([0.5; 4]);
    eigenvec = eigenvec.normalize();

    for _ in 0..x {
        let mut av = [0.0; 4];
        for i in 0..4{
            for j in 0..4{
                av[i] += a[i][j] * eigenvec.to_array()[j]
            }
        }
        eigenvec = Quat::from_array(av).normalize();
        
    }
    if eigenvec.to_array()[0] < 0.0 {
        let mut temp = eigenvec.to_array();
        temp.iter_mut().for_each(|x| *x *= -1.0);
        eigenvec = Quat::from_array(temp);
    }
    eigenvec
    
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


pub fn accurate_quat_average(
    quats: &Vec<Quat>,
    quality_count: usize,
    start_quat: Quat,
)-> Quat {
    let mut accum = Mat4::ZERO;

    for (i, q) in quats.iter().enumerate() {
        // assumes that quats are xyzw instead of wxyz.. i hope
        accum += mat4(
            vec4(q.w * q.w, q.w * q.x, q.w * q.y, q.w * q.z),
            vec4(q.x * q.w, q.x * q.x, q.x * q.y, q.x * q.z),
            vec4(q.y * q.w, q.y * q.x, q.y * q.y, q.y * q.z),
            vec4(q.z * q.w, q.z * q.x, q.z * q.y, q.z * q.z),
            
        );  
    }
    let u = svd_dominant_eigen(accum, start_quat.into(), quality_count, 0.0001);
    let v = (accum * u).normalize();

    quat_abs(Quat::from_xyzw(v.w, v.x, v.y, v.z))
}

fn svd_dominant_eigen(
    a: Mat4,
    v0: Vec4,
    iterations: usize,
    epsilon: f32,
) -> Vec4{
    let mut v = v0;
    let mut ev = ((a * v) / v).x;

    for i in 0..iterations {

        
        let av = a * v;

        let v_new = av.normalize();
        let ev_new = ((a * v_new) / v_new).x;

        if f32::abs(ev - ev_new) < epsilon{
            break;
        }
        v = v_new;
        ev = ev_new;
    }

    v
}


fn quat_abs(a: Quat) -> Quat {
        if a.w.is_sign_negative() {
            -a
        }else {
            a
        }
    }
