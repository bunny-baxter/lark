use cgmath::vec2;

use crate::types::*;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum CellType {
    OutOfBounds = -1,
    Empty = 0,
    Floor = 1,
    DefaultWall,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Cell {
    cell_type: CellType,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            cell_type: CellType::Empty,
        }
    }
}

#[derive(Debug)]
pub struct Actor {
    id: u32,
    position: TilePoint,
}

#[derive(Debug)]
pub struct Room {
    pub size: TileSize,
    pub cells: Vec<Vec<Cell>>,
    pub actors: Vec<Actor>,
    pub next_actor_id: u32,
    pub player_index: usize,
}

impl Room {
    fn new(size: TileSize) -> Self {
        let mut cells = Vec::with_capacity(size.x);
        for _i in 0..size.x {
            cells.push(vec![ Cell::default() ; size.y ]);
        }
        Room {
            size,
            cells,
            actors: vec![],
            next_actor_id: 0,
            player_index: 0,
        }
    }

    fn create_actor(&mut self, position: TilePoint) {
        self.actors.push(Actor { id: self.next_actor_id, position });
        self.next_actor_id += 1;
    }

    pub fn create_player(&mut self, position: TilePoint) {
        self.create_actor(position);
        self.player_index = self.actors.len() - 1;
    }

    pub fn find_actors_at(&self, position: TilePoint) -> Vec<usize> {
        let mut result = vec![];
        for i in 0..self.actors.len() {
            if self.actors[i].position == position {
                result.push(i);
            }
        }
        result
    }

    pub fn get_cell_type(&self, position: TilePoint) -> CellType {
        if position.x < 0 || position.y < 0 || position.x as usize >= self.size.x || position.y as usize >= self.size.y {
            return CellType::OutOfBounds;
        }
        self.cells[position.x as usize][position.y as usize].cell_type
    }

    fn teleport_actor(&mut self, actor_index: usize, new_position: TilePoint) {
        self.actors[actor_index].position = new_position;
    }

    fn actor_walk(&mut self, actor_index: usize, delta: TileDelta) -> bool {
        let next_position = self.actors[actor_index].position + delta;
        let next_cell_type = self.get_cell_type(next_position);
        match next_cell_type {
            CellType::DefaultWall | CellType::OutOfBounds => return false,
            _ => {},
        };
        if self.find_actors_at(next_position).len() > 0 {
            return false;
        }
        self.teleport_actor(actor_index, next_position);
        true
    }
}

fn create_blank_room(size: TileSize) -> Room {
    let mut room = Room::new(size);
    for x in 0..size.x {
        for y in 0..size.y {
            if x == 0 || y == 0 || x as usize == size.x - 1 || y as usize == size.y - 1 {
                room.cells[x][y].cell_type = CellType::DefaultWall;
            } else {
                room.cells[x][y].cell_type = CellType::Floor;
            }
        }
    }
    room
}

#[derive(Copy, Clone, Debug)]
pub enum Command {
    Walk { delta: TileDelta },
}

pub struct GameInstance {
    pub current_room: Room,
}

impl GameInstance {
    pub fn new() -> Self {
        GameInstance {
            current_room: create_blank_room(vec2(9, 9)),
        }
    }

    pub fn execute_command(&mut self, command: Command) {
        match command {
            Command::Walk { delta } => self.current_room.actor_walk(self.current_room.player_index, delta),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_room() {
        let room = Room::new(vec2(4, 5));
        assert_eq!(room.size, TileSize::new(4, 5));
        assert_eq!(room.cells.len(), 4);
        assert_eq!(room.cells[0].len(), 5);
        assert_eq!(room.actors.len(), 0);
    }

    #[test]
    fn test_create_actors() {
        let mut room = Room::new(vec2(4, 5));
        room.create_actor(vec2(0, 0));
        room.create_player(vec2(1, 0));
        assert_eq!(room.actors.len(), 2);
        assert_ne!(room.actors[0].id, room.actors[1].id);
        assert_eq!(room.player_index, 1);
    }

    #[test]
    fn test_actor_moves() {
        let mut game = GameInstance::new();
        let room = &mut game.current_room;
        room.create_player(vec2(1, 0));
        let walk_1 = room.actor_walk(room.player_index, vec2(0, -1));
        assert!(!walk_1);
        assert_eq!(room.actors[room.player_index].position, vec2(1, 0));
        let walk_2 = room.actor_walk(room.player_index, vec2(0, 1));
        assert!(walk_2);
        assert_eq!(room.actors[room.player_index].position, vec2(1, 1));
    }
}
