use std::collections::HashMap;

use endlessgrid::Grid;
use netcode::server::Server;
use slotmap::{DefaultKey, SlotMap};
use tiled::Loader;

#[derive(Clone, Default)]
pub struct Tile {}
pub struct Entity {}

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
                        //let mut keys = HashMap::default();
                        if classes.contains_key("player") {
                            /*  let key = entities.insert(Entity {
                                index:tile.id() as u16,
                                pos:tile_pos.into(),
                                is_player:true
                            });
                            keys.insert(key, ());*/
                        }

                        grid.insert(
                            tile_pos,
                            Tile {
                           /* index:if classes.contains_key("entity") { 0 } else { tile.id() as u16 },
                            solid:classes.contains_key("solid"),
                            entities:keys*/
                        },
                        );
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let mut grid = Grid::default();
    let mut entities = Default::default();
    load_map(&mut grid, &mut entities);

    let server = Server::default() as Server<String>;
}
