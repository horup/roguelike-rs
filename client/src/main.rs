//! Renders a 2D scene containing a single, moving sprite.

use std::sync::Mutex;
use bevy::prelude::*;
use shared::Message;

#[derive(Resource)]
pub struct Client {
    pub client:Mutex<netcode::client::Client<Message>>
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Client {
            client:Mutex::new(Default::default())
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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, client:ResMut<Client>) {
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

/// The sprite is animated by changing its translation depending on the time that has passed since
/// the last frame.
fn update(time: Res<Time>, mut sprite_position: Query<(&mut Direction, &mut Transform)>, client:ResMut<Client>) {
    let mut client = client.client.lock().unwrap();
    for e in client.poll() {
        match e {
            netcode::client::Event::Connecting => {

            },
            netcode::client::Event::Connected => {

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