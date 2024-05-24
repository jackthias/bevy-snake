use std::cmp::PartialEq;
use std::fmt;
use std::fmt::Formatter;

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use rand::Rng;

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

const MOVE_TIMER_SECONDS: f32 = 1. / 12.;
const SCORE_TEXT_TRANSLATION: Vec3 = Vec3 {
    x: -595.,
    y: 330.,
    z: 5.,
};

const PLAYER_START_LENGTH: u32 = 3;

#[derive(Copy, Clone, Ord, PartialOrd, Eq)]
struct GridCoord {
    i: i32,
    j: i32, // i = x; j = y;
}

impl Default for GridCoord {
    fn default() -> Self {
        return GridCoord {
            i: GRID_X_MID as i32,
            j: GRID_Y_MID as i32,
        };
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
        y: (y as f32) * GRID_SIZE - Y_MID + GRID_HALF,
    };
}

fn random_cell() -> GridCoord {
    let mut rng = rand::thread_rng();
    let i = rng.gen_range(0..GRID_X) as i32;
    let j = rng.gen_range(0..GRID_Y) as i32;
    return GridCoord { i, j };
}

#[derive(Default, Copy, Clone, PartialEq)]
enum Direction {
    #[default]
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

fn delta_from_direction(direction: Direction) -> GridCoord {
    return match direction {
        Direction::UP => GridCoord { i: 0, j: 1 },
        Direction::DOWN => GridCoord { i: 0, j: -1 },
        Direction::LEFT => GridCoord { i: -1, j: 0 },
        Direction::RIGHT => GridCoord { i: 1, j: 0 },
    };
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_scoreboard(mut commands: Commands) {
    commands.spawn(Text2dBundle {
        text: Text::from_section(
            "Score:",
            TextStyle {
                font_size: 40.,
                color: Color::YELLOW,
                ..default()
            },
        )
        .with_justify(JustifyText::Left),
        transform: Transform::from_xyz(
            SCORE_TEXT_TRANSLATION.x,
            SCORE_TEXT_TRANSLATION.y,
            SCORE_TEXT_TRANSLATION.z,
        ),
        ..default()
    });
}

fn spawn_bounds(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let bounds = Mesh2dHandle(meshes.add(Rectangle::new(
        X_EXTENT + (BORDER_WIDTH * 2.),
        Y_EXTENT + (BORDER_WIDTH * 2.),
    )));
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: bounds,
            material: materials.add(Color::ANTIQUE_WHITE),
            transform: Transform::from_xyz(0f32, 0f32, -5.),
            ..default()
        },
        SaveOnTeardown {},
    ));

    let inner_bounds = Mesh2dHandle(meshes.add(Rectangle::new(X_EXTENT, Y_EXTENT)));
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: inner_bounds,
            material: materials.add(Color::DARK_GRAY),
            transform: Transform::from_xyz(0f32, 0f32, -4.),
            ..default()
        },
        SaveOnTeardown {},
    ));
}

fn spawn_coin(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut game: ResMut<Game>,
) {
    let circle = Mesh2dHandle(meshes.add(Circle {
        radius: GRID_SIZE / 2.,
    }));

    let cell = random_cell();
    game.coin.cell = cell;
    let coordinates = grid_space_to_vec(cell.i, cell.j);

    game.coin.entity = Some(
        commands
            .spawn(MaterialMesh2dBundle {
                mesh: circle,
                material: materials.add(Color::GOLD),
                transform: Transform::from_xyz(coordinates.x, coordinates.y, -1.),
                ..default()
            })
            .id(),
    );
}

#[derive(Default)]
struct Segment {
    cell: GridCoord,
    direction: Direction,
    entity: Option<Entity>,
}

#[derive(Default)]
struct Player {
    entity: Option<Entity>,
    length: u32,
    cell: GridCoord,
    direction: Direction,
    segments: Vec<Segment>,
}

#[derive(Default)]
struct Coin {
    entity: Option<Entity>,
    cell: GridCoord,
}

#[derive(Default, Clone, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Playing,
    GameOver,
}

#[derive(Resource, Default)]
struct Game {
    player: Player,
    coin: Coin,
    game_state: GameState,
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut game: ResMut<Game>,
) {
    game.player.cell = GridCoord::default();
    game.player.segments = Vec::new();
    let player = Mesh2dHandle(meshes.add(Rectangle::new(GRID_SIZE, GRID_SIZE)));
    let coordinates = grid_space_to_vec(game.player.cell.i, game.player.cell.j);
    game.player.entity = Some(
        commands
            .spawn(MaterialMesh2dBundle {
                mesh: player,
                material: materials.add(Color::SEA_GREEN),
                transform: Transform::from_xyz(coordinates.x, coordinates.y, 1.),
                ..default()
            })
            .id(),
    );

    game.player.length = PLAYER_START_LENGTH;
    let mut segments = Vec::new();
    segments.push(Segment {
        cell: game.player.cell,
        entity: game.player.entity,
        ..default()
    });

    for i in 1..game.player.length {
        let cell = GridCoord {
            i: game.player.cell.i,
            j: game.player.cell.j - i as i32,
        };
        let segment = Mesh2dHandle(meshes.add(Rectangle::new(GRID_SIZE, GRID_SIZE)));
        let segment_coordinates = grid_space_to_vec(cell.i, cell.j);
        let entity = Some(
            commands
                .spawn(MaterialMesh2dBundle {
                    mesh: segment,
                    material: materials.add(Color::SEA_GREEN),
                    transform: Transform::from_xyz(
                        segment_coordinates.x,
                        segment_coordinates.y,
                        1.,
                    ),
                    ..default()
                })
                .id(),
        );
        segments.push(Segment {
            cell,
            entity,
            ..default()
        })
    }
    game.player.segments = segments;
}

fn move_player(
    commands: Commands,
    mut game: ResMut<Game>,
    grid_delta: GridCoord,
    mut transforms: Query<&mut Transform>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    if game.game_state != GameState::Playing {
        return;
    };
    game.player.cell = GridCoord {
        i: game.player.cell.i + grid_delta.i,
        j: game.player.cell.j + grid_delta.j,
    };
    let coordinates = grid_space_to_vec(game.player.cell.i, game.player.cell.j);

    let mut old_translation = transforms
        .get_mut(game.player.entity.unwrap())
        .unwrap()
        .translation
        .clone();
    transforms
        .get_mut(game.player.entity.unwrap())
        .unwrap()
        .translation
        .x = coordinates.x;
    transforms
        .get_mut(game.player.entity.unwrap())
        .unwrap()
        .translation
        .y = coordinates.y;
    let mut old_cell = game.player.segments[0].cell.clone();
    let mut old_direction = game.player.segments[0].direction.clone();
    game.player.segments[0].cell = game.player.cell.clone();
    game.player.segments[0].direction = game.player.direction.clone();

    for segment in &mut game.player.segments[1..] {
        let tmp_translation = transforms
            .get_mut(segment.entity.unwrap())
            .unwrap()
            .translation
            .clone();
        let tmp_direction = segment.direction;
        let tmp_cell = segment.cell;
        transforms
            .get_mut(segment.entity.unwrap())
            .unwrap()
            .translation = old_translation.clone();
        segment.direction = old_direction;
        segment.cell = old_cell;
        old_translation = tmp_translation.clone();
        old_cell = tmp_cell.clone();
        old_direction = tmp_direction.clone();
    }

    check_player_on_coin(commands, game, meshes, materials)
}

#[derive(Resource)]
struct MoveTimer(Timer);

fn schedule_player_move(
    commands: Commands,
    time: Res<Time>,
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

fn change_player_direction(keyboard_input: Res<ButtonInput<KeyCode>>, mut game: ResMut<Game>) {
    for kc in keyboard_input.get_just_pressed() {
        game.player.direction = match kc {
            KeyCode::ArrowUp => {
                if game.player.direction == Direction::DOWN {
                    Direction::DOWN
                } else {
                    Direction::UP
                }
            }
            KeyCode::ArrowDown => {
                if game.player.direction == Direction::UP {
                    Direction::UP
                } else {
                    Direction::DOWN
                }
            }
            KeyCode::ArrowLeft => {
                if game.player.direction == Direction::RIGHT {
                    Direction::RIGHT
                } else {
                    Direction::LEFT
                }
            }
            KeyCode::ArrowRight => {
                if game.player.direction == Direction::LEFT {
                    Direction::LEFT
                } else {
                    Direction::RIGHT
                }
            }
            _ => game.player.direction,
        };
    }
}

#[derive(Component)]
struct SaveOnTeardown();

impl PartialEq for GridCoord {
    fn eq(&self, other: &Self) -> bool {
        return self.i == other.i && self.j == other.j;
    }
}

fn check_player_on_coin(
    mut commands: Commands,
    mut game: ResMut<Game>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if game.player.cell == game.coin.cell {
        game.player.length += 1;
        commands
            .entity(game.coin.entity.unwrap())
            .despawn_recursive();

        let last_segment_cell = game.player.segments.last().unwrap().cell;
        let last_segment_direction = game.player.segments.last().unwrap().direction;
        let segment = Mesh2dHandle(meshes.add(Rectangle::new(GRID_SIZE, GRID_SIZE)));
        let segment_cell = match last_segment_direction {
            Direction::UP => GridCoord {
                i: last_segment_cell.i,
                j: last_segment_cell.j - 1,
            },
            Direction::DOWN => GridCoord {
                i: last_segment_cell.i,
                j: last_segment_cell.j + 1,
            },
            Direction::LEFT => GridCoord {
                i: last_segment_cell.i + 1,
                j: last_segment_cell.j,
            },
            Direction::RIGHT => GridCoord {
                i: last_segment_cell.i - 1,
                j: last_segment_cell.j,
            },
        };
        let segment_coordinates = grid_space_to_vec(segment_cell.i, segment_cell.j);
        game.player.segments.push(Segment {
            cell: segment_cell,
            direction: last_segment_direction.clone(),
            entity: Some(
                commands
                    .spawn(MaterialMesh2dBundle {
                        mesh: segment,
                        material: materials.add(Color::SEA_GREEN),
                        transform: Transform::from_xyz(
                            segment_coordinates.x,
                            segment_coordinates.y,
                            1.,
                        ),
                        ..default()
                    })
                    .id(),
            ),
        });

        spawn_coin(commands, meshes, materials, game);
    }
}

fn update_scoreboard(game: ResMut<Game>, mut query: Query<(&mut Text, &mut Transform)>) {
    for (mut text, mut transform) in query.iter_mut() {
        if text.sections[0].value.contains("Score") {
            let text_value = format!("Score: {}", game.player.length - PLAYER_START_LENGTH);
            text.sections[0].value = text_value.clone();
            let text_width = measure_text_width(&text_value);
            transform.translation.x = text_width / 2.0 + SCORE_TEXT_TRANSLATION.x;
        }
    }
}

fn measure_text_width(text: &str) -> f32 {
    // This is a simplified example. You need to actually measure the width based on the font and size.
    // For accurate measurement, you may need to use a text renderer or font metrics.
    let character_width = 20.0; // Assume an average character width (in pixels)
    text.len() as f32 * character_width
}

fn check_player_in_bounds(game: Res<Game>, next_state: ResMut<NextState<GameState>>) {
    if game.player.cell.i < 0
        || game.player.cell.j < 0
        || game.player.cell.i >= GRID_X as i32
        || game.player.cell.j >= GRID_Y as i32
    {
        println!("Out of bounds!");
        end_game(next_state, game);
    }
}

fn end_game(mut next_state: ResMut<NextState<GameState>>, game: Res<Game>) {
    println!("Ending game: {}", game.player.cell);
    next_state.set(GameState::GameOver);
}

fn show_game_over(mut commands: Commands, game: Res<Game>) {
    let text = format!(
        "Game Over!\nScore: {}\nPress Space to Restart",
        game.player.length as i32 - PLAYER_START_LENGTH as i32
    );
    commands.spawn(Text2dBundle {
        text: Text::from_section(
            text,
            TextStyle {
                font_size: 40.,
                color: Color::RED,
                ..default()
            },
        )
        .with_justify(JustifyText::Center),
        transform: Transform::from_xyz(0., 0., 10.),
        ..default()
    });
}

fn check_restart_gameover(
    mut next_state: ResMut<NextState<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing)
    }
}

fn check_player_overlap_self(next_state: ResMut<NextState<GameState>>, game: Res<Game>) {
    let mut colliding = false;
    for segment in &game.player.segments[1..] {
        if segment.cell == game.player.cell {
            colliding = true;
            break;
        }
    }
    if colliding {
        println!("Overlap self!");
        end_game(next_state, game);
    }
}

fn teardown(
    mut commands: Commands,
    entities: Query<Entity, (Without<Camera>, Without<Window>, Without<SaveOnTeardown>)>,
) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Game>()
        .insert_resource(MoveTimer(Timer::from_seconds(
            MOVE_TIMER_SECONDS,
            TimerMode::Repeating,
        )))
        .init_state::<GameState>()
        .add_systems(Startup, (setup_camera, spawn_bounds))
        .add_systems(
            OnEnter(GameState::Playing),
            (spawn_coin, spawn_player, setup_scoreboard),
        )
        .add_systems(
            Update,
            (
                check_restart_gameover.run_if(in_state(GameState::GameOver)),
                bevy::window::close_on_esc,
            ),
        )
        .add_systems(
            Update,
            (
                schedule_player_move,
                change_player_direction,
                update_scoreboard,
                check_player_in_bounds,
                check_player_overlap_self,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnExit(GameState::Playing), teardown)
        .add_systems(OnEnter(GameState::GameOver), show_game_over)
        .add_systems(OnExit(GameState::GameOver), teardown)
        .run();
}
