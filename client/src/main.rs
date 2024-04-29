use bevy::{
    prelude::*,
    render::{
        mesh::primitives, settings::{Backends, WgpuSettings}, texture::ImageSampler, RenderPlugin
    },
};
use endlessgrid::Grid;
use shared::{HasClass, Message};
use slotmap::{DefaultKey, SlotMap};
use std::{collections::HashMap, sync::Mutex};
use uuid::Uuid;

#[derive(Component, Default, Clone)]
pub struct Thing {
    pub entity:Option<Entity>,
    pub classes:String,
    pub pos:IVec2,
    pub visible:bool,
}
impl HasClass for Thing {
    fn classes(&self) -> &String {
        &self.classes
    }
}

#[derive(Component, Clone, Default)]
pub struct Tile {
    pub entity:Option<Entity>,
    pub pos:IVec2,
    pub wall:bool,
    pub visible:bool,
}

#[derive(Component, Default)]
pub struct CameraController {
}

#[derive(Component)]
pub struct Ground;

#[derive(Component)]
pub struct CenterText;

#[derive(Resource, Default)]
pub struct CommonAssets {
    pub thing_mesh:Handle<Mesh>,
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
        .add_systems(Update, (poll_client, spawn_tile, update_tile, spawn_things, update_things, camera_control, cursor).chain())
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
    //ca.thing_mesh = meshes.add(Cuboid::new(1., 1., 0.01));
    ca.thing_mesh = meshes.add(Plane3d::default().mesh().size(1.0, 1.0).normal(Direction3d::Z));
    ca.block_mesh = meshes.add(Cuboid::new(1., 1., 1.));
    ca.floor_mesh = meshes.add(Plane3d::default().mesh().size(1., 1.));
    let mut load = |name:&str, texture:&str| {
        let texture = texture.to_owned();
        ca.standard_materials.insert(name.to_owned(), ass.add(StandardMaterial {
            base_color_texture:Some(ass.load(texture)),
            cull_mode:None,
            unlit:true,
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
        transform: Transform::from_xyz(0.0, 7.0, 7.0).looking_at(Vec3::ZERO, -Vec3::Z),
        ..default()
    }).insert(CameraController::default());
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

fn camera_control(mut q:Query<(&mut CameraController, &mut Transform)>, keyboard_input:Res<ButtonInput<KeyCode>>, time:Res<Time>) {
    let (_controller, mut transform) = q.single_mut();
    let mut d = Vec3::default();
    if keyboard_input.pressed(KeyCode::KeyA) {
        d.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        d.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyW) {
        d.z -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        d.z += 1.0;
    }
    let speed = 10.0;
    let v = d * time.delta_seconds() * speed;
    transform.translation += v;
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

fn spawn_things(mut commands: Commands, mut st:ResMut<ServerState>) {
    for (_id, thing) in st.things.iter_mut() {
        if thing.entity.is_none() {
            let id = commands.spawn(PbrBundle::default()).insert(Thing::default()).id();
            thing.entity = Some(id);
        }
    }
}

fn update_things(mut q:Query<(&mut Thing, &mut Transform, &mut Handle<Mesh>, &mut Handle<StandardMaterial>)>, mut st:ResMut<ServerState>, ca:Res<CommonAssets>) {
    for (_, thing) in st.things.iter_mut() {
        let Some(entity) = thing.entity else { continue; };
        let Ok((mut entity_thing, mut transform, mut mesh, mut material)) = q.get_mut(entity) else { continue;};
        *mesh = ca.thing_mesh.clone();
        if thing.has_class("player") {
            *material = ca.standard_material("player");
        }
        if thing.has_class("door") {
            *material = ca.standard_material("door");
        }
        *transform = Transform::from_xyz(thing.pos.x as f32 + 0.5, 0.5, thing.pos.y as f32 + 0.5).looking_to(Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0));//.looking_at(Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 0.0));
        *entity_thing = thing.clone();
    }
}

fn spawn_tile(mut commands: Commands, mut st:ResMut<ServerState>) {
    for chunk in &mut st.grid {
        for (_, tile) in chunk {
            if tile.entity.is_none() {
                let id = commands.spawn(PbrBundle::default()).insert(Tile::default()).id();
                tile.entity = Some(id);
            }
        }
    }
}

fn update_tile(mut q:Query<(&mut Tile, &mut Transform, &mut Handle<Mesh>, &mut Handle<StandardMaterial>)>, mut st:ResMut<ServerState>, ca:Res<CommonAssets>) {
    for chunk in &mut st.grid {
        for (_, tile) in chunk {
            let Some(entity) = tile.entity else { continue; };
            let Ok((mut entity_tile, mut transform, mut mesh, mut material)) = q.get_mut(entity) else { continue; };
            let wall = tile.wall;
            *material = if wall { ca.standard_material("wall")} else {ca.standard_material("floor")};
            *mesh = if wall { ca.block_mesh.clone() } else { ca.floor_mesh.clone() };
            let y = if wall { 0.5 } else { 0.01 };
            let pos:IVec2 = tile.pos;
            *transform = Transform::from_xyz(pos.x as f32 + 0.5, y, pos.y as f32 + 0.5);
            *entity_tile = tile.clone();
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
                        let i:(i32, i32) = pos.into();
                        let tile = match st.grid.get_mut(i) {
                            Some(tile) => tile,
                            None => {
                                st.grid.insert(i, Default::default());
                                st.grid.get_mut(i).unwrap()
                            }
                        };
                        tile.pos = pos;
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
                        thing.pos = pos.unwrap_or(thing.pos);
                        thing.classes = classes.unwrap_or(thing.classes.clone());
                    }
                    _ => {}
                }
            }
        }
    }
}