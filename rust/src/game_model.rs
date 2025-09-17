use std::collections::HashMap;

use cgmath::vec2;

use crate::content;
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

fn distance(p1: TilePoint, p2: TilePoint) -> i32 {
    return (p1.x - p2.x).abs() + (p1.y - p2.y).abs();
}

#[derive(Debug)]
pub struct Actor {
    id: u32,
    pub actor_type: ActorType,
    pub position: TilePoint,
    ai_data: i32,
    pub is_dead: bool,
    pub max_hp: i32,
    pub current_hp: i32,
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

    pub fn create_actor(&mut self, actor_type: ActorType, position: TilePoint) -> u32 {
        let id = self.next_actor_id;
        let stats = content::get_base_stats(actor_type);
        self.actors.push(Actor {
            id,
            actor_type,
            position,
            ai_data: 0,
            is_dead: false,
            max_hp: stats.max_hp,
            current_hp: stats.max_hp,
        });
        self.next_actor_id += 1;
        id
    }

    pub fn create_player(&mut self, position: TilePoint) {
        self.create_actor(ActorType::Player, position);
        self.player_index = self.actors.len() - 1;
    }

    pub fn find_actors_at(&self, position: TilePoint, include_dead: bool) -> Vec<usize> {
        let mut result = vec![];
        for i in 0..self.actors.len() {
            if self.actors[i].position == position && (include_dead || !self.actors[i].is_dead) {
                result.push(i);
            }
        }
        result
    }

    pub fn get_player(&self) -> &Actor {
        &self.actors[self.player_index]
    }

    pub fn get_actor(&self, actor_id: u32) -> &Actor {
        self.actors.iter().find(|a| a.id == actor_id).expect("get_actor failed to find actor")
    }

    pub fn get_actor_mut(&mut self, actor_id: u32) -> &mut Actor {
        self.actors.iter_mut().find(|a| a.id == actor_id).expect("get_actor failed to find actor")
    }

    pub fn get_cell_type(&self, position: TilePoint) -> CellType {
        if position.x < 0 || position.y < 0 || position.x as usize >= self.size.x || position.y as usize >= self.size.y {
            return CellType::OutOfBounds;
        }
        self.cells[position.x as usize][position.y as usize].cell_type
    }

    fn modify_hp(&mut self, actor_index: usize, delta: i32) {
        let actor = &mut self.actors[actor_index];
        actor.current_hp += delta;
        if actor.current_hp > actor.max_hp {
            actor.current_hp = actor.max_hp;
        } else if actor.current_hp <= 0 {
            actor.is_dead = true;
        }
    }

    fn melee_attack(&mut self, attacker_index: usize, defender_index: usize) -> Vec<GameEvent> {
        self.modify_hp(defender_index, -1);
        let mut new_events = vec![
            GameEvent::MeleeAttack {
                attacker_id: self.actors[attacker_index].id,
                defender_id: self.actors[defender_index].id,
                damage: 1,
            }
        ];
        if self.actors[defender_index].is_dead {
            new_events.push(GameEvent::Death { actor_id: self.actors[defender_index].id });
        }
        new_events
    }

    fn run_monster_turn(&mut self, index: usize) -> Vec<GameEvent> {
        if self.actors[index].is_dead {
            return vec![];
        }
        let mut new_events = vec![];
        match self.actors[index].actor_type {
            ActorType::Player => (),
            ActorType::Toad => {
                let distance_to_player = distance(self.get_player().position, self.actors[index].position);
                if  distance_to_player == 1 {
                    new_events.append(&mut self.melee_attack(index, self.player_index));
                } else {
                    let walk_delta = match self.actors[index].ai_data {
                        0 => vec2(1, 0),
                        1 => vec2(0, 1),
                        2 => vec2(-1, 0),
                        3 => vec2(0, -1),
                        _ => unreachable!(),
                    };
                    self.actor_walk(index, walk_delta);
                    self.actors[index].ai_data += 1;
                    if self.actors[index].ai_data > 3 {
                        self.actors[index].ai_data = 0;
                    }
                }
            },
        }
        new_events
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
        if self.find_actors_at(next_position, false).len() > 0 {
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
    Wait,
    Walk { delta: TileDelta },
    Fight { delta: TileDelta },
}

pub struct GameInstance {
    pub turn: u32,
    pub current_room: Room,
    pub event_log: Vec<GameEvent>,
}

impl GameInstance {
    pub fn new() -> Self {
        GameInstance {
            turn: 0,
            current_room: create_blank_room(vec2(13, 9)),
            event_log: vec![],
        }
    }

    pub fn execute_command(&mut self, command: Command) {
        let turn_ended = match command {
            Command::Wait => true,
            Command::Walk { delta } => {
                let succeeded = self.current_room.actor_walk(self.current_room.player_index, delta);
                if !succeeded {
                    self.event_log.push(GameEvent::Bonk { actor_id: self.current_room.get_player().id });
                }
                succeeded
            }
            Command::Fight { delta } => {
                let attack_position = self.current_room.get_player().position + delta;
                let other_actors = self.current_room.find_actors_at(attack_position, false);
                if other_actors.len() > 0 {
                    let defender_index = other_actors[0];
                    self.event_log.append(&mut self.current_room.melee_attack(self.current_room.player_index, defender_index));
                }
                true
            },
        };
        if turn_ended {
            for i in 0..self.current_room.actors.len() {
                self.event_log.append(&mut self.current_room.run_monster_turn(i));
            }
            self.turn += 1;
        }
    }

    pub fn build_type_table(&self) -> HashMap<u32, ActorType> {
        let mut result = HashMap::new();
        for actor in self.current_room.actors.iter() {
            result.insert(actor.id, actor.actor_type);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance() {
        assert_eq!(0, distance(vec2(0, 0), vec2(0, 0)));
        assert_eq!(1, distance(vec2(1, 0), vec2(0, 0)));
        assert_eq!(4, distance(vec2(1, 0), vec2(4, 1)));
        assert_eq!(4, distance(vec2(1, 1), vec2(-1, -1)));
    }

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
        room.create_actor(ActorType::Toad, vec2(0, 0));
        room.create_player(vec2(1, 0));
        assert_eq!(room.actors.len(), 2);
        assert_ne!(room.actors[0].id, room.actors[1].id);
        assert_eq!(room.player_index, 1);
    }

    #[test]
    fn test_actor_walk() {
        let mut game = GameInstance::new();
        assert_eq!(0, game.turn);
        game.current_room.create_player(vec2(1, 1));

        game.execute_command(Command::Walk { delta: vec2(0, -1) });
        assert_eq!(game.current_room.get_player().position, vec2(1, 1));
        assert_eq!(vec![
            GameEvent::Bonk { actor_id: game.current_room.get_player().id },
        ], game.event_log);
        assert_eq!(0, game.turn);

        game.execute_command(Command::Walk { delta: vec2(0, 1) });
        assert_eq!(game.current_room.get_player().position, vec2(1, 2));
        assert_eq!(1, game.event_log.len()); // No new event
        assert_eq!(1, game.turn);
    }

    #[test]
    fn test_toad_monster_moves_in_circles() {
        let mut game = GameInstance::new();
        let monster_id = {
            let room = &mut game.current_room;
            room.create_player(vec2(4, 4));
            room.create_actor(ActorType::Toad, vec2(1, 1))
        };
        game.execute_command(Command::Wait);
        assert_eq!(game.current_room.get_actor(monster_id).position, vec2(2, 1));
        game.execute_command(Command::Wait);
        assert_eq!(game.current_room.get_actor(monster_id).position, vec2(2, 2));
        game.execute_command(Command::Wait);
        assert_eq!(game.current_room.get_actor(monster_id).position, vec2(1, 2));
        game.execute_command(Command::Wait);
        assert_eq!(game.current_room.get_actor(monster_id).position, vec2(1, 1));
    }

    #[test]
    fn test_toad_monster_fights_player() {
        let mut game = GameInstance::new();
        let (monster_id, player_id, player_max_hp, monster_max_hp) = {
            let room = &mut game.current_room;
            room.create_player(vec2(1, 1));
            let monster_id = room.create_actor(ActorType::Toad, vec2(2, 1));
            let player_id = room.get_player().id;
            let player_max_hp = room.get_player().current_hp;
            let monster_max_hp = room.get_actor(monster_id).current_hp;
            (monster_id, player_id, player_max_hp, monster_max_hp)
        };
        game.execute_command(Command::Fight { delta: vec2(1, 0) });
        assert!(game.current_room.get_player().current_hp < player_max_hp);
        assert!(game.current_room.get_actor(monster_id).current_hp < monster_max_hp);
        assert_eq!(vec![
            GameEvent::MeleeAttack { attacker_id: player_id, defender_id: monster_id, damage: 1 },
            GameEvent::MeleeAttack { attacker_id: monster_id, defender_id: player_id, damage: 1 },
        ], game.event_log);
    }

    #[test]
    fn test_toad_dies() {
        let mut game = GameInstance::new();
        let monster_id = {
            let room = &mut game.current_room;
            room.create_player(vec2(1, 1));
            let monster_id = room.create_actor(ActorType::Toad, vec2(2, 1));
            room.get_actor_mut(monster_id).current_hp = 1;
            monster_id
        };
        game.execute_command(Command::Fight { delta: vec2(1, 0) });
        assert!(game.current_room.get_actor(monster_id).is_dead);
        game.execute_command(Command::Walk { delta: vec2(1, 0) });
        assert_eq!(game.current_room.get_player().position, vec2(2, 1)); // Player can occupy that space now
        assert_eq!(game.current_room.get_actor(monster_id).position, vec2(2, 1)); // Dead monster shouldn't move
    }
}
