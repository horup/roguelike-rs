use bevy::{
    prelude::*,
    render::{
        mesh::primitives, settings::{Backends, WgpuSettings}, RenderPlugin
    },
};
use shared::Message;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Component)]
pub struct Ground;

#[derive(Component)]
pub struct CenterText;

#[derive(Resource, Default)]
pub struct CommonAssets {
    pub block_mesh:Handle<Mesh>
}

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
        .insert_resource(CommonAssets::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (cursor, update).chain())
        .run();
}

fn setup(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    client: ResMut<Player>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut common_assets:ResMut<CommonAssets>
) {
    common_assets.block_mesh = meshes.add(Cuboid::new(1., 1., 1.));
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
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(64., 64.)),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3)),
        ..default()
    }).insert(Ground);
}

fn cursor(
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    ground_query: Query<&GlobalTransform, With<Ground>>,
    windows: Query<&Window>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = camera_query.single();
    let ground = ground_query.single();

    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };

    // Calculate a ray pointing from the camera into the world based on the cursor's position.
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // Calculate if and where the ray is hitting the ground plane.
    let Some(distance) = ray.intersect_plane(ground.translation(), Plane3d::new(ground.up()))
    else {
        return;
    };
    let point = ray.get_point(distance);

    // Draw a circle just above the ground plane at that position.
    gizmos.circle(
        point + ground.up() * 0.01,
        Direction3d::new_unchecked(ground.up()), // Up vector is already normalized.
        0.2,
        Color::RED,
    );
}

fn update(
    player: ResMut<Player>,
    mut center_text: Query<&mut Text, With<CenterText>>,
    mut commands:Commands,
    ca:Res<CommonAssets>
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
            netcode::client::Event::Message(msg) => {
                match msg {
                    Message::JoinAsPlayer { id, name } => {},
                    Message::TileVisible { pos, wall } => {
                        commands.spawn(PbrBundle {
                            mesh:ca.block_mesh.clone(),
                            transform:Transform::from_xyz(pos.x as f32, 0.5, pos.y as f32),
                            ..Default::default()
                        });
                    },
                }
            }
        }
    }
}
