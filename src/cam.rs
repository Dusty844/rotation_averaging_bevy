
use bevy::prelude::*;


use std::f32::consts::PI;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::window::{CursorGrabMode, CursorOptions};

pub struct CameraControllerPlugin;


impl Plugin for CameraControllerPlugin{
    fn build(&self, app: &mut App) {
        app.add_systems(Update, cam_controller);
        app.register_type::<CameraController>();
    }
}

#[derive(Component, Copy, Clone, Reflect)]
pub struct CameraController{
    pub mouse_sensitivity: f32,
    pub speed: f32,
    pub shift_multi: f32,
}

impl Default for CameraController {
    fn default() -> Self {
     CameraController { mouse_sensitivity: 1.0, speed: 2.0, shift_multi: 2.0 }        
    }
}

fn cam_controller(
    mut camera_q: Query<(&mut Transform, &CameraController), With<Camera>>,
    mut cursor: Single<&mut CursorOptions>,
    window: Single<&Window>,
    key: Res<ButtonInput<KeyCode>>,
    mouse_movement: Res<AccumulatedMouseMotion>,
    time: Res<Time<Virtual>>,
){

    let delta = time.delta_secs();

    let cursor_lock = cursor.grab_mode == CursorGrabMode::Locked;

    let (mut cam_transform, cam_controller) = camera_q.single_mut().unwrap();

    if mouse_movement.delta != Vec2::ZERO && cursor_lock{

        //camera movement
        let mut mouse_delta = mouse_movement.delta;
        
        let sensitivity = cam_controller.mouse_sensitivity;
        mouse_delta *= sensitivity;

        let delta_x = mouse_delta.x / window.width() * PI;
        let delta_y = mouse_delta.y / window.height() * PI;
        let yaw = Quat::from_rotation_y(-delta_x);
        let pitch = Quat::from_rotation_x(-delta_y);
        
        cam_transform.rotation = yaw * cam_transform.rotation;

        let pitch_rotation = cam_transform.rotation * pitch;

        let up_vector = pitch_rotation * Vec3::Y;

        if up_vector.y > 0.01 {
            
            cam_transform.rotation = pitch_rotation;
        }

        cam_transform.rotation = pitch_rotation;
    }

    let mut speed = 2.0;

    let mut direction = Vec3::ZERO;
    
    if key.pressed(KeyCode::KeyW) {
        direction += cam_transform.forward().as_vec3();

    }
    if key.pressed(KeyCode::KeyS) {
        direction += cam_transform.back().as_vec3();

    }
    if key.pressed(KeyCode::KeyA) {
        direction += cam_transform.left().as_vec3();

    }
    if key.pressed(KeyCode::KeyD) {
        direction += cam_transform.right().as_vec3();

    }
    if key.pressed(KeyCode::Space) {
        direction += cam_transform.up().as_vec3();

    }
    if key.pressed(KeyCode::ControlLeft) {
        direction += cam_transform.down().as_vec3();

    }
    if key.pressed(KeyCode::ShiftLeft) {
        speed *= 4.0;

    }

    direction = direction.normalize_or_zero();

    

    cam_transform.translation += direction * speed * delta;

    if key.just_pressed(KeyCode::KeyZ) {
        if cursor.visible{
            cursor.visible = false;
            cursor.grab_mode = CursorGrabMode::Locked
        }else{
            cursor.visible = true;
            cursor.grab_mode = CursorGrabMode::None
        }
    }
}

