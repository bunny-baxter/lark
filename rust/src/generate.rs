use cgmath::vec2;

use crate::data::{ActorType, CellType, ItemType, TilePoint, TileDelta, TileSize};

fn create_2d_vec<T: Default + Clone>(size: TileSize) -> Vec<Vec<T>> {
    let mut result = Vec::with_capacity(size.x);
    for _i in 0..size.x {
        result.push(vec![ T::default() ; size.y ]);
    }
    result
}

#[derive(Debug, Default)]
pub struct RoomGenerationConfig {
}

#[derive(Clone, Debug, Default)]
pub struct GeneratedCell {
    pub cell_type: CellType,
    pub monster: Option<ActorType>,
    pub item: Option<ItemType>,
}

pub fn generate_room(config: RoomGenerationConfig) -> Vec<Vec<GeneratedCell>> {
    create_2d_vec::<GeneratedCell>(vec2(13, 9))
}
