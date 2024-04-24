use std::{collections::HashMap, time::Duration};

use endlessgrid::Grid;
use glam::IVec2;
use log::{error, info};
use netcode::server::Server;
use shared::Message;
use slotmap::{DefaultKey, Key, SlotMap};
use tiled::Loader;
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct Tile {
    pub wall: bool,
}
pub struct Thing {
    pub pos: IVec2,
    pub classes:String
}

#[derive(Default)]
pub struct Player {
    pub thing:Option<DefaultKey>
}
pub struct State {
    pub grid: Grid<Tile>,
    pub things: SlotMap<DefaultKey, Thing>,
    pub players: HashMap<Uuid, Player>,
}
impl State {
    pub fn spawn_entity(&mut self, entity: Thing) -> DefaultKey {
        self.things.insert(entity)
    }
}

fn load_map(grid: &mut Grid<Tile>, entities: &mut SlotMap<DefaultKey, Thing>) {
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
                        if classes.contains("tile") {
                            let mut tile = Tile::default();
                            if classes.contains("wall") {
                                tile.wall = true;
                            }
                            grid.insert(tile_pos, tile);
                        }
                        if classes.contains("entity") {
                            let entity = Thing {
                                classes:classes.clone(),
                                pos:tile_pos.into()
                            };
                            entities.insert(entity);
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
        things: Default::default(),
        players: Default::default(),
    };
    load_map(&mut state.grid, &mut state.things);
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
                        let mut player = match state.players.get_mut(id) {
                            Some(player) => player,
                            None => {
                                state.players.insert(id.to_owned(), Player::default());
                                state.players.get_mut(id).unwrap()
                            },
                        };
                        if player.thing.is_none() {
                            // spawn entity for player
                            let player_spawn = state.things.iter().filter(|x|x.1.classes.contains("spawn_player")).next();
                            match player_spawn {
                                Some(player_spawn) => {
                                    let spawn_pos = player_spawn.1.pos;
                                    let id = state.things.insert(Thing {
                                        pos:spawn_pos.clone(),
                                        classes:"player".to_owned()
                                    });
                                    player.thing = Some(id);
                                    info!("Spawning player at {}", spawn_pos);
                                    server.send(client_id.to_owned(), Message::WelcomePlayer { your_entity: id.data().as_ffi() });

                                },
                                None => {
                                    error!("Cannot spawn player since 'player_spawn' is not found");
                                },
                            }
                        }
                        for chunk in &state.grid {
                            for (i, tile) in chunk {
                                let r = server.send(client_id.to_owned(), Message::TileVisible { pos: i.into(), wall: tile.wall });
                                if r == false {
                                    dbg!("failed");
                                }
                            }
                        }

                        for (id, thing) in &state.things {
                            server.send(*client_id, Message::ThingUpdate { id: id, pos: Some(thing.pos), classes: Some(thing.classes.clone()), visible: Some(true) });
                            server.send(*client_id, Message::ThingUpdate { id: id, pos: Some(thing.pos), classes: Some(thing.classes.clone()), visible: Some(true) });
                        }
                    }
                    _ => {}
                },
            }
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
