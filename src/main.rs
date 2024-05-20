use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle}
};

#[derive(Component)]
struct GameCamera;

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}


fn spawn_coin(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    let circle = Mesh2dHandle(meshes.add(Circle {radius: 50.0}));

    commands.spawn(MaterialMesh2dBundle {
       mesh: circle,
        material: materials.add(Color::AQUAMARINE),
        transform: Transform::from_xyz(
            0f32, 0f32, 0f32
        ),
        ..default()
    });
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_camera, spawn_coin))
        .run();
}
