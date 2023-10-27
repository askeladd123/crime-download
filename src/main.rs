use bevy::{
    log::LogPlugin, pbr::extract_camera_previous_view_projection, prelude::*,
    sprite::MaterialMesh2dBundle,
};
use bevy_svg::prelude::*;
use bevy_text_popup::{
    TextPopupButton, TextPopupEvent, TextPopupLocation, TextPopupPlugin, TextPopupTimeout,
};
use rand::Rng;

mod popups;
use popups::*;
const PI: f32 = std::f32::consts::PI;
const BBOX_SIZE: Vec2 = Vec2 { x: 50., y: 50. };

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Msaa::Sample4)
        .add_state::<State>()
        .add_event::<AddComputerAndUsb>()
        .add_event::<PopupCommand>()
        .add_event::<AddEnemy>()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "game jam 2".to_string(),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(LogPlugin {
                    filter: "warn,stealy=trace,wgpu_hal::vulkan::instance=off".into(),
                    ..default()
                }),
            TextPopupPlugin,
        ))
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
                add_computer_and_usb,
                add_enemy,
                pick_up_usb,
                insert_usb,
                update_progress_and_spawn_popups,
                update_score,
                handle_popup_events,
            )
                .run_if(in_state(State::InGame)),
        )
        .add_systems(OnEnter(State::GameOver), game_over)
        .run();
}

// Resources, Components and Events

#[derive(Resource)]
struct Common {
    enemy_speed: f32,
    score: u32,
}

impl Default for Common {
    fn default() -> Self {
        Self {
            enemy_speed: 1.,
            score: 0,
        }
    }
}

#[derive(Component)]
struct ProgressBar {
    timer: Timer,
    timer_popups: Timer,
    progress: u32,
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.25, TimerMode::Repeating),
            timer_popups: Timer::from_seconds(3., TimerMode::Repeating),
            progress: 0,
        }
    }
}

#[derive(Bundle)]
struct ProgressBarBundle {
    text: Text2dBundle,
    tag: ProgressBar,
}

impl Default for ProgressBarBundle {
    fn default() -> Self {
        Self {
            text: Text2dBundle {
                text: Text::from_section(
                    "downloading...",
                    TextStyle {
                        font_size: 30.,
                        ..default()
                    },
                ),
                transform: Transform::default(),
                ..default()
            },
            tag: ProgressBar::default(),
        }
    }
}

#[derive(Resource)]
struct AssetPool {
    computer: Handle<Image>,
    usb: Handle<Image>,
    police: Handle<Svg>,
}

#[derive(Event)]
struct AddComputerAndUsb;

#[derive(Event)]
struct AddEnemy;

#[derive(Component)]
struct Computer;

#[derive(Component)]
struct Usb;

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
    let player = query_player.single();
    for enemy in query_enemies.iter() {
        if bevy::sprite::collide_aabb::collide(
            player.translation,
            BBOX_SIZE,
            enemy.translation,
            BBOX_SIZE,
        )
        .is_some()
        {
            next_state.set(State::GameOver);
            return;
        }
    }
}

#[derive(Component)]
struct Score;

fn setup(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query_window: Query<&Window>,
    mut writer: EventWriter<AddComputerAndUsb>,
    mut w_enemy: EventWriter<AddEnemy>,
) {
    writer.send(AddComputerAndUsb);
    writer.send(AddComputerAndUsb);
    // writer.send(AddComputerAndUsb);

    const STARTING_ENEMIES: u32 = 2;

    let mut rng = rand::thread_rng();
    let window = query_window.single();

    cmd.spawn(Camera2dBundle::default());

    // cmd.spawn(Svg2dBundle {
    //     svg: asset_server.load("computer-1.svg"),
    //     transform: Transform {
    //         scale: Vec3 {
    //             x: 1.5,
    //             y: 1.5,
    //             ..default()
    //         },
    //         translation: Vec3 {
    //             x: 0.,
    //             y: 0.,
    //             ..default()
    //         },
    //         ..default()
    //     },
    //     origin: Origin::Center,
    //     ..default()
    // });

    // images stolen counter in top left
    // cmd.spawn(Text2dBundle {
    //     text: Text::from_section("translation", TextStyle::default()),
    //     ..default()
    // });

    // let police_handle = asset_server.load("police.svg");
    for i in 0..STARTING_ENEMIES {
        w_enemy.send(AddEnemy);
        // cmd.spawn((
        //     Enemy {
        //         goal: random_window_position(&window, &mut rng),
        //         ..default()
        //     },
        //     TransformBundle {
        //         local: Transform {
        //             translation: Vec3 {
        //                 x: -window.width() / 2.0,
        //                 y: -window.height() / 2.0,
        //                 z: i as f32,
        //             },
        //             ..default()
        //         },
        //         ..default()
        //     },
        //     Velocity::default(),
        //     VisibilityBundle::default(),
        // ))
        // .with_children(|cmd| {
        //     // cmd.spawn(MaterialMesh2dBundle {
        //     //     mesh: meshes
        //     //         .add(shape::Quad::new(Vec2::new(50., 50.)).into())
        //     //         .into(),
        //     //     material: materials.add(ColorMaterial::from(Color::LIME_GREEN)),
        //     //     ..default()
        //     // });
        //     cmd.spawn(Svg2dBundle {
        //         svg: police_handle.clone(),
        //         transform: Transform {
        //             scale: Vec3 {
        //                 x: 1.5,
        //                 y: 1.5,
        //                 ..default()
        //             },
        //             translation: Vec3 {
        //                 x: -25.,
        //                 y: 25.,
        //                 ..default()
        //             },
        //             ..default()
        //         },
        //         ..default()
        //     });
        // });
    }
    cmd.insert_resource(AssetPool {
        computer: asset_server.load("computer.png"),
        usb: asset_server.load("usb.png"),
        police: asset_server.load("police.svg"),
    });

    cmd.insert_resource(Common::default());

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
        Score,
        Text2dBundle {
            text: Text::from_section(
                "no score",
                TextStyle {
                    font_size: 60.,
                    ..default()
                },
            ),
            transform: Transform {
                translation: Vec3 {
                    y: 300.,
                    z: 100.,
                    ..default()
                },
                ..default()
            },
            ..default()
        },
    ));

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
                // translation: Vec3 {
                //     x: -25.,
                //     y: 25.,
                //     z: 10.,
                //     ..Default::default()
                // },
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
            origin: Origin::Center,
            // origin: Origin::TopLeft,
            ..default()
        });
    });
}

fn update_enemies(
    mut query: Query<(&Transform, &mut Velocity, &mut Enemy)>,
    time: Res<Time>,
    window: Query<&Window>,
    common: ResMut<Common>,
) {
    let speed: f32 = common.enemy_speed;
    const GOAL_MARGIN: f32 = 6.;

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
            vel.0 += dir.normalize() * speed;
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
    const SPEED: f32 = 4.;
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

fn random_window_position(window: &Window, rng: &mut rand::rngs::ThreadRng) -> Vec2 {
    let (left, right, up, down) = (
        -window.width() / 2.,
        window.width() / 2.,
        -window.height() / 2.,
        window.height() / 2.,
    );

    Vec2 {
        x: rng.gen_range(left..right),
        y: rng.gen_range(up..down),
    }
}

fn add_computer_and_usb(
    mut cmd: Commands,
    mut reader: EventReader<AddComputerAndUsb>,
    query_window: Query<&Window>,
    asset_pool: Res<AssetPool>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for _ in reader.iter() {
        let mut rng = rand::thread_rng();
        let window = query_window.single();
        cmd.spawn((
            Computer,
            SpriteBundle {
                texture: asset_pool.computer.clone(),
                transform: Transform {
                    translation: random_window_position(&window, &mut rng).extend(0.),
                    scale: Vec3 {
                        x: 0.2,
                        y: 0.2,
                        ..default()
                    },
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|cmd| {
            cmd.spawn(MaterialMesh2dBundle {
                mesh: meshes
                    .add(shape::Quad::new(Vec2::new(50., 50.)).into())
                    .into(),
                material: materials.add(ColorMaterial::from(Color::LIME_GREEN)),
                ..default()
            });
            // cmd.spawn(Svg2dBundle {
            //     svg: asset_pool.computer.clone(),
            //     transform: Transform {
            //         scale: Vec3 {
            //             x: 0.5,
            //             y: 0.5,
            //             ..default()
            //         },
            //         translation: Vec3 {
            //             // x: -25.,
            //             // y: 25.,
            //             ..default()
            //         },
            //         ..default()
            //     },
            //     origin: Origin::Center,
            //     ..default()
            // });
        });

        cmd.spawn((
            Usb,
            SpriteBundle {
                texture: asset_pool.usb.clone(),
                transform: Transform {
                    translation: random_window_position(&window, &mut rng).extend(0.),
                    scale: Vec3 {
                        x: 0.15,
                        y: 0.15,
                        ..default()
                    },
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|cmd| {
            cmd.spawn(MaterialMesh2dBundle {
                mesh: meshes
                    .add(shape::Quad::new(Vec2::new(50., 50.)).into())
                    .into(),
                material: materials.add(ColorMaterial::from(Color::LIME_GREEN)),
                ..default()
            });
            // cmd.spawn(Svg2dBundle {
            //     svg: asset_pool.usb.clone(),
            //     transform: Transform {
            //         scale: Vec3 {
            //             x: 1.5,
            //             y: 1.5,
            //             ..default()
            //         },
            //         translation: Vec3 {
            //             x: -25.,
            //             y: 25.,
            //             ..default()
            //         },
            //         ..default()
            //     },
            //     ..default()
            // });
        });
    }
}

fn pick_up_usb(
    mut q_usb: Query<(&mut Transform, Entity), (With<Usb>, Without<Player>)>,
    q_player: Query<(&Transform, Entity), (With<Player>, Without<Usb>)>,
    mut cmd: Commands,
) {
    let (
        Transform {
            translation: player_trans,
            ..
        },
        player,
    ) = q_player.single();

    for (mut usb_transform, usb) in q_usb.iter_mut() {
        if bevy::sprite::collide_aabb::collide(
            *player_trans,
            BBOX_SIZE,
            usb_transform.translation,
            BBOX_SIZE,
        )
        .is_some()
        {
            let mut player = cmd.get_entity(player).unwrap();
            usb_transform.translation.x = 30.;
            usb_transform.translation.y = 30.;
            player.push_children(&[usb]);
        }
    }
}

fn insert_usb(
    q_usb: Query<(&GlobalTransform, Entity), (With<Usb>, Without<Computer>)>,
    q_computer: Query<(&Transform, Entity), (With<Computer>, Without<Usb>)>,
    mut cmd: Commands,
) {
    for (usb_transform, usb_entity) in q_usb.iter() {
        // info!("tick usb");
        for (computer_transform, computer_entity) in q_computer.iter() {
            // dbg!(computer_transform.translation, usb_transform.translation());
            // info!("tick computer");

            if bevy::sprite::collide_aabb::collide(
                usb_transform.translation(),
                BBOX_SIZE,
                computer_transform.translation,
                BBOX_SIZE,
            )
            .is_some()
            {
                cmd.entity(usb_entity).despawn_recursive();

                cmd.spawn(ProgressBarBundle {
                    text: Text2dBundle {
                        text: Text::from_section(
                            "downloading...",
                            TextStyle {
                                font_size: 30.,
                                ..default()
                            },
                        ),
                        transform: Transform::from_translation(
                            computer_transform.translation + Vec3::new(0., 50., 10.),
                        ),
                        ..default()
                    },
                    ..default()
                });
                return;
            }
        }
    }
}

fn update_progress_and_spawn_popups(
    mut q: Query<Option<(Entity, &mut ProgressBar, &mut Text)>>,
    time: Res<Time>,
    mut writer: EventWriter<TextPopupEvent>,
    mut common: ResMut<Common>,
    mut cmd: Commands,
) {
    for (entity, mut p, mut text) in q.iter_mut().flatten() {
        if p.timer.tick(time.delta()).just_finished() {
            if 100 == p.progress {
                common.score += 1;
                // cmd.entity(entity).despawn_recursive();
            } else {
                p.progress += 1;
            }
        }

        if p.timer_popups.tick(time.delta()).just_finished() && rand::random() {
            insert_random_popup(&mut writer);
        }

        text.sections.first_mut().unwrap().value = format!("download {}", p.progress);
    }
}

fn add_enemy(
    mut cmd: Commands,
    mut r: EventReader<AddEnemy>,
    q_window: Query<&Window>,
    asset_pool: Res<AssetPool>,
) {
    let window = q_window.single();
    let mut rng = rand::thread_rng();

    for _ in r.iter() {
        cmd.spawn((
            Enemy {
                goal: random_window_position(&window, &mut rng),
                ..default()
            },
            TransformBundle {
                local: Transform {
                    translation: Vec3 {
                        x: -window.width() / 2.0,
                        y: -window.height() / 2.0,
                        z: 0.,
                    },
                    ..default()
                },
                ..default()
            },
            Velocity::default(),
            VisibilityBundle::default(),
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
                svg: asset_pool.police.clone(),
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
}

fn handle_popup_events(
    cmd: Commands,
    mut reader: EventReader<PopupCommand>,
    mut q_enemy: Query<&mut Enemy>,
    mut q_player: Query<&Transform, With<Player>>,
    mut w_enemy: EventWriter<AddEnemy>,
    mut common: ResMut<Common>,
) {
    for event in reader.iter() {
        match event {
            PopupCommand::AddCop => {
                w_enemy.send(AddEnemy);
            }
            PopupCommand::CopsTargetPlayer => {
                let player_trans = q_player.single();
                for mut enemy in q_enemy.iter_mut() {
                    enemy.goal = player_trans.translation.truncate();
                }
            }
            PopupCommand::IncreaseCopSpeed => {
                common.enemy_speed += 1.;
            }
        }
    }
}

fn update_score(common: Res<Common>, mut q: Query<&mut Text, With<Score>>) {
    q.single_mut().sections.first_mut().unwrap().value =
        format!("crime downloaded: {}", common.score);
}
