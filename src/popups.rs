use bevy::prelude::*;
use bevy_text_popup::{
    TextPopupButton, TextPopupEvent, TextPopupLocation,
};
use rand::seq::IteratorRandom;

#[derive(Event)]
pub enum PopupCommand {
    AddCop,
    CopsTargetPlayer,
    IncreaseCopSpeed,
}

pub fn insert_random_popup(writer: &mut EventWriter<TextPopupEvent>) {

    let random_location = ||[    
        TextPopupLocation::TopLeft,
        TextPopupLocation::Top,
        TextPopupLocation::TopRight,
        TextPopupLocation::Left,
        TextPopupLocation::Center,
        TextPopupLocation::Right,
        TextPopupLocation::BottomLeft,
        TextPopupLocation::Bottom,
        TextPopupLocation::BottomRight,
    ].into_iter().choose(&mut rand::thread_rng()).unwrap();
    
    let list: Vec<_> = [TextPopupEvent {
        content: "Do you want to allow us to enhance the experience by increasing your safety and security?".to_string(),
        location: random_location(), 
        font_size: 12.,
        confirm_button: Some(TextPopupButton {
            text: "yes".to_string(),
            action: |commands, root_entity| {
                commands.add(|world: &mut World| {
                    world.send_event(PopupCommand::AddCop);
                });
                commands.entity(root_entity).despawn_recursive();
            },
            ..Default::default()
        }),
        dismiss_button: Some(TextPopupButton {
            text: "no".to_string(),
            background_color: Color::RED,
            ..Default::default()
        }),
        ..default()
    }, TextPopupEvent {
        content: "Do you want us to keep the experience as it is, and not change your safety and security?".to_string(),
        location: random_location(), 
        font_size: 12.,
        confirm_button: Some(TextPopupButton {
            text: "yes".to_string(),
            ..Default::default()
        }),
        dismiss_button: Some(TextPopupButton {
            text: "no".to_string(),
            background_color: Color::RED, 
            action: |commands, root_entity| {
                            commands.add(|world: &mut World| {
                                world.send_event(PopupCommand::AddCop);
                            });
                            commands.entity(root_entity).despawn_recursive();
                        },
            ..Default::default()
        }),
        ..default()
    }, TextPopupEvent {
        content: "Allow access to player location for enhanced experience?".to_string(),
        location: random_location(), 
        font_size: 12.,
        confirm_button: Some(TextPopupButton {
            text: "allow".to_string(),
            action: |commands, root_entity| {
                commands.add(|world: &mut World| {
                    world.send_event(PopupCommand::CopsTargetPlayer);
                });
                commands.entity(root_entity).despawn_recursive();
            },
            ..Default::default()
        }),
        dismiss_button: Some(TextPopupButton {
            text: "deny".to_string(),
            background_color: Color::RED,
            ..Default::default()
        }),
        ..default()
    }, TextPopupEvent {
        content: "Keep access to player location the same as before?".to_string(),
        location: random_location(), 
        font_size: 12.,
        confirm_button: Some(TextPopupButton {
            text: "allow".to_string(),
            ..Default::default()
        }),
        dismiss_button: Some(TextPopupButton {
            text: "deny".to_string(),
            action: |commands, root_entity| {
                commands.add(|world: &mut World| {
                    world.send_event(PopupCommand::CopsTargetPlayer);
                });
                commands.entity(root_entity).despawn_recursive();
            },
            background_color: Color::RED,
            ..Default::default()
        }),
        ..default()
    }, TextPopupEvent {
        content: "Increase performance by allowing police speed optimization?".to_string(),
        location: random_location(), 
        font_size: 12.,
        confirm_button: Some(TextPopupButton {
            text: "allow".to_string(),
            action: |commands, root_entity| {
                commands.add(|world: &mut World| {
                    world.send_event(PopupCommand::IncreaseCopSpeed);
                });
                commands.entity(root_entity).despawn_recursive();
            },
            ..Default::default()
        }),
        dismiss_button: Some(TextPopupButton {
            text: "deny".to_string(),
            background_color: Color::RED,
            ..Default::default()
        }),
        ..default()
    }, TextPopupEvent {
        content: "Performance of police speed high. Do nothing about it?".to_string(),
        location: random_location(), 
        font_size: 12.,
        confirm_button: Some(TextPopupButton {
            text: "allow".to_string(),
            ..Default::default()
        }),
        dismiss_button: Some(TextPopupButton {
            text: "deny".to_string(),
            action: |commands, root_entity| {
                commands.add(|world: &mut World| {
                    world.send_event(PopupCommand::IncreaseCopSpeed);
                });
                commands.entity(root_entity).despawn_recursive();
            },
            background_color: Color::RED,
            ..Default::default()
        }),
        ..default()
    }]
    .into();

    let mut rng = rand::thread_rng();
    writer.send(list.into_iter().choose(&mut rng).unwrap());
}
