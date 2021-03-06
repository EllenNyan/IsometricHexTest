use vermarine_lib::hexmap::{Axial, Hex, HexChunk, HexMap, CHUNK_HEIGHT, CHUNK_WIDTH};

use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

use crate::consts::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum HexPathNode {
    TopLeft,
    TopRight,
    Right,
    BottomRight,
    BottomLeft,
    Left,

    Goal,
}

impl HexPathNode {
    pub fn from_hex(start: Hex, end: Hex) -> HexPathNode {
        let start = start.to_axial();
        let end = end.to_axial();

        let (q, r) = (start.q - end.q, start.r - end.r);

        match (q, r) {
            (0, -1) => HexPathNode::TopLeft,
            (1, -1) => HexPathNode::TopRight,
            (1, 0) => HexPathNode::Right,
            (0, 1) => HexPathNode::BottomRight,
            (-1, 1) => HexPathNode::BottomLeft,
            (-1, 0) => HexPathNode::Left,
            (0, 0) => HexPathNode::Goal,
            _ => unreachable!(),
        }
    }

    pub fn to_hex(&self) -> Hex {
        let (q, r) = match self {
            HexPathNode::TopLeft => (0, -1),
            HexPathNode::TopRight => (1, -1),
            HexPathNode::Right => (1, 0),
            HexPathNode::BottomRight => (0, 1),
            HexPathNode::BottomLeft => (-1, 1),
            HexPathNode::Left => (-1, 0),
            HexPathNode::Goal => (0, 0),
        };

        Axial::new(q, r).to_hex()
    }
}

pub struct Map {
    pub terrain: HexMap<HexTileData>,
    pub dijkstra: HexMap<HexPathNode>,
}

impl Map {
    pub fn new() -> Map {
        let hex_width = 36.;
        let hex_height = 36.;
        let hex_vert_step = 28.;
        let hex_depth_step = 12.;

        let wall_vert_offset = 12.;
        let wall_vert_step = 12.;

        let mut terrain = HexMap::<HexTileData>::new(
            hex_width,
            hex_height,
            hex_vert_step,
            hex_depth_step,
            wall_vert_offset,
            wall_vert_step,
        );

        let mut rand = StdRng::from_entropy();
        let mut chunks = vec![];
        let mut tallest = 0;
        for q in 0..WIDTH {
            for r in 0..HEIGHT {
                let mut tiles = [None; CHUNK_WIDTH * CHUNK_HEIGHT];

                for tile in tiles.iter_mut() {
                    let value = rand.gen_range(0, MAX_FLOOR_HEIGHT as u16 + 1) as u8;
                    *tile = Some(HexTileData::new(value));
                    if value > tallest {
                        tallest = value;
                    }
                }

                chunks.push(HexChunk::new(tiles, q as i32 - 1, r as i32 - 1));
            }
        }

        terrain.get_height = HexTileData::get_height;

        terrain.tallest = tallest;
        for chunk in chunks.into_iter() {
            terrain.insert_chunk(chunk);
        }

        let dijkstra = HexMap::<HexPathNode>::new(
            hex_width,
            hex_height,
            hex_vert_step,
            hex_depth_step,
            wall_vert_offset,
            wall_vert_step,
        );
        let goal_hex = Axial::new(10, 5).to_hex();

        let mut map = Map { terrain, dijkstra };

        let goals = vec![goal_hex + Axial::new(0, 1), goal_hex + Axial::new(1, 1)];
        map.update_dijkstra(goals);

        map
    }

    pub fn get_path(&self, start: Hex) -> Option<Vec<Hex>> {
        let mut path = vec![start];

        self.dijkstra.get_tile(start)?;

        let mut current_tile = start;
        loop {
            let path_node = *self.dijkstra.get_tile(current_tile).unwrap();

            if path_node == HexPathNode::Goal {
                return Some(path);
            }

            let mut hex = path_node.to_hex();
            hex += current_tile;

            path.push(hex);
            current_tile = hex;
        }
    }

    pub fn flatten_tile(&mut self, hex: Hex, height: u8) {
        if let Some(tile) = self.terrain.get_tile_mut(hex) {
            if tile.get_height() == height {
                return;
            } else if tile.get_height() < height {
                tile.wall_height = height;
            } else if tile.get_height() > height {
                tile.wall_height = height;

                if tile.ground_height > height {
                    tile.ground_height = height;
                }
            }
        } else {
            self.terrain.set_tile(hex, HexTileData::new_wall(height));
            return;
        };
    }

    pub fn update_dijkstra(&mut self, goals: Vec<Hex>) {
        update_dijkstra_hexmap(&self.terrain, &mut self.dijkstra, goals);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HexTileData {
    pub ground_height: u8,
    pub wall_height: u8,
}

impl HexTileData {
    pub fn new(height: u8) -> HexTileData {
        HexTileData {
            ground_height: height,
            wall_height: height,
        }
    }

    pub fn new_wall(height: u8) -> HexTileData {
        HexTileData {
            ground_height: 0,
            wall_height: height,
        }
    }

    pub fn get_height(&self) -> u8 {
        self.wall_height
    }
}

pub fn update_dijkstra_hexmap(
    terrain: &HexMap<HexTileData>,
    dijkstra: &mut HexMap<HexPathNode>,
    mut goals: Vec<Hex>,
) {
    dijkstra.clear_map();

    for &hex in goals.iter() {
        dijkstra.set_tile(hex, HexPathNode::Goal);
    }

    while !goals.is_empty() {
        let length = goals.len();
        for _ in 0..length {
            let tile = *goals.first().unwrap();
            let neighbors: Vec<_> = tile
                .neighbors()
                .iter()
                .filter(|&&hex| {
                    if terrain.get_tile(hex).is_some() && dijkstra.get_tile(hex).is_none() {
                        let tile_height = terrain.get_tile(tile).unwrap().get_height();
                        let hex_height = terrain.get_tile(hex).unwrap().get_height();

                        let (larger, smaller) = if hex_height > tile_height {
                            (hex_height, tile_height)
                        } else {
                            (tile_height, hex_height)
                        };

                        larger - smaller <= 1 && larger < MAX_BRICK_HEIGHT
                    } else {
                        false
                    }
                })
                .cloned()
                .collect();

            for neighbor in neighbors {
                goals.push(neighbor);
                dijkstra.set_tile(neighbor, HexPathNode::from_hex(tile, neighbor));
            }

            goals.remove(0);
        }
    }
}
