use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_svg::prelude::*;

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
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
                keyboard_input,
                // apply_velocity
            ),
        )
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Component, Default, Copy, Clone, Debug)]
struct Velocity(Vec2);

fn setup(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    cmd.spawn(Camera2dBundle::default());
    // cmd.spawn(Text2dBundle {
    //     text: Text::from_section("translation", TextStyle::default()),
    //     ..default()
    // });

    // for _ in 0..2 {
    //     cmd.spawn((Enemy, TransformBundle::default(), Velocity::default()))
    //         .with_children(|cmd| {
    //             cmd.spawn(Svg2dBundle {
    //                 svg: asset_server.load("police.svg"),
    //                 transform: Transform {
    //                     scale: Vec3 {
    //                         x: 0.1,
    //                         y: 0.1,
    //                         ..default()
    //                     },
    //                     ..default()
    //                 },
    //                 ..default()
    //             });
    //         });
    // }

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
        Player,
        TransformBundle::default(),
        // Velocity::default(),
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

// fn update_enemies(mut query: Query<&mut Transform, With<Enemy>>) {
//     for mut trans in query.iter_mut() {}
// }

// fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>) {
//     for (mut trans, vel) in query.iter_mut() {
//         trans.translation.x += vel.0.x;
//         trans.translation.y += vel.0.y;
//     }
// }

fn keyboard_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    const SPEED: f32 = 5.;
    const DASH_C: f32 = 3.;

    let mut trans = query.single_mut();

    if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
        trans.translation.x -= SPEED;
    }
    if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
        trans.translation.x += SPEED;
    }
    if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
        trans.translation.y += SPEED;
    }
    if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S) {
        trans.translation.y -= SPEED;
    }
}
