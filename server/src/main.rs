use std::{collections::HashMap, time::Duration};

use endlessgrid::Grid;
use glam::IVec2;
use log::info;
use netcode::server::Server;
use shared::Message;
use slotmap::{DefaultKey, SlotMap};
use tiled::Loader;
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct Tile {
    pub wall: bool,
}
pub struct Entity {
    pub pos: IVec2,
}
pub struct Player {}
pub struct State {
    pub grid: Grid<Tile>,
    pub entities: SlotMap<DefaultKey, Entity>,
    pub players: HashMap<Uuid, Player>,
}
impl State {
    pub fn spawn_entity(&mut self, entity: Entity) -> DefaultKey {
        self.entities.insert(entity)
    }
}

fn load_map(grid: &mut Grid<Tile>, _entities: &mut SlotMap<DefaultKey, Entity>) {
    let mut loader = Loader::new();
    let map = loader.load_tmx_map("assets/maps/basic.tmx").unwrap();
    for layer in map.layers() {
        let Some(layer) = layer.as_tile_layer() else {
            continue;
        };
        let tiled::TileLayer::Infinite(layer) = layer else {
            continue;
        };
        for (chunk_pos, chunk) in layer.chunks() {
            for x in 0..tiled::ChunkData::WIDTH as i32 {
                for y in 0..tiled::ChunkData::HEIGHT as i32 {
                    if let Some(tile) = chunk.get_tile(x, y) {
                        let tile_pos = (
                            chunk_pos.0 * tiled::ChunkData::WIDTH as i32 + x,
                            chunk_pos.1 * tiled::ChunkData::HEIGHT as i32 + y,
                        );
                        let classes = tile
                            .get_tile()
                            .unwrap()
                            .user_type
                            .clone()
                            .unwrap_or_default();
                        let classes = classes.split(' ').map(|x| (x.to_owned(), ()));
                        let classes: HashMap<String, ()> = classes.collect();
                        if classes.contains_key("tile") {
                            let mut tile = Tile::default();
                            if classes.contains_key("wall") {
                                tile.wall = true;
                            }
                            grid.insert(tile_pos, tile);
                        }
                        if classes.contains_key("entity") {
                            if classes.contains_key("player") {
                               
                            }
                            if classes.contains_key("door") {
                               
                            }
                        }
                    }
                }
            }
        }
    }

    
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut state = State {
        grid: Default::default(),
        entities: Default::default(),
        players: Default::default(),
    };
    load_map(&mut state.grid, &mut state.entities);
    let port = 8080;
    info!("Starting server on port {}", port);
    let mut server = Server::default() as Server<Message>;
    server.start(port).await;
    loop {
        for e in server.poll().iter() {
            match e {
                netcode::server::Event::ClientConnected { client_id } => {
                    //println!("client connected");
                }
                netcode::server::Event::ClientDisconnected { client_id } => {
                    //println!("client disconnected");
                }
                netcode::server::Event::Message { client_id, msg } => match msg {
                    Message::JoinAsPlayer { id, name } => {
                        info!("Player '{}' joined with id {}", name, id);
                        for chunk in &state.grid {
                            for (i, tile) in chunk {
                                server.send(client_id.to_owned(), Message::TileVisible { pos: i.into(), wall: tile.wall });
                            }
                        }
                    }
                    Message::TileVisible { pos: _, wall: _ } => {},
                },
            }
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
