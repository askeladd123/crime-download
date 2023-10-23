use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_svg::prelude::*;
use rand::Rng;

const PI: f32 = std::f32::consts::PI;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Msaa::Sample4)
        .add_state::<State>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "SVG Plugin".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(bevy_svg::prelude::SvgPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update_enemies,
                keyboard_input,
                apply_velocity,
                pull_inside_bounds,
                check_game_over,
            )
                .run_if(in_state(State::InGame)),
        )
        .add_systems(OnEnter(State::GameOver), game_over)
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum State {
    #[default]
    InGame,
    GameOver,
}

#[derive(Component)]
struct Player {
    dash_cooldown: Timer,
    dash_duration: Timer,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            dash_cooldown: Timer::from_seconds(0.8, TimerMode::Once),
            dash_duration: Timer::from_seconds(0.1, TimerMode::Once),
        }
    }
}

#[derive(Component)]
struct Enemy {
    // change to change_goal
    change_goal: Timer,
    angle: f32,
    goal: Vec2,
}

impl Default for Enemy {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            change_goal: Timer::from_seconds(rng.gen_range(1.0..10.0), TimerMode::Repeating),

            angle: Default::default(),
            goal: Default::default(),
        }
    }
}

#[derive(Component, Default, Copy, Clone, Debug)]
struct Velocity(Vec2);

fn game_over(mut cmd: Commands) {
    cmd.spawn(Text2dBundle {
        text: Text::from_section(
            "game over",
            TextStyle {
                font_size: 100.,
                ..default()
            },
        ),
        transform: Transform {
            translation: Vec3 {
                z: 100.,
                ..default()
            },
            ..default()
        },
        ..default()
    });
}

fn check_game_over(
    mut next_state: ResMut<NextState<State>>,
    query_player: Query<&Transform, With<Player>>,
    query_enemies: Query<&Transform, With<Enemy>>,
) {
    const BBOX_SIZE: f32 = 50.;
    let player = query_player.single();
    for enemy in query_enemies.iter() {
        if bevy::sprite::collide_aabb::collide(
            player.translation,
            Vec2::new(BBOX_SIZE, BBOX_SIZE),
            enemy.translation,
            Vec2::new(BBOX_SIZE, BBOX_SIZE),
        )
        .is_some()
        {
            next_state.set(State::GameOver);
            return;
        }
    }
}

fn setup(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query_window: Query<&Window>,
) {
    const STARTING_ENEMIES: u32 = 5;

    let mut rng = rand::thread_rng();
    let window = query_window.single();
    cmd.spawn(Camera2dBundle::default());
    // cmd.spawn(Text2dBundle {
    //     text: Text::from_section("translation", TextStyle::default()),
    //     ..default()
    // });

    for _ in 0..STARTING_ENEMIES {
        cmd.spawn((
            Enemy {
                goal: Vec2 {
                    x: rng.gen_range((-window.width() / 2.0)..(window.width() / 2.0)),
                    y: rng.gen_range((-window.height() / 2.0)..(window.height() / 2.0)),
                },
                ..default()
            },
            TransformBundle {
                local: Transform {
                    translation: Vec3 {
                        x: -window.width() / 2.0,
                        y: -window.height() / 2.0,
                        ..default()
                    },
                    ..default()
                },
                ..default()
            },
            Velocity::default(),
            ComputedVisibility::default(),
            Visibility::Visible,
        ))
        .with_children(|cmd| {
            // cmd.spawn(MaterialMesh2dBundle {
            //     mesh: meshes
            //         .add(shape::Quad::new(Vec2::new(50., 50.)).into())
            //         .into(),
            //     material: materials.add(ColorMaterial::from(Color::LIME_GREEN)),
            //     ..default()
            // });
            cmd.spawn(Svg2dBundle {
                svg: asset_server.load("police.svg"),
                transform: Transform {
                    scale: Vec3 {
                        x: 1.5,
                        y: 1.5,
                        ..default()
                    },
                    translation: Vec3 {
                        x: -25.,
                        y: 25.,
                        ..default()
                    },
                    ..default()
                },
                ..default()
            });
        });
    }

    cmd.spawn(SpriteBundle {
        texture: asset_server.load("floor.jpg"),
        transform: Transform {
            translation: Vec3 {
                z: -1.,
                ..default()
            },
            scale: Vec3 {
                x: 0.25,
                y: 0.25,
                ..default()
            },
            ..default()
        },
        ..default()
    });
    cmd.spawn((
        Player::default(),
        TransformBundle::default(),
        Velocity::default(),
        ComputedVisibility::default(),
        Visibility::Visible,
    ))
    .with_children(|cmd| {
        // cmd.spawn(MaterialMesh2dBundle {
        //     mesh: meshes
        //         .add(shape::Quad::new(Vec2::new(50., 50.)).into())
        //         .into(),
        //     material: materials.add(ColorMaterial::from(Color::LIME_GREEN)),
        //     ..default()
        // });
        cmd.spawn(Svg2dBundle {
            svg: asset_server.load("thief.svg"),
            transform: Transform {
                translation: Vec3 {
                    x: -25.,
                    y: 25.,
                    z: 10.,
                    ..Default::default()
                },
                scale: Vec3 {
                    x: 0.1,
                    y: 0.1,
                    ..default()
                },
                ..default()
            },
            origin: Origin::TopLeft,
            ..default()
        });
    });
}

fn update_enemies(
    mut query: Query<(&Transform, &mut Velocity, &mut Enemy)>,
    time: Res<Time>,
    window: Query<&Window>,
) {
    const SPEED: f32 = 1.;
    const GOAL_MARGIN: f32 = 1.;

    let window = window.single();
    let (left, right, up, down) = (
        -window.width() / 2.,
        window.width() / 2.,
        -window.height() / 2.,
        window.height() / 2.,
    );
    for (
        Transform {
            translation: trans, ..
        },
        mut vel,
        mut enemy,
    ) in query.iter_mut()
    {
        if enemy.change_goal.tick(time.delta()).just_finished() {
            let mut rng = rand::thread_rng();
            enemy.goal = Vec2 {
                x: rng.gen_range(left..right),
                y: rng.gen_range(up..down),
            };
        }
        let pos = Vec2 {
            x: trans.x,
            y: trans.y,
        };

        if GOAL_MARGIN < (enemy.goal - pos).length() {
            // vel.0 += Vec2::ONE;

            let dir = enemy.goal - pos;
            vel.0 += dir.normalize() * SPEED;
        }
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &mut Velocity)>) {
    const MIN_VEL: f32 = 0.1;
    const DRAG_C: f32 = 0.5;

    for (mut trans, mut vel) in query.iter_mut() {
        trans.translation.x += vel.0.x;
        trans.translation.y += vel.0.y;
        vel.0 = if MIN_VEL < vel.0.length() {
            vel.0 * DRAG_C
        } else {
            Vec2::ZERO
        }
    }
}

fn pull_inside_bounds(mut query: Query<(&Transform, &mut Velocity)>, query_window: Query<&Window>) {
    const PULL_VEL: f32 = 6.;
    let window = query_window.single();

    let (left, right, up, down) = (
        -window.width() / 2.,
        window.width() / 2.,
        -window.height() / 2.,
        window.height() / 2.,
    );

    for (
        Transform {
            translation: Vec3 { x, y, .. },
            ..
        },
        mut vel,
    ) in query.iter_mut()
    {
        if right < *x {
            vel.0.x -= PULL_VEL;
        }
        if *x < left {
            vel.0.x += PULL_VEL;
        }
        if down < *y {
            vel.0.y -= PULL_VEL;
        }
        if *y < up {
            vel.0.y += PULL_VEL;
        }
    }
}

fn keyboard_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Player)>,
    time: Res<Time>,
) {
    const SPEED: f32 = 5.;
    const DASH_C: f32 = 4.;

    let (mut vel, mut player) = query.single_mut();

    player.dash_cooldown.tick(time.delta());
    player.dash_duration.tick(time.delta());

    if keyboard_input.pressed(KeyCode::Space) {
        if player.dash_cooldown.finished() {
            player.dash_duration.reset();
        }
        player.dash_cooldown.reset();
    }

    let speed = if player.dash_duration.finished() {
        SPEED
    } else {
        SPEED * DASH_C
    };

    let (up, down, left, right) = (
        keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S),
        keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W),
        keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A),
        keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D),
    );

    vel.0 += match (up, down, left, right) {
        (true, false, false, false) => Vec2::from_angle(3. * PI / 2.),
        (false, true, false, false) => Vec2::from_angle(PI / 2.),
        (false, false, true, false) => Vec2::from_angle(PI),
        (false, false, false, true) => Vec2::from_angle(0.),
        (false, true, true, false) => Vec2::from_angle(3. * PI / 4.),
        (false, true, false, true) => Vec2::from_angle(PI / 4.),
        (true, false, true, false) => Vec2::from_angle(5. * PI / 4.),
        (true, false, false, true) => Vec2::from_angle(7. * PI / 4.),
        _ => Vec2::ZERO,
    } * speed;
}
