use std::sync::Mutex;
use bevy::{prelude::*, render::{settings::{Backends, WgpuSettings}, RenderPlugin}};
use shared::Message;
use uuid::Uuid;

#[derive(Component)]
pub struct CenterText;

#[derive(Resource)]
pub struct Player {
    pub id:Uuid,
    pub name:String,
    pub client:Mutex<netcode::client::Client<Message>>
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            render_creation:bevy::render::settings::RenderCreation::Automatic(WgpuSettings {
                backends:Some(Backends::VULKAN),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .insert_resource(Player {
            client:Mutex::new(Default::default()),
            id:Uuid::new_v4(),
            name:"Player".to_owned()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

#[derive(Component)]
enum Direction {
    Up,
    Down,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, client:ResMut<Player>) {
    client.client.lock().unwrap().connect("ws://localhost:8080");
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("branding/icon.png"),
            transform: Transform::from_xyz(100., 0., 0.),
            ..default()
        },
        Direction::Up,
    ));

    commands.spawn(Text2dBundle {
        text:Text::from_section("", TextStyle {
            font_size:64.0,
            ..Default::default()
        }),
        ..Default::default()
    }).insert(CenterText);
}

fn update(time: Res<Time>, mut sprite_position: Query<(&mut Direction, &mut Transform)>, player:ResMut<Player>, mut center_text:Query<&mut Text, With<CenterText>>) {
    let mut client = player.client.lock().unwrap();
    for e in client.poll() {
        match e {
            netcode::client::Event::Connecting => {
                center_text.single_mut().sections[0] = "Connecting...".into();
            },
            netcode::client::Event::Connected => {
                client.send(Message::JoinAsPlayer { id: player.id, name: player.name.clone() });
                center_text.single_mut().sections[0] = "".into();
            },
            netcode::client::Event::Disconnected => {
                center_text.single_mut().sections[0] = "Lost connection to server!".into();
            },
            netcode::client::Event::Message(_) => {
                
            },
        }
    }
    for (mut logo, mut transform) in &mut sprite_position {
        match *logo {
            Direction::Up => transform.translation.y += 150. * time.delta_seconds(),
            Direction::Down => transform.translation.y -= 150. * time.delta_seconds(),
        }

        if transform.translation.y > 200. {
            *logo = Direction::Down;
        } else if transform.translation.y < -200. {
            *logo = Direction::Up;
        }
    }
}