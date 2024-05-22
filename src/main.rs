use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle}
};
use rand::Rng;
use std::fmt;
use std::fmt::Formatter;

const GRID_SIZE: f32 = 20.;
// const X_EXTENT: f32 = 600.;
const BORDER_WIDTH: f32 = 4.;

const GRID_X: u32 = 60;
const GRID_Y: u32 = 35;

const X_EXTENT: f32 = GRID_SIZE * (GRID_X as f32);
const Y_EXTENT: f32 = GRID_SIZE * (GRID_Y as f32);

const X_MID: f32 = X_EXTENT / 2.;
const Y_MID: f32 = Y_EXTENT / 2.;
const GRID_HALF: f32 = GRID_SIZE / 2.;
const GRID_X_MID: u32 = GRID_X / 2;
const GRID_Y_MID: u32 = GRID_Y / 2;

const MOVE_TIMER_SECONDS: f32 = 0.25;
const SCORE_TEXT_TRANSLATION: Vec3 = Vec3 {x:  -595., y: 330., z: 5.};

#[derive(Copy, Clone, Ord, PartialOrd, Eq)]
struct GridCoord {
    i: i32, j: i32 // i = x; j = y;
}
impl Default for GridCoord {
    fn default() -> Self {
        return GridCoord{ i: GRID_X_MID as i32, j: GRID_Y_MID as i32 }
    }
}
impl fmt::Display for GridCoord {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.i, self.j)
    }
}

fn grid_space_to_vec(x: i32, y: i32) -> Vec2 {
    return Vec2 {
        x: (x as f32) * GRID_SIZE - X_MID + GRID_HALF,
        y: (y as f32) * GRID_SIZE - Y_MID + GRID_HALF
    }
}

fn random_cell() -> GridCoord {
    let mut rng = rand::thread_rng();
    let i = rng.gen_range(0..GRID_X) as i32;
    let j = rng.gen_range(0..GRID_Y) as i32;
    return GridCoord{i, j}
}

#[derive(Default, Copy, Clone)]
enum Direction {
    #[default]
    UP,
    DOWN,
    LEFT,
    RIGHT
}

fn delta_from_direction(direction: Direction) -> GridCoord {
    return match direction {
        Direction::UP => GridCoord    { i:  0, j:  1 },
        Direction::DOWN => GridCoord  { i:  0, j: -1 },
        Direction::LEFT => GridCoord  { i: -1, j:  0 },
        Direction::RIGHT => GridCoord { i:  1, j:  0 }
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_scoreboard(mut commands: Commands) {
    commands.spawn(
        Text2dBundle {
            text: Text::from_section("Score:", TextStyle {
                font_size: 40.,
                color: Color::YELLOW,
                ..default()
            }).with_justify(JustifyText::Left),
            transform: Transform::from_xyz(SCORE_TEXT_TRANSLATION.x, SCORE_TEXT_TRANSLATION.y, SCORE_TEXT_TRANSLATION.z),
            ..default()
        },
    );
}

fn spawn_bounds(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let bounds = Mesh2dHandle(meshes.add(Rectangle::new(X_EXTENT + (BORDER_WIDTH * 2.), Y_EXTENT + (BORDER_WIDTH * 2.))));
    commands.spawn(MaterialMesh2dBundle {
        mesh: bounds,
        material: materials.add(Color::ANTIQUE_WHITE),
        transform: Transform::from_xyz(0f32, 0f32, -5.),
        ..default()
    });

    let inner_bounds = Mesh2dHandle(meshes.add(Rectangle::new(X_EXTENT, Y_EXTENT)));
    commands.spawn(MaterialMesh2dBundle {
        mesh: inner_bounds,
        material: materials.add(Color::DARK_GRAY),
        transform: Transform::from_xyz(0f32, 0f32, -4.),
        ..default()
    });
}

fn spawn_coin(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut game: ResMut<Game>,
) {
    let circle = Mesh2dHandle(meshes.add(Circle {radius: GRID_SIZE / 2. }));

    let cell = random_cell();
    game.coin.cell = cell;
    let coordinates = grid_space_to_vec(cell.i, cell.j);

    game.coin.entity = Some(commands.spawn(MaterialMesh2dBundle {
        mesh: circle,
        material: materials.add(Color::GOLD),
        transform: Transform::from_xyz(coordinates.x, coordinates.y, -1.),
        ..default()
    }).id());
}

#[derive(Default)]
struct Player {
    entity: Option<Entity>,
    length: u32,
    cell: GridCoord,
    direction: Direction
}

#[derive(Default)]
struct Coin {
    entity: Option<Entity>,
    cell: GridCoord
}

#[derive(Resource, Default)]
struct Game {
    player: Player,
    coin: Coin,
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut game: ResMut<Game>,
) {
    let player = Mesh2dHandle(meshes.add(Rectangle::new(GRID_SIZE, GRID_SIZE)));
    let coordinates = grid_space_to_vec(game.player.cell.i, game.player.cell.j);
    game.player.entity = Some(commands.spawn(MaterialMesh2dBundle {
        mesh: player,
        material: materials.add(Color::SEA_GREEN),
        transform: Transform::from_xyz(coordinates.x, coordinates.y, 1.),
        ..default()
    }).id());
}

fn move_player(
    commands: Commands,
    mut game: ResMut<Game>,
    grid_delta: GridCoord,
    mut transforms: Query<&mut Transform>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    game.player.cell = GridCoord{
        i: game.player.cell.i + grid_delta.i,
        j: game.player.cell.j + grid_delta.j
    };
    let coordinates = grid_space_to_vec(game.player.cell.i, game.player.cell.j);

    transforms.get_mut(game.player.entity.unwrap()).unwrap().translation.x = coordinates.x;
    transforms.get_mut(game.player.entity.unwrap()).unwrap().translation.y = coordinates.y;

    check_player_on_coin(commands, game, meshes, materials)
}

#[derive(Resource)]
struct MoveTimer(Timer);

fn schedule_player_move(
    commands: Commands, time: Res<Time>,
    mut timer: ResMut<MoveTimer>,
    game: ResMut<Game>,
    transforms: Query<&mut Transform>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let direction = &game.player.direction;
        let delta = delta_from_direction(*direction);
        move_player(commands, game, delta, transforms, meshes, materials);
    }
}

fn change_player_direction(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut game: ResMut<Game>,
) {
    for kc in keyboard_input.get_just_pressed() {
        game.player.direction = match kc {
            KeyCode::ArrowUp => Direction::UP,
            KeyCode::ArrowDown => Direction::DOWN,
            KeyCode::ArrowLeft => Direction::LEFT,
            KeyCode::ArrowRight => Direction::RIGHT,
            _ => game.player.direction
        };
    }
}

impl PartialEq for GridCoord {
    fn eq(&self, other: &Self) -> bool {
        return self.i == other.i && self.j == other.j;
    }
}

fn check_player_on_coin(
    mut commands: Commands,
    mut game: ResMut<Game>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    if game.player.cell == game.coin.cell {
        game.player.length += 1;
        commands.entity(game.coin.entity.unwrap()).despawn_recursive();
        spawn_coin(commands, meshes, materials, game);
    }
}

fn update_scoreboard(
    game: ResMut<Game>,
    mut query: Query<(&mut Text, &mut Transform)>
) {
    let (mut text, mut transform) = query.single_mut();
    let text_value = format!("Score: {}", game.player.length);
    text.sections[0].value = text_value.clone();
    let text_width = measure_text_width(&text_value);
    transform.translation.x = text_width / 2.0 + SCORE_TEXT_TRANSLATION.x;
}

fn measure_text_width(text: &str) -> f32 {
    // This is a simplified example. You need to actually measure the width based on the font and size.
    // For accurate measurement, you may need to use a text renderer or font metrics.
    let character_width = 20.0; // Assume an average character width (in pixels)
    text.len() as f32 * character_width
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Game>()
        .insert_resource(MoveTimer(Timer::from_seconds(MOVE_TIMER_SECONDS, TimerMode::Repeating)))
        .add_systems(Startup, (setup_camera, spawn_coin, spawn_bounds, spawn_player, setup_scoreboard))
        .add_systems(Update, (schedule_player_move, change_player_direction, update_scoreboard))
        .run();
}
