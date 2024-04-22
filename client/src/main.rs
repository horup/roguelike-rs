use bevy::{
    prelude::*,
    render::{
        settings::{Backends, WgpuSettings},
        RenderPlugin,
    },
};
use shared::Message;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Component)]
pub struct CenterText;

#[derive(Resource)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub client: Mutex<netcode::client::Client<Message>>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            render_creation: bevy::render::settings::RenderCreation::Automatic(WgpuSettings {
                backends: Some(Backends::VULKAN),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .insert_resource(Player {
            client: Mutex::new(Default::default()),
            id: Uuid::new_v4(),
            name: "Player".to_owned(),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    client: ResMut<Player>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    client.client.lock().unwrap().connect("ws://localhost:8080");

    commands.spawn(Camera2dBundle::default());
    commands.spawn(Camera3dBundle {
        camera:Camera {
            order:-1,
            ..Default::default()
        },
        transform: Transform::from_xyz(15.0, 5.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    commands
        .spawn(Text2dBundle {
            text: Text::from_section(
                "",
                TextStyle {
                    font_size: 16.0,
                    ..Default::default()
                },
            ),
            ..Default::default()
        })
        .insert(CenterText);

    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(20., 20.)),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3)),
        ..default()
    });
}

fn update(
    player: ResMut<Player>,
    mut center_text: Query<&mut Text, With<CenterText>>,
) {
    let mut client = player.client.lock().unwrap();
    for e in client.poll() {
        match e {
            netcode::client::Event::Connecting => {
                center_text.single_mut().sections[0] = "Connecting...".into();
            }
            netcode::client::Event::Connected => {
                client.send(Message::JoinAsPlayer {
                    id: player.id,
                    name: player.name.clone(),
                });
                center_text.single_mut().sections[0] = "".into();
            }
            netcode::client::Event::Disconnected => {
                center_text.single_mut().sections[0] = "Lost connection to server!".into();
            }
            netcode::client::Event::Message(_) => {}
        }
    }
}
