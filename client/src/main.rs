use bevy::{
    prelude::*,
    render::{
        mesh::primitives, settings::{Backends, WgpuSettings}, texture::ImageSampler, RenderPlugin
    },
};
use endlessgrid::Grid;
use shared::Message;
use slotmap::{DefaultKey, SlotMap};
use std::{collections::HashMap, sync::Mutex};
use uuid::Uuid;

#[derive(Component, Default)]
pub struct Thing {
    pub entity:Option<Entity>,
    pub classes:String,
    pub pos:Vec2,
    pub visible:bool,
}

#[derive(Component, Clone, Default)]
pub struct Tile {
    pub entity:Option<Entity>,
    pub pos:IVec2,
    pub wall:bool,
    pub visible:bool,
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
}

#[derive(Resource, Default)]
pub struct ServerState {
    pub things:SlotMap<DefaultKey, Thing>,
    pub grid:Grid<Tile>
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
        .insert_resource(ServerState::default())
        .insert_resource(CommonAssets::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (poll_client, spawn_tile, cursor).chain())
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

fn spawn_tile(mut commands: Commands, mut st:ResMut<ServerState>, ca:Res<CommonAssets>) {
    for chunk in &mut st.grid {
        for (index, tile) in chunk {
            if tile.entity.is_none() {
                let wall = tile.wall;
                let material = if wall { ca.standard_material("wall")} else {ca.standard_material("floor")};
                let mesh = if wall { ca.block_mesh.clone() } else { ca.floor_mesh.clone() };
                let y = if wall { 0.5 } else { 0.01 };
                let pos:IVec2 = index.into();
                let transform = Transform::from_xyz(pos.x as f32, y, pos.y as f32);
                let id = commands.spawn(PbrBundle {
                    mesh,
                    material,
                    transform,
                    ..Default::default()
                }).insert(tile.clone()).id();
                tile.entity = Some(id);
            }
        }
        
    }
}

fn poll_client(
    client: ResMut<Client>,
    mut player: ResMut<Player>,
    mut center_text: Query<&mut Text, With<CenterText>>,
    mut st:ResMut<ServerState>
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
                    Message::TileUpdate { pos, wall, visible } => {
                        /*let material = if wall { ca.standard_material("wall")} else {ca.standard_material("floor")};
                        let mesh = if wall { ca.block_mesh.clone() } else { ca.floor_mesh.clone() };
                        let y = if wall { 0.5 } else { 0.01 };
                        let transform = Transform::from_xyz(pos.x as f32, y, pos.y as f32);
                        commands.spawn(PbrBundle {
                            mesh,
                            material,
                            transform,
                            ..Default::default()
                        });*/
                        let i:(i32, i32) = pos.into();
                        let tile = match st.grid.get_mut(i) {
                            Some(tile) => tile,
                            None => {
                                st.grid.insert(i, Default::default());
                                st.grid.get_mut(i).unwrap()
                            }
                        };
                        tile.wall = wall.unwrap_or(tile.wall);
                        tile.visible = visible.unwrap_or(tile.visible);
                    },
                    Message::WelcomePlayer { your_entity } => {
                        player.entityid = Some(your_entity);
                    },
                    Message::ThingUpdate { id, pos, classes, visible } => {
                        let thing = match st.things.get_mut(id) {
                            Some(thing) => thing,
                            None => {
                                st.things.insert(Default::default());
                                st.things.get_mut(id).unwrap()
                            },
                        };
                    }
                    _ => {}
                }
            }
        }
    }
}