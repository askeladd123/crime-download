#![allow(unused, clippy::type_complexity)]
use bevy::{log::LogPlugin, prelude::*, sprite::MaterialMesh2dBundle};
use bevy_svg::prelude::*;
use bevy_text_popup::{TextPopupEvent, TextPopupPlugin};
use rand::Rng;

mod popups;
use popups::*;
const PI: f32 = std::f32::consts::PI;
const BBOX_SIZE: Vec2 = Vec2 { x: 50., y: 50. };
const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Msaa::Sample4)
        .add_state::<State>()
        .add_event::<Items>()
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
                    filter: "warn,crime-download=trace,wgpu_hal::vulkan::instance=off".into(),
                    ..default()
                }),
            TextPopupPlugin,
        ))
        .add_plugins(bevy_svg::prelude::SvgPlugin)
        .add_systems(Startup, setup)
        .add_systems(OnEnter(State::InGame), spawn)
        .add_systems(
            Update,
            (
                update_enemies,
                keyboard_input,
                apply_velocity,
                pull_inside_bounds,
                check_game_over,
                handle_item_events,
                add_enemy,
                pick_up_usb,
                insert_usb,
                update_progress_and_spawn_popups,
                update_score,
                handle_popup_events,
            )
                .run_if(in_state(State::InGame)),
        )
        .add_systems(OnEnter(State::GameOver), game_over_spawn)
        .add_systems(Update, (check_restart).run_if(in_state(State::GameOver)))
        .add_systems(OnExit(State::GameOver), game_over_despawn)
        .add_systems(
            OnTransition {
                from: State::GameOver,
                to: State::InGame,
            },
            despawn,
        )
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
            enemy_speed: 60.,
            score: 0,
        }
    }
}

#[derive(Component)]
struct GameOver;

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

#[derive(Resource, Clone)]
struct AssetPool {
    pc: Handle<Image>,
    usb: Handle<Image>,
    police: Handle<Svg>,
    thief: Handle<Svg>,
}

#[derive(Event)]
enum Items {
    AddPcUsb,
}

#[derive(Event)]
struct AddEnemy;

#[derive(Component)]
struct Pc;

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
    has_usb: bool,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            dash_cooldown: Timer::from_seconds(0.8, TimerMode::Once),
            dash_duration: Timer::from_seconds(0.1, TimerMode::Once),
            has_usb: false,
        }
    }
}

#[derive(Component)]
struct Enemy {
    change_goal: Timer,
    goal: Vec2,
}

impl Default for Enemy {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            change_goal: Timer::from_seconds(rng.gen_range(1.0..10.0), TimerMode::Repeating),
            goal: Default::default(),
        }
    }
}

#[derive(Component, Default, Copy, Clone, Debug)]
struct Velocity(Vec2);

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

fn setup(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn(Camera2dBundle::default());

    let asset_pool = AssetPool {
        pc: asset_server.load("computer.png"),
        usb: asset_server.load("usb.png"),
        police: asset_server.load("police.svg"),
        thief: asset_server.load("thief.svg"),
    };
    cmd.insert_resource(asset_pool.clone());

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
}

fn despawn(
    mut cmd: Commands,
    q: Query<Entity, Or<(With<Pc>, With<Usb>, With<Player>, With<Enemy>)>>,
) {
    cmd.remove_resource::<Common>();

    for entity in q.iter() {
        cmd.entity(entity).despawn_recursive();
    }
}

fn spawn(
    mut cmd: Commands,
    mut w_items: EventWriter<Items>,
    mut w_enemy: EventWriter<AddEnemy>,
    asset_pool: Res<AssetPool>,
) {
    cmd.insert_resource(Common::default());

    w_items.send(Items::AddPcUsb);
    w_items.send(Items::AddPcUsb);

    const STARTING_ENEMIES: u32 = 2;
    for _ in 0..STARTING_ENEMIES {
        w_enemy.send(AddEnemy);
    }

    cmd.spawn((
        Player::default(),
        TransformBundle::default(),
        Velocity::default(),
        ComputedVisibility::default(),
        Visibility::Visible,
    ))
    .with_children(|cmd| {
        cmd.spawn(Svg2dBundle {
            svg: asset_pool.thief.clone(),
            transform: Transform {
                translation: Vec3 {
                    x: -25.,
                    y: 25.,
                    z: 10.,
                },
                scale: Vec3 {
                    x: 0.1,
                    y: 0.1,
                    ..default()
                },
                ..default()
            },
            origin: Origin::Center,
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
            let dir = enemy.goal - pos;
            vel.0 += dir.normalize() * speed;
        }
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &mut Velocity)>, time: Res<Time>) {
    const MIN_VEL: f32 = 0.1;
    const DRAG_C: f32 = 0.5;

    for (mut trans, mut vel) in query.iter_mut() {
        trans.translation.x += vel.0.x * time.delta_seconds();
        trans.translation.y += vel.0.y * time.delta_seconds();
        vel.0 = if MIN_VEL < vel.0.length() {
            vel.0 * DRAG_C
        } else {
            Vec2::ZERO
        }
    }
}

fn pull_inside_bounds(mut query: Query<(&Transform, &mut Velocity)>, query_window: Query<&Window>) {
    const PULL_VEL: f32 = 360.;
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
    const SPEED: f32 = 240.;
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

fn handle_item_events(
    mut cmd: Commands,
    mut reader: EventReader<Items>,
    query_window: Query<&Window>,
    asset_pool: Res<AssetPool>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for event in reader.iter() {
        match event {
            Items::AddPcUsb => {
                let mut rng = rand::thread_rng();
                let window = query_window.single();
                cmd.spawn((
                    Pc,
                    SpriteBundle {
                        texture: asset_pool.pc.clone(),
                        transform: Transform {
                            translation: random_window_position(window, &mut rng).extend(0.),
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
                });

                cmd.spawn((
                    Usb,
                    SpriteBundle {
                        texture: asset_pool.usb.clone(),
                        transform: Transform {
                            translation: random_window_position(window, &mut rng).extend(0.),
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
                });
            }
        }
    }
}

fn pick_up_usb(
    mut q_usb: Query<(&mut Transform, Entity), (With<Usb>, Without<Player>)>,
    q_player: Query<(Entity, &Transform, &Children), With<Player>>,
    mut cmd: Commands,
) {
    let (
        player_entity,
        Transform {
            translation: player_trans,
            ..
        },
        player_children,
    ) = q_player.single();

    let mut has_usb = player_children
        .iter()
        .any(|entity| q_usb.get(*entity).is_ok());

    for (mut usb_transform, usb) in q_usb.iter_mut() {
        if !has_usb
            && bevy::sprite::collide_aabb::collide(
                *player_trans,
                BBOX_SIZE,
                usb_transform.translation,
                BBOX_SIZE,
            )
            .is_some()
        {
            let mut player = cmd.get_entity(player_entity).unwrap();
            usb_transform.translation.x = 30.;
            usb_transform.translation.y = 30.;
            player.push_children(&[usb]);
        }
    }
}

fn insert_usb(
    q_usb: Query<(&GlobalTransform, Entity), (With<Usb>, Without<Pc>)>,
    q_pc: Query<(&Transform, Entity), (With<Pc>, Without<Usb>)>,
    mut cmd: Commands,
) {
    for (usb_transform, usb_entity) in q_usb.iter() {
        for (pc_transform, _pc_entity) in q_pc.iter() {
            if bevy::sprite::collide_aabb::collide(
                usb_transform.translation(),
                BBOX_SIZE,
                pc_transform.translation,
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
                            pc_transform.translation + Vec3::new(0., 50., 10.),
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
) {
    for (_entity, mut p, mut text) in q.iter_mut().flatten() {
        if p.timer.tick(time.delta()).just_finished() {
            if 100 == p.progress {
                common.score += 1;
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
                goal: random_window_position(window, &mut rng),
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
    _cmd: Commands,
    q_player: Query<&Transform, With<Player>>,
    mut q_enemy: Query<&mut Enemy>,
    mut w_enemy: EventWriter<AddEnemy>,
    mut common: ResMut<Common>,
    mut reader: EventReader<PopupCommand>,
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

fn game_over_despawn(mut cmd: Commands, q: Query<Entity, With<GameOver>>) {
    for entity in q.iter() {
        cmd.entity(entity).despawn_recursive();
    }
}

fn game_over_spawn(mut cmd: Commands) {
    cmd.spawn(GameOver).insert(Text2dBundle {
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

    cmd.spawn(GameOver)
        .insert(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(133.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(150.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    border_color: BorderColor(Color::BLACK),
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "restart",
                        TextStyle {
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..Default::default()
                        },
                    ));
                });
        });
}

fn check_restart(
    keyboard_input: Res<Input<KeyCode>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<State>>,
) {
    if keyboard_input.pressed(KeyCode::Space)
        || keyboard_input.pressed(KeyCode::Return)
        || keyboard_input.pressed(KeyCode::R)
    {
        next_state.set(State::InGame);
        return;
    }

    for (interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::RED;
                next_state.set(State::InGame);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}
