use std::sync::Mutex;
use bevy::prelude::*;
use shared::Message;
use uuid::Uuid;

#[derive(Resource)]
pub struct Player {
    pub id:Uuid,
    pub name:String,
    pub client:Mutex<netcode::client::Client<Message>>
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
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
}

fn update(time: Res<Time>, mut sprite_position: Query<(&mut Direction, &mut Transform)>, player:ResMut<Player>) {
    let mut client = player.client.lock().unwrap();
    for e in client.poll() {
        match e {
            netcode::client::Event::Connecting => {
            },
            netcode::client::Event::Connected => {
                client.send(Message::JoinAsPlayer { id: player.id, name: player.name.clone() });
            },
            netcode::client::Event::Disconnected => {

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