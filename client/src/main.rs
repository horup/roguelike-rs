use bevy::{
    prelude::*,
    render::{
        mesh::primitives, settings::{Backends, WgpuSettings}, texture::ImageSampler, RenderPlugin
    },
};
use shared::Message;
use std::{collections::HashMap, sync::Mutex};
use uuid::Uuid;

#[derive(Component)]
pub struct Thing {
    pub id:u64,
    pub classes:String,
    pub pos:Vec2,
    pub visible:bool
}

#[derive(Component)]
pub struct Ground;

#[derive(Component)]
pub struct CenterText;

#[derive(Resource, Default)]
pub struct CommonAssets {
    pub block_mesh:Handle<Mesh>,
    pub floor_mesh:Handle<Mesh>,
    pub standard_materials:HashMap<String, Handle<StandardMaterial>>,
    pub things:HashMap<u64, Entity>
}
impl CommonAssets {
    pub fn standard_material(&self, name:&str) -> Handle<StandardMaterial> {
        if let Some(handle) = self.standard_materials.get(name) {
            return handle.to_owned();
        }
        Default::default()
    }
}

#[derive(Resource)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub entityid:Option<u64>
}

#[derive(Resource)]
pub struct Client {
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
        }).set(ImagePlugin::default_nearest()))
        .insert_resource(Player {
            id: Uuid::new_v4(),
            name: "Player".to_owned(),
            entityid:None
        })
        .insert_resource(Client {
            client: Mutex::new(Default::default()),
        })
        .insert_resource(CommonAssets::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (cursor, update).chain())
        .run();
}

fn setup(
    mut commands: Commands,
    ass: Res<AssetServer>,
    client: ResMut<Client>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ca:ResMut<CommonAssets>
) {
    ca.block_mesh = meshes.add(Cuboid::new(1., 1., 1.));
    ca.floor_mesh = meshes.add(Plane3d::default().mesh().size(1., 1.));
    let mut load = |name:&str, texture:&str| {
        let texture = texture.to_owned();
        ca.standard_materials.insert(name.to_owned(), ass.add(StandardMaterial {
            base_color_texture:Some(ass.load(texture)),
            ..Default::default()
        }));
    };

    load("floor", "imgs/floor.png");
    load("wall", "imgs/wall.png");
    load("player", "imgs/player.png");
    load("door", "imgs/door.png");
    
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
        material: materials.add(StandardMaterial {
            base_color:Color::rgb(0., 0., 0.),
            unlit:true,
            ..Default::default()
        }),
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
    client: ResMut<Client>,
    mut player: ResMut<Player>,
    mut center_text: Query<&mut Text, With<CenterText>>,
    mut commands:Commands,
    mut things:Query<&mut Thing>,
    ca:Res<CommonAssets>
) {
    let mut client = client.client.lock().unwrap();
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
                    Message::TileVisible { pos, wall } => {
                        let material = if wall { ca.standard_material("wall")} else {ca.standard_material("floor")};
                        let mesh = if wall { ca.block_mesh.clone() } else { ca.floor_mesh.clone() };
                        let y = if wall { 0.5 } else { 0.01 };
                        let transform = Transform::from_xyz(pos.x as f32, y, pos.y as f32);
                        commands.spawn(PbrBundle {
                            mesh,
                            material,
                            transform,
                            ..Default::default()
                        });
                    },
                    Message::WelcomePlayer { your_entity } => {
                        player.entityid = Some(your_entity);
                    },
                    Message::ThingUpdate { id, pos, classes, visible } => {
                        /*let entity = match ca.things.get(&id) {
                            Some(entity) => *entity,
                            None => commands.spawn(Thing {
                                id,
                                classes: classes.unwrap_or_default(),
                                pos: pos.unwrap_or_default(),
                                visible: visibility.unwrap_or_default(),
                            }),
                        };*/
                        /*match things.iter_mut().filter(|x|x.id == id).next() {
                            Some(thing) => {

                            },
                            None => {
                                commands.
                            },
                        };*/
                    }
                    _ => {}
                }
            }
        }
    }
}
