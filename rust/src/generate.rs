use std::collections::HashSet;

use cgmath::vec2;
use rand::prelude::IndexedRandom;
use rand::Rng;

use crate::data::{ActorType, CellType, ItemType, MiscEntityType, NEIGHBORS, TilePoint, TileSize};

fn create_2d_vec<T: Default + Clone>(size: TileSize) -> Vec<Vec<T>> {
    let mut result = Vec::with_capacity(size.x);
    for _i in 0..size.x {
        result.push(vec![ T::default() ; size.y ]);
    }
    result
}

const AUTOMATA_NEIGHBORS: &[(i32, i32)] = &[
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

#[derive(Clone, Debug)]
pub struct RoomGenerationConfig {
    pub size: TileSize,
}

#[derive(Clone, Debug, Default)]
pub struct GeneratedCell {
    pub cell_type: CellType,
    pub immutable: bool,
    pub monster: Option<ActorType>,
    pub item: Option<ItemType>,
    pub misc_entity: Option<MiscEntityType>,
}

#[derive(Debug)]
pub struct GeneratedRoom {
    pub cells: Vec<Vec<GeneratedCell>>,
    pub exits: Vec<TilePoint>,
    pub player_start: TilePoint,
}

fn is_navigable(cell_type: CellType) -> bool {
    match cell_type {
        CellType::DefaultFloor | CellType::FloorMoss | CellType::FloorThyme | CellType::RoomExit | CellType::Water => true,
        CellType::OutOfBounds | CellType::DefaultWall | CellType::Empty => false,
    }
}

fn is_open(cell_type: CellType) -> bool {
    match cell_type {
        CellType::DefaultFloor | CellType::FloorMoss | CellType::FloorThyme => true,
        _ => false,
    }
}

fn floodfill_navigable_area_recursive_helper(room: &Vec<Vec<GeneratedCell>>, current: TilePoint, result: &mut HashSet<TilePoint>) {
    if !is_navigable(room[current.x as usize][current.y as usize].cell_type) {
        return;
    }
    result.insert(current);
    for &(dx, dy) in NEIGHBORS.iter() {
        let adj = vec2(current.x + dx, current.y + dy);
        if result.contains(&adj) {
            continue;
        }
        floodfill_navigable_area_recursive_helper(room, adj, result);
    }
}

fn floodfill_navigable_area(room: &Vec<Vec<GeneratedCell>>, start: TilePoint) -> HashSet<TilePoint> {
    let mut result = HashSet::new();
    floodfill_navigable_area_recursive_helper(room, start, &mut result);
    result
}

fn partition_navigable_areas(size: TileSize, room: &Vec<Vec<GeneratedCell>>) -> Vec<HashSet<TilePoint>> {
    let mut visited_points = HashSet::new();
    let mut areas = vec![];
    for x in 0..size.x { for y in 0..size.y {
        let point = vec2(x as i32, y as i32);
        if visited_points.contains(&point) {
            continue;
        }
        let area = floodfill_navigable_area(room, point);
        for &p in area.iter() {
            visited_points.insert(p);
        }
        if area.len() > 0 {
            areas.push(area);
        }
    }}
    areas
}

fn connect_with_drunkards_walk(room: &mut Vec<Vec<GeneratedCell>>, current: TilePoint, end: TilePoint) {
    let mut rng = rand::rng();
    if current == end {
        return;
    }
    let delta = if rng.random::<f32>() < 0.25 {
        *NEIGHBORS.choose(&mut rng).unwrap()
    } else {
        let dx = end.x - current.x;
        let dy = end.y - current.y;
        if dx.abs() > dy.abs() {
            (dx.signum(), 0)
        } else {
            (0, dy.signum())
        }
    };
    let next = vec2(current.x + delta.0, current.y + delta.1);
    let navigable = is_navigable(room[next.x as usize][next.y as usize].cell_type);
    if room[next.x as usize][next.y as usize].immutable && !navigable {
        connect_with_drunkards_walk(room, current, end);
    } else {
        if !navigable {
            room[next.x as usize][next.y as usize].cell_type = CellType::DefaultFloor;
        }
        connect_with_drunkards_walk(room, next, end);
    }
}

fn find_edge_walls(size: TileSize, room: &Vec<Vec<GeneratedCell>>) -> Vec<TilePoint> {
    let mut result = vec![];
    for x in 0..size.x { for y in 0..size.y {
        if room[x][y].cell_type != CellType::DefaultWall {
            continue;
        }
        let point = vec2(x as i32, y as i32);
        let mut adjacent_floors = 0;
        for &d in NEIGHBORS.iter() {
            let neighbor = vec2(point.x + d.0, point.y + d.1);
            if neighbor.x < 0 || neighbor.y < 0 || neighbor.x as usize >= size.x || neighbor.y as usize >= size.y {
                continue;
            }
            if is_navigable(room[neighbor.x as usize][neighbor.y as usize].cell_type) {
                adjacent_floors += 1;
            }
        }
        if adjacent_floors == 1 || adjacent_floors == 2 {
            result.push(point);
        }
    }}
    result
}

fn collect_open_cells(size: TileSize, room: &Vec<Vec<GeneratedCell>>, player_start: TilePoint) -> Vec<TilePoint> {
    let mut result = vec![];
    for x in 0..size.x { for y in 0..size.y {
        let p = vec2(x as i32, y as i32);
        if p != player_start && is_open(room[x][y].cell_type) {
            result.push(p);
        }
    }}
    result
}

pub fn generate_room(maybe_player_start: Option<TilePoint>, config: RoomGenerationConfig) -> GeneratedRoom {
    let mut room = create_2d_vec::<GeneratedCell>(config.size);
    let mut rng = rand::rng();

    let inner_width_range = 1..(config.size.x - 1);
    let inner_height_range = 1..(config.size.y - 1);

    let player_start = match maybe_player_start {
        Some(player_start) => vec2(player_start.x as usize, player_start.y as usize),
        None => vec2(rng.random_range(inner_width_range.clone()), rng.random_range(inner_height_range.clone())),
    };

    // Initialize randomly
    for x in 0..config.size.x { for y in 0..config.size.y {
        if x == 0 || y == 0 || x == config.size.x - 1 || y == config.size.y - 1 {
            room[x][y].cell_type = CellType::DefaultWall;
            room[x][y].immutable = true;
        } else if x == player_start.x && y == player_start.y {
            room[x][y].cell_type = CellType::DefaultFloor;
            room[x][y].immutable = true;
        } else {
            room[x][y].cell_type = if rng.random::<f32>() < 0.7 { CellType::DefaultFloor } else { CellType::DefaultWall };
        }
    }}

    // Smooth with cellular automata
    for _i in 0..2 {
        let mut next_room = room.clone();
        for x in inner_width_range.clone() { for y in inner_height_range.clone() {
            if room[x][y].immutable {
                next_room[x][y] = room[x][y].clone();
                continue;
            }

            let mut neighbor_walls = 0;
            for &neighbor in AUTOMATA_NEIGHBORS.iter() {
                // Edges are safe because we iterate only over the inner size
                let adj = vec2((x as i32 + neighbor.0) as usize, (y as i32 + neighbor.1) as usize);
                if room[adj.x][adj.y].cell_type == CellType::DefaultWall {
                    neighbor_walls += 1;
                }
            }
            let mut next_cell_type = room[x][y].cell_type;
            if neighbor_walls < 2 {
                if rng.random::<f32>() < 0.7 {
                    next_cell_type = CellType::DefaultFloor;
                }
            } else if neighbor_walls < 7 {
                if rng.random::<f32>() < 0.05 {
                    next_cell_type = if next_cell_type == CellType::DefaultWall { CellType::DefaultFloor } else { CellType::DefaultWall };
                }
            } else {
                if rng.random::<f32>() < 0.5 {
                    next_cell_type = CellType::DefaultWall;
                }
            }
            next_room[x][y].cell_type = next_cell_type;
        }}
        room = next_room;
    }

    // Erase small islands
    {
        let navigable_areas = partition_navigable_areas(config.size, &room);
        for area in navigable_areas.into_iter() {
            if area.len() <= 3 {
                for p in area.into_iter() {
                    if !room[p.x as usize][p.y as usize].immutable {
                        room[p.x as usize][p.y as usize].cell_type = CellType::DefaultWall;
                    }
                }
            }
        }
    }

    // Connect large islands
    {
        let navigable_areas = partition_navigable_areas(config.size, &room);
        let first_area = navigable_areas[0].iter().collect::<Vec<&TilePoint>>();
        for i in 1..navigable_areas.len() {
            let area = navigable_areas[i].iter().collect::<Vec<&TilePoint>>();
            let start = **first_area.choose(&mut rng).unwrap();
            let end = **area.choose(&mut rng).unwrap();
            connect_with_drunkards_walk(&mut room, start, end);
        }
    }

    // Place exit(s)
    let mut exits = vec![];
    {
        let exit_candidates = find_edge_walls(config.size, &room);
        let exit = exit_candidates.choose(&mut rng).unwrap();
        exits.push(*exit);
        room[exit.x as usize][exit.y as usize].cell_type = CellType::RoomExit;
    }

    let player_start_i32 = vec2(player_start.x as i32, player_start.y as i32);
    let mut open_cells: Vec<TilePoint> = collect_open_cells(config.size, &room, player_start_i32);

    let monster_count = rng.random_range(5..=8);
    let monster_types = [
        ActorType::Toad,
        ActorType::Toad,
        ActorType::Toad,
        ActorType::Toad,
        ActorType::Toad,
        ActorType::MouseWarrior,
        ActorType::ToothyStarling,
        ActorType::ToothyStarling,
        ActorType::DustySkeleton,
        ActorType::BlueJelly,
        ActorType::BlueJelly,
        ActorType::BlueJelly,
    ];
    for _ in 0..monster_count {
        if open_cells.is_empty() {
            break;
        }
        let i = rng.random_range(0..open_cells.len());
        let pos = open_cells.swap_remove(i);
        let monster_type = *monster_types.choose(&mut rng).unwrap();
        room[pos.x as usize][pos.y as usize].monster = Some(monster_type);
    }

    let item_count = rng.random_range(2..=4);
    let item_types = [
        ItemType::LumpOfBlackstone,
        ItemType::LumpOfBlackstone,
        ItemType::MoonlightKnife,
        ItemType::MoonlightKnife,
        ItemType::BlackstoneSpear,
        ItemType::BlackstoneSpear,
        ItemType::CarmineSword,
        ItemType::BoneLamellar,
        ItemType::BoneLamellar,
        ItemType::BoneLamellar,
        ItemType::CarmineChainmail,
        ItemType::FeatheredCavalier,
        ItemType::FeatheredCavalier,
        ItemType::FeatheredCavalier,
        ItemType::CarmineHelm,
        ItemType::Bloodflower,
        ItemType::Bloodflower,
        ItemType::Bloodflower,
        ItemType::Bloodflower,
        ItemType::Bloodflower,
        ItemType::Bloodflower,
        ItemType::WandOfIce,
        ItemType::WandOfIce,
        ItemType::WandOfIce,
    ];
    for _ in 0..item_count {
        if open_cells.is_empty() {
            break;
        }
        let i = rng.random_range(0..open_cells.len());
        let pos = open_cells.swap_remove(i);
        let item_type = *item_types.choose(&mut rng).unwrap();
        room[pos.x as usize][pos.y as usize].item = Some(item_type);
    }

    let thistle_count = rng.random_range(0..=12);
    for _ in 0..thistle_count {
        if open_cells.is_empty() {
            break;
        }
        let i = rng.random_range(0..open_cells.len());
        let pos = open_cells.swap_remove(i);
        room[pos.x as usize][pos.y as usize].misc_entity = Some(MiscEntityType::SteelThistle);
    }

    GeneratedRoom {
        cells: room,
        exits,
        player_start: player_start_i32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partition_navigable_areas() {
        let size = vec2(5, 5);
        let mut room = create_2d_vec::<GeneratedCell>(size);
        for x in 0..size.x { for y in 0..size.y {
            room[x][y].cell_type = CellType::DefaultWall;
        }}
        room[1][1].cell_type = CellType::DefaultFloor;
        room[3][1].cell_type = CellType::DefaultFloor;
        room[3][2].cell_type = CellType::DefaultFloor;

        let partition_result = partition_navigable_areas(size, &room);
        assert_eq!(2, partition_result.len());
        assert_eq!(1, partition_result[0].len());
        assert!(partition_result[0].contains(&vec2(1, 1)));
        assert_eq!(2, partition_result[1].len());
        assert!(partition_result[1].contains(&vec2(3, 1)));
        assert!(partition_result[1].contains(&vec2(3, 2)));
    }
}
