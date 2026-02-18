use std::collections:: {HashMap, HashSet};

use cgmath::vec2;
use rand::Rng;

use crate::data::{ActorType, CellType, ItemType, GameEvent, STEEL_THISTLE_CYCLE_MAX, TilePoint, TileDelta, TileSize, get_base_stats, get_item_data, ItemData, MiscEntityType};
use crate::{generate, generate::RoomGenerationConfig};

#[repr(C)]
#[derive(Clone, Debug, Default)]
pub struct Cell {
    cell_type: CellType,
}

fn distance(p1: TilePoint, p2: TilePoint) -> i32 {
    return (p1.x - p2.x).abs() + (p1.y - p2.y).abs();
}

// Orthogonal line algorithm, very similar to Bresenham's but only moves in cardinal directions
fn orthogonal_line(p1: TilePoint, p2: TilePoint) -> Vec<TilePoint> {
    let mut coordinates: Vec<TilePoint> = vec![];

    let dx = (p2.x - p1.x).abs();
    let dy = (p2.y - p1.y).abs();
    let sx = if p1.x < p2.x { 1 } else { -1 };
    let sy = if p1.y < p2.y { 1 } else { -1 };

    let mut current_x = p1.x;
    let mut current_y = p1.y;
    let mut error = dx - dy;

    loop {
        coordinates.push(vec2(current_x, current_y));

        if current_x == p2.x && current_y == p2.y {
            break;
        }

        // Move only in one direction per step
        if error > 0 {
            current_x += sx;
            error -= dy;
        } else {
            current_y += sy;
            error += dx;
        }
    }

    coordinates
}

#[derive(Clone, Debug)]
pub struct Actor {
    pub id: u32,
    pub actor_type: ActorType,
    pub position: TilePoint,
    ai_data: i32,
    skip_next_turn: bool,
    pub is_dead: bool,
    pub max_hp: i32,
    pub current_hp: i32,
    pub attack_power: i32,
    pub defense_power: i32,
}

#[derive(Clone, Debug)]
pub struct Item {
    pub id: u32,
    pub item_type: ItemType,
    pub position: TilePoint,
    carried: bool,
    pub equipped: bool,
    pub destroyed: bool,
}

#[derive(Clone, Debug)]
pub struct MiscEntity {
    #[allow(unused)]
    pub id: u32,
    pub entity_type: MiscEntityType,
    pub position: TilePoint,
    pub data: i32,
}

#[derive(Debug)]
pub struct Room {
    pub size: TileSize,
    pub cells: Vec<Vec<Cell>>,
    pub actors: Vec<Actor>,
    pub items: Vec<Item>,
    pub misc_entities: Vec<MiscEntity>,
    pub player_inventory: Vec<u32>,
    pub visible: HashSet<TilePoint>,
    pub explored: HashSet<TilePoint>,
    pub exits: HashMap<TilePoint, RoomGenerationConfig>,
    pub next_id: u32,
    pub player_index: usize,
}

struct WalkResult {
    succeeded: bool,
    events: Vec<GameEvent>,
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
            items: vec![],
            misc_entities: vec![],
            player_inventory: vec![],
            visible: HashSet::new(),
            explored: HashSet::new(),
            exits: HashMap::new(),
            next_id: 0,
            player_index: 0,
        }
    }

    fn generate(player_start: Option<TilePoint>, config: RoomGenerationConfig) -> Self {
        let mut room = Self::new(config.size);
        let gen_result = generate::generate_room(player_start, config.clone());
        for x in 0..config.size.x { for y in 0..config.size.y {
            let pos = vec2(x as i32, y as i32);
            let cell = &gen_result.cells[x][y];
            room.set_cell(pos, cell.cell_type);
            if let Some(monster_type) = cell.monster {
                room.create_actor(monster_type, pos);
            }
            if let Some(item_type) = cell.item {
                room.create_item(item_type, pos);
            }
            if let Some(entity_type) = cell.misc_entity {
                room.create_misc_entity(entity_type, pos);
            }
        }}
        for &exit in gen_result.exits.iter() {
            room.exits.insert(exit, config.clone());
        }
        if player_start.is_none() {
            room.create_player(gen_result.player_start);
        } // else change_rooms needs to clone the existing player
        room
    }

    pub fn set_cell(&mut self, position: TilePoint, cell_type: CellType) {
        self.cells[position.x as usize][position.y as usize].cell_type = cell_type;
    }

    pub fn create_actor(&mut self, actor_type: ActorType, position: TilePoint) -> u32 {
        let id = self.next_id;
        let stats = get_base_stats(actor_type);
        self.actors.push(Actor {
            id,
            actor_type,
            position,
            ai_data: 0,
            skip_next_turn: false,
            is_dead: false,
            max_hp: stats.max_hp,
            current_hp: stats.max_hp,
            attack_power: stats.attack_power,
            defense_power: stats.defense_power,
        });
        self.next_id += 1;
        id
    }

    pub fn create_player(&mut self, position: TilePoint) {
        self.create_actor(ActorType::Player, position);
        self.player_index = self.actors.len() - 1;
        self.update_visible_and_explored();
    }

    fn clone_actor(&mut self, other_actor: &Actor) -> u32 {
        let id = self.next_id;
        let mut new_actor = other_actor.clone();
        new_actor.id = id;
        self.actors.push(new_actor);
        if other_actor.actor_type == ActorType::Player {
            self.player_index = self.actors.len() - 1;
        }
        self.next_id += 1;
        id
    }

    pub fn create_item(&mut self, item_type: ItemType, position: TilePoint) -> u32 {
        let id = self.next_id;
        self.items.push(Item {
            id,
            item_type,
            position,
            carried: false,
            equipped: false,
            destroyed: false,
        });
        self.next_id += 1;
        id
    }

    fn clone_item(&mut self, other_item: &Item) -> u32 {
        let id = self.next_id;
        let mut new_item = other_item.clone();
        new_item.id = id;
        self.items.push(new_item);
        self.next_id += 1;
        id
    }

    pub fn create_misc_entity(&mut self, entity_type: MiscEntityType, position: TilePoint) -> u32 {
        let id = self.next_id;
        let mut entity = MiscEntity {
            id,
            entity_type,
            position,
            data: 0,
        };
        if entity_type == MiscEntityType::SteelThistle {
            let mut rng = rand::rng();
            entity.data = rng.random_range(0..=STEEL_THISTLE_CYCLE_MAX);
        }
        self.misc_entities.push(entity);
        self.next_id += 1;
        id
    }

    pub fn find_misc_entities_at(&self, position: TilePoint) -> Vec<usize> {
        let mut result = vec![];
        for i in 0..self.misc_entities.len() {
            if self.misc_entities[i].position == position {
                result.push(i);
            }
        }
        result
    }

    pub fn destroy_item(&mut self, item_id: u32) {
        self.get_item_mut(item_id).destroyed = true;
        self.player_inventory.swap_remove(self.player_inventory.iter().position(|&id| id == item_id).unwrap());
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

    pub fn find_loose_items_at(&self, position: TilePoint) -> Vec<usize> {
        let mut result = vec![];
        for i in 0..self.items.len() {
            if self.items[i].position == position && !self.items[i].carried && !self.items[i].destroyed {
                result.push(i);
            }
        }
        result
    }

    pub fn get_player(&self) -> &Actor {
        &self.actors[self.player_index]
    }

    pub fn get_player_mut(&mut self) -> &mut Actor {
        &mut self.actors[self.player_index]
    }

    #[allow(unused)]
    pub fn get_actor(&self, actor_id: u32) -> &Actor {
        self.actors.iter().find(|a| a.id == actor_id).expect("get_actor failed to find actor")
    }

    #[allow(unused)]
    pub fn get_actor_mut(&mut self, actor_id: u32) -> &mut Actor {
        self.actors.iter_mut().find(|a| a.id == actor_id).expect("get_actor failed to find actor")
    }

    pub fn get_item(&self, item_id: u32) -> &Item {
        self.items.iter().find(|i| i.id == item_id).expect("get_item failed to find item")
    }

    pub fn get_item_mut(&mut self, item_id: u32) -> &mut Item {
        self.items.iter_mut().find(|i| i.id == item_id).expect("get_item failed to find item")
    }

    pub fn get_item_data(&self, item_id: u32) -> &'static ItemData {
        get_item_data(self.get_item(item_id).item_type)
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
        let damage = (self.actors[attacker_index].attack_power - self.actors[defender_index].defense_power).max(0);
        self.modify_hp(defender_index, -damage);
        let mut new_events = vec![
            GameEvent::MeleeAttack {
                attacker_id: self.actors[attacker_index].id,
                defender_id: self.actors[defender_index].id,
                damage: damage,
            }
        ];
        if self.actors[defender_index].is_dead {
            new_events.push(GameEvent::Death { actor_id: self.actors[defender_index].id });
        }
        new_events
    }

    fn run_monster_turn(&mut self, index: usize) -> Vec<GameEvent> {
        if index == self.player_index {
            return vec![];
        }
        if self.actors[index].is_dead {
            return vec![];
        }
        if self.actors[index].skip_next_turn {
            self.actors[index].skip_next_turn = false;
            return vec![];
        }
        let mut new_events = vec![];
        match self.actors[index].actor_type {
            ActorType::Player => unreachable!(),
            ActorType::Toad | ActorType::ToothyStarling => {
                let distance_to_player = distance(self.get_player().position, self.actors[index].position);
                if  distance_to_player == 1 {
                    new_events.append(&mut self.melee_attack(index, self.player_index));
                } else {
                    const TOAD_PATROL_PATTERN: &[&[TileDelta]] = &[
                        &[vec2(1, 0)],
                        &[vec2(0, 1)],
                        &[vec2(-1, 0)],
                        &[vec2(0, -1)],
                    ];
                    // ..*..
                    // *.*.*
                    // .....
                    // .*.*.
                    const STARLING_PATROL_PATTERN: &[&[TileDelta]] = &[
                        &[vec2(0, 1), vec2(1, 0), vec2(1, 0)],
                        &[vec2(-1, 0), vec2(0, 1), vec2(0, 1)],
                        &[vec2(0, -1), vec2(0, -1), vec2(-1, 0)],
                        &[vec2(-1, 0), vec2(0, 1), vec2(0, 1)],
                        &[vec2(0, -1), vec2(0, -1), vec2(-1, 0)],
                        &[vec2(1, 0), vec2(1, 0), vec2(0, -1)],
                    ];
                    let patrol_pattern = match self.actors[index].actor_type {
                        ActorType::Toad => TOAD_PATROL_PATTERN,
                        ActorType::ToothyStarling => STARLING_PATROL_PATTERN,
                        _ => unreachable!(),
                    };
                    let deltas = patrol_pattern[self.actors[index].ai_data as usize];
                    for &delta in deltas.iter() {
                        self.actor_walk(index, delta);
                    }
                    self.actors[index].ai_data += 1;
                    if self.actors[index].ai_data >= patrol_pattern.len() as i32 {
                        self.actors[index].ai_data = 0;
                    }
                }
            },
            ActorType::MouseWarrior | ActorType::BlueJelly => {
                let should_act = match self.actors[index].actor_type {
                    ActorType::BlueJelly => {
                        let acts_this_turn = self.actors[index].ai_data % 2 == 0;
                        self.actors[index].ai_data += 1;
                        acts_this_turn
                    },
                    _ => true,
                };
                if should_act {
                    let actor_pos = self.actors[index].position;
                    let player_pos = self.get_player().position;
                    let distance_to_player = distance(player_pos, actor_pos);
                    if  distance_to_player == 1 {
                        new_events.append(&mut self.melee_attack(index, self.player_index));
                    } else {
                        let dx = player_pos.x - actor_pos.x;
                        let dy = player_pos.y - actor_pos.y;
                        let delta = if dx.abs() > dy.abs() {
                            vec2(dx.signum(), 0)
                        } else {
                            vec2(0, dy.signum())
                        };
                        self.actor_walk(index, delta);
                    }
                }
            },
            ActorType::DustySkeleton => {
                let distance_to_player = distance(self.get_player().position, self.actors[index].position);
                if  distance_to_player == 1 {
                    new_events.append(&mut self.melee_attack(index, self.player_index));
                } else {
                    let delta = match self.actors[index].ai_data {
                        0 => vec2(1, 0),
                        1 => vec2(0, 1),
                        2 => vec2(-1, 0),
                        3 => vec2(0, -1),
                        _ => unreachable!(),
                    };
                    let walk_result = self.actor_walk(index, delta);
                    if !walk_result.succeeded {
                        // Hit a wall, turn 90 degrees clockwise
                        self.actors[index].ai_data = (self.actors[index].ai_data + 1) % 4;
                    }
                }
            },
        }
        new_events
    }

    fn update_misc_entity(&mut self, index: usize) -> Vec<GameEvent> {
        let mut events = vec![];
        match self.misc_entities[index].entity_type {
            MiscEntityType::SteelThistle => {
                self.misc_entities[index].data += 1;
                if self.misc_entities[index].data > STEEL_THISTLE_CYCLE_MAX {
                    self.misc_entities[index].data = 0;
                }
                match self.misc_entities[index].data {
                    0..STEEL_THISTLE_CYCLE_MAX => (),
                    STEEL_THISTLE_CYCLE_MAX => {
                        // Strike actors
                        for actor_index in self.find_actors_at(self.misc_entities[index].position, false) {
                            self.modify_hp(actor_index, -1);
                            events.push(GameEvent::SteelThistleHit { actor_id: self.actors[actor_index].id, damage: 1 });
                            if self.actors[actor_index].is_dead {
                                events.push(GameEvent::Death { actor_id: self.actors[actor_index].id });
                            }
                        }
                    },
                    _ => unreachable!(),
                }
            },
            _ => (),
        };
        events
    }

    fn update_visible_and_explored(&mut self) {
        self.visible.clear();

        let player_pos = self.get_player().position;
        for x in 0..(self.size.x as i32) {
            for y in 0..(self.size.y as i32) {
                let current = vec2(x, y);
                for p in orthogonal_line(player_pos, current).into_iter() {
                    self.visible.insert(p);
                    self.explored.insert(p);
                    let cell_type = self.get_cell_type(p);
                    match cell_type {
                        CellType::OutOfBounds | CellType::DefaultWall | CellType::RoomExit => break,
                        _ => (),
                    };
                }
            }
        }
    }

    fn teleport_actor(&mut self, actor_index: usize, new_position: TilePoint) -> Vec<GameEvent> {
        let mut events = vec![];
        self.actors[actor_index].position = new_position;
        if actor_index == self.player_index {
            for item in self.items.iter_mut() {
                if self.player_inventory.contains(&item.id) {
                    item.position = new_position;
                }
            }
            self.update_visible_and_explored();
        }
        let entered_cell_type = self.cells[new_position.x as usize][new_position.y as usize].cell_type;
        if entered_cell_type == CellType::Water {
            self.actors[actor_index].skip_next_turn = true;
            events.push(GameEvent::SlowedByWater { actor_id: self.actors[actor_index].id });
        }
        events
    }

    fn actor_walk(&mut self, actor_index: usize, delta: TileDelta) -> WalkResult {
        let next_position = self.actors[actor_index].position + delta;
        let next_cell_type = self.get_cell_type(next_position);
        match next_cell_type {
            CellType::DefaultWall => return WalkResult {
                succeeded: false,
                events: vec![ GameEvent::Bonk { actor_id: self.actors[actor_index].id } ],
            },
            CellType::OutOfBounds => return WalkResult {
                succeeded: false,
                events: vec![],
            },
            _ => {},
        };
        if self.find_actors_at(next_position, false).len() > 0 {
            return WalkResult {
                succeeded: false,
                events: vec![],
            };
        }
        let events = self.teleport_actor(actor_index, next_position);
        WalkResult {
            succeeded: true,
            events: events,
        }
    }

    fn equip_item(&mut self, item_id: u32) -> Vec<GameEvent> {
        let mut events = vec![];
        let item_data = self.get_item_data(item_id);

        for &other_item_id in self.player_inventory.iter() {
            let other_item = self.get_item(other_item_id);
            if other_item.equipped && get_item_data(other_item.item_type).equip_slot == item_data.equip_slot {
                events.push(self.unequip_item(other_item_id));
                break;
            }
        }

        self.get_item_mut(item_id).equipped = true;
        if let Some(attack_bonus) = item_data.attack_bonus {
            self.get_player_mut().attack_power += attack_bonus;
        }
        if let Some(defense_bonus) = item_data.defense_bonus {
            self.get_player_mut().defense_power += defense_bonus;
        }

        events.push(GameEvent::EquippedItem { item_id });
        events
    }

    fn unequip_item(&mut self, item_id: u32) -> GameEvent {
        self.get_item_mut(item_id).equipped = false;
        let item_data = self.get_item_data(item_id);
        if let Some(attack_bonus) = item_data.attack_bonus {
            self.get_player_mut().attack_power -= attack_bonus;
        }
        if let Some(defense_bonus) = item_data.defense_bonus {
            self.get_player_mut().defense_power -= defense_bonus;
        }
        GameEvent::UnequippedItem { item_id }
    }

    fn eat_item(&mut self, item_id: u32) -> Vec<GameEvent> {
        let mut events = vec![ GameEvent::AteItem { item_id } ];
        match self.get_item(item_id).item_type {
            ItemType::Bloodflower => {
                self.modify_hp(self.player_index, 16);
                events.push(GameEvent::EffectHealed { actor_id: self.get_player().id });
                self.destroy_item(item_id);
            },
            _ => return vec![ GameEvent::ItemNotEdible { item_id } ],
        };
        events
    }

    fn apply_item_to_actor(&mut self, item_id: u32, actor_index: usize) -> Vec<GameEvent> {
        match self.get_item(item_id).item_type {
            ItemType::WandOfIce => {
                let damage = 4;
                self.modify_hp(actor_index, -damage);
                let mut new_events = vec![];
                new_events.push(GameEvent::EffectIceDamage { actor_id: self.actors[actor_index].id, damage });
                if self.actors[actor_index].is_dead {
                    new_events.push(GameEvent::Death { actor_id: self.actors[actor_index].id });
                }
                new_events
            },
            ItemType::LumpOfBlackstone => {
                let damage = 1;
                self.modify_hp(actor_index, -damage);
                let mut new_events = vec![];
                new_events.push(GameEvent::ThrownStoneDamage { actor_id: self.actors[actor_index].id, damage });
                if self.actors[actor_index].is_dead {
                    new_events.push(GameEvent::Death { actor_id: self.actors[actor_index].id });
                }
                new_events
            },
            _ => unreachable!(),
        }
    }

    fn activate_item_by_direction(&mut self, item_id: u32, direction: TileDelta) -> Vec<GameEvent> {
        assert_eq!(1, (direction.x + direction.y).abs());
        let mut events = vec![ GameEvent::ActivatedItem { item_id } ];

        let item_type = self.get_item(item_id).item_type;
        if item_type != ItemType::WandOfIce && item_type != ItemType::LumpOfBlackstone {
            events.push(GameEvent::NoEffect { item_id });
            return events;
        }

        let mut current_position = self.get_player().position;
        loop {
            current_position += direction;
            let other_actors = self.find_actors_at(current_position, false);
            if other_actors.len() > 0 {
                events.append(&mut self.apply_item_to_actor(item_id, other_actors[0]));
                break;
            }
            let cell_type = self.get_cell_type(current_position);
            match cell_type {
                CellType::DefaultWall | CellType::OutOfBounds => break,
                _ => {},
            }
        }
        self.destroy_item(item_id);

        events
    }
}

fn create_blank_room(size: TileSize) -> Room {
    let mut room = Room::new(size);
    for x in 0..size.x {
        for y in 0..size.y {
            if x == 0 || y == 0 || x as usize == size.x - 1 || y as usize == size.y - 1 {
                room.cells[x][y].cell_type = CellType::DefaultWall;
            } else {
                room.cells[x][y].cell_type = CellType::DefaultFloor;
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
    GetItem { item_id: u32 },
    DropItem { item_id: u32 },
    ToggleEquipment { item_id: u32 },
    EatItem { item_id: u32 },
    ActivateItemByDirection { item_id: u32, direction: TileDelta },
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
            // Placeholder room used only in tests
            current_room: create_blank_room(vec2(8, 8)),
            event_log: vec![],
        }
    }

    pub fn create_first_room(&mut self) {
        self.current_room = Room::generate(None, RoomGenerationConfig { size: vec2(19, 11) });
    }

    fn change_rooms(&mut self, player_start: TilePoint) {
        // player_start is for the next room, it may not be valid for the current room.
        let player_pos = self.current_room.get_player().position;
        let config = self.current_room.exits.get(&player_pos)
            .expect("change_rooms called but player not on exit");

        let mut new_room = Room::generate(Some(player_start), config.clone());

        new_room.clone_actor(self.current_room.get_player());
        let mut new_inventory = vec![];
        for &item_id in self.current_room.player_inventory.iter() {
            new_inventory.push(new_room.clone_item(self.current_room.get_item(item_id)));
        }
        new_room.player_inventory = new_inventory;
        new_room.teleport_actor(new_room.player_index, player_start);

        self.current_room = new_room;
    }

    pub fn execute_command(&mut self, command: Command) {
        let turn_ended = match command {
            Command::Wait => true,
            Command::Walk { delta } => {
                let previous_pos = self.current_room.get_player().position;
                let mut result = self.current_room.actor_walk(self.current_room.player_index, delta);
                self.event_log.append(&mut result.events);
                if result.succeeded {
                    if self.current_room.exits.contains_key(&self.current_room.get_player().position) {
                        // Use the player's previous position as the next room start. Otherwise the
                        // player could be on the room edge, which is never valid.
                        self.change_rooms(previous_pos);
                    }
                }
                result.succeeded
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
            Command::GetItem { item_id } => {
                self.current_room.get_item_mut(item_id).carried = true;
                self.current_room.player_inventory.push(item_id);
                self.event_log.push(GameEvent::GotItem { item_id });
                true
            },
            Command::DropItem { item_id } => {
                if self.current_room.get_item(item_id).equipped {
                    self.event_log.push(self.current_room.unequip_item(item_id));
                }
                self.current_room.get_item_mut(item_id).carried = false;
                self.current_room.player_inventory.swap_remove(self.current_room.player_inventory.iter().position(|&id| id == item_id).unwrap());
                self.event_log.push(GameEvent::DroppedItem { item_id });
                true
            },
            Command::ToggleEquipment { item_id } => {
                if self.current_room.get_item(item_id).equipped {
                    self.event_log.push(self.current_room.unequip_item(item_id));
                } else {
                    self.event_log.append(&mut self.current_room.equip_item(item_id));
                }
                true
            },
            Command::EatItem { item_id } => {
                self.event_log.append(&mut self.current_room.eat_item(item_id));
                true
            },
            Command::ActivateItemByDirection { item_id, direction } => {
                self.event_log.append(&mut self.current_room.activate_item_by_direction(item_id, direction));
                true
            },
        };
        if turn_ended {
            for i in 0..self.current_room.actors.len() {
                self.event_log.append(&mut self.current_room.run_monster_turn(i));
            }
            for i in 0..self.current_room.misc_entities.len() {
                self.event_log.append(&mut self.current_room.update_misc_entity(i));
            }
            self.turn += 1;
            if self.current_room.get_player().skip_next_turn {
                self.current_room.get_player_mut().skip_next_turn = false;
                self.execute_command(Command::Wait);
            }
        }
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

    #[test]
    fn test_pick_up_and_drop_item() {
        let mut game = GameInstance::new();
        let item_id = {
            let room = &mut game.current_room;
            room.create_player(vec2(1, 1));
            let item_id = room.create_item(ItemType::LumpOfBlackstone, vec2(1, 1));
            item_id
        };
        assert!(!game.current_room.get_item(item_id).carried);
        assert_eq!(0, game.current_room.player_inventory.len());
        assert_eq!(vec![0], game.current_room.find_loose_items_at(vec2(1, 1)));

        game.execute_command(Command::GetItem { item_id });
        assert!(game.current_room.get_item(item_id).carried);
        assert_eq!(vec![item_id], game.current_room.player_inventory);
        assert_eq!(0, game.current_room.find_loose_items_at(vec2(1, 1)).len());

        game.execute_command(Command::Walk { delta: vec2(1, 0) });
        // Item moves with player
        assert_eq!(vec2(2, 1), game.current_room.get_item(item_id).position);

        game.execute_command(Command::DropItem { item_id });
        assert!(!game.current_room.get_item(item_id).carried);
        assert_eq!(0, game.current_room.player_inventory.len());

        game.execute_command(Command::Walk { delta: vec2(-1, 0) });
        // Item remains where dropped
        assert_eq!(vec2(2, 1), game.current_room.get_item(item_id).position);
    }

    #[test]
    fn test_pick_up_and_drop_events() {
        let mut game = GameInstance::new();
        let item_id = {
            let room = &mut game.current_room;
            room.create_player(vec2(1, 1));
            let item_id = room.create_item(ItemType::LumpOfBlackstone, vec2(1, 1));
            item_id
        };
        game.execute_command(Command::GetItem { item_id });
        game.execute_command(Command::DropItem { item_id });
        assert_eq!(vec![
            GameEvent::GotItem { item_id },
            GameEvent::DroppedItem { item_id },
        ], game.event_log);
    }

    #[test]
    fn test_wield_spear() {
        let mut game = GameInstance::new();
        let item_id = {
            let room = &mut game.current_room;
            room.create_player(vec2(1, 1));
            let item_id = room.create_item(ItemType::BlackstoneSpear, vec2(1, 1));
            item_id
        };
        let attack_power_pre = game.current_room.get_player().attack_power;
        game.execute_command(Command::GetItem { item_id });
        game.execute_command(Command::ToggleEquipment { item_id });
        assert!(game.current_room.get_player().attack_power > attack_power_pre);

        game.execute_command(Command::ToggleEquipment { item_id });
        assert!(game.current_room.get_player().attack_power == attack_power_pre);

        assert_eq!(vec![
            GameEvent::GotItem { item_id },
            GameEvent::EquippedItem { item_id },
            GameEvent::UnequippedItem { item_id },
        ], game.event_log);
    }

    #[test]
    fn test_swap_equipment() {
        let mut game = GameInstance::new();
        let (id1, id2) = {
            let room = &mut game.current_room;
            room.create_player(vec2(1, 1));
            let item_id1 = room.create_item(ItemType::BlackstoneSpear, vec2(1, 1));
            let item_id2 = room.create_item(ItemType::BlackstoneSpear, vec2(1, 1));
            (item_id1, item_id2)
        };
        game.execute_command(Command::GetItem { item_id: id1 });
        game.execute_command(Command::GetItem { item_id: id2 });
        game.execute_command(Command::ToggleEquipment { item_id: id1 });
        game.execute_command(Command::ToggleEquipment { item_id: id2 });
        assert!(!game.current_room.get_item(id1).equipped);
        assert!(game.current_room.get_item(id2).equipped);

        assert_eq!(vec![
            GameEvent::GotItem { item_id: id1 },
            GameEvent::GotItem { item_id: id2 },
            GameEvent::EquippedItem { item_id: id1 },
            GameEvent::UnequippedItem { item_id: id1 },
            GameEvent::EquippedItem { item_id: id2 },
        ], game.event_log);
    }

    #[test]
    fn test_drop_equipped_item() {
        let mut game = GameInstance::new();
        let item_id = {
            let room = &mut game.current_room;
            room.create_player(vec2(1, 1));
            let item_id = room.create_item(ItemType::BlackstoneSpear, vec2(1, 1));
            item_id
        };
        let attack_power_pre = game.current_room.get_player().attack_power;
        game.execute_command(Command::GetItem { item_id });
        game.execute_command(Command::ToggleEquipment { item_id });
        game.execute_command(Command::DropItem { item_id });
        assert!(game.current_room.get_player().attack_power == attack_power_pre);

        assert_eq!(vec![
            GameEvent::GotItem { item_id },
            GameEvent::EquippedItem { item_id },
            GameEvent::UnequippedItem { item_id },
            GameEvent::DroppedItem { item_id },
        ], game.event_log);
    }

    #[test]
    fn test_eat_bloodflower() {
        let mut game = GameInstance::new();
        let item_id = {
            let room = &mut game.current_room;
            room.create_player(vec2(1, 1));
            room.get_player_mut().current_hp = 1;
            let item_id = room.create_item(ItemType::Bloodflower, vec2(1, 1));
            item_id
        };
        game.execute_command(Command::GetItem { item_id });
        game.execute_command(Command::EatItem { item_id });
        assert_eq!(game.current_room.get_player().max_hp, game.current_room.get_player().current_hp);
        assert_eq!(0, game.current_room.player_inventory.len());
        assert_eq!(0, game.current_room.find_loose_items_at(vec2(1, 1)).len());
        assert_eq!(vec![
            GameEvent::GotItem { item_id },
            GameEvent::AteItem { item_id },
            GameEvent::EffectHealed { actor_id: game.current_room.get_player().id },
        ], game.event_log);
    }

    #[test]
    fn test_cant_eat_rock() {
        let mut game = GameInstance::new();
        let item_id = {
            let room = &mut game.current_room;
            room.create_player(vec2(1, 1));
            room.get_player_mut().current_hp = 1;
            let item_id = room.create_item(ItemType::LumpOfBlackstone, vec2(1, 1));
            item_id
        };
        game.execute_command(Command::GetItem { item_id });
        game.execute_command(Command::EatItem { item_id });
        assert_eq!(1, game.current_room.get_player().current_hp);
        assert_eq!(vec![ item_id ], game.current_room.player_inventory);
        assert_eq!(vec![
            GameEvent::GotItem { item_id },
            GameEvent::ItemNotEdible { item_id },
        ], game.event_log);
    }

    #[test]
    fn test_slowed_by_water() {
        let mut game = GameInstance::new();
        {
            let room = &mut game.current_room;
            room.create_player(vec2(1, 1));
            room.set_cell(vec2(2, 1), CellType::Water);
            room.set_cell(vec2(3, 1), CellType::Water);
        }
        game.execute_command(Command::Walk { delta: vec2(1, 0) });
        assert_eq!(2, game.turn);
        game.execute_command(Command::Walk { delta: vec2(1, 0) });
        assert_eq!(4, game.turn);
        game.execute_command(Command::Walk { delta: vec2(1, 0) });
        assert_eq!(5, game.turn);

        let player_id = game.current_room.get_player().id;
        assert_eq!(vec![
            GameEvent::SlowedByWater { actor_id: player_id },
            GameEvent::SlowedByWater { actor_id: player_id },
        ], game.event_log);
    }

    #[test]
    fn test_hit_monster_with_wand() {
        let mut game = GameInstance::new();
        let (item_id, monster_id) = {
            let room = &mut game.current_room;
            room.create_player(vec2(1, 1));
            let item_id = room.create_item(ItemType::WandOfIce, vec2(1, 1));
            let monster_id = room.create_actor(ActorType::Toad, vec2(3, 1));
            (item_id, monster_id)
        };
        let monster_max_hp = game.current_room.get_actor(monster_id).max_hp;
        game.execute_command(Command::GetItem { item_id });
        game.execute_command(Command::ActivateItemByDirection { item_id, direction: vec2(1, 0) });
        assert!(game.current_room.get_actor(monster_id).current_hp < monster_max_hp);

        assert_eq!(vec![
            GameEvent::GotItem { item_id },
            GameEvent::ActivatedItem { item_id },
            GameEvent::EffectIceDamage { actor_id: monster_id, damage: 4 },
            GameEvent::Death { actor_id: monster_id },
        ], game.event_log);
    }

    #[test]
    fn test_wand_beam_ends_at_wall() {
        let mut game = GameInstance::new();
        let (item_id, monster_id) = {
            let room = &mut game.current_room;
            room.create_player(vec2(1, 1));
            room.set_cell(vec2(2, 1), CellType::DefaultWall);
            let item_id = room.create_item(ItemType::WandOfIce, vec2(1, 1));
            let monster_id = room.create_actor(ActorType::Toad, vec2(3, 1));
            (item_id, monster_id)
        };
        let monster_max_hp = game.current_room.get_actor(monster_id).max_hp;
        game.execute_command(Command::GetItem { item_id });
        game.execute_command(Command::ActivateItemByDirection { item_id, direction: vec2(1, 0) });
        assert!(game.current_room.get_actor(monster_id).current_hp == monster_max_hp);

        assert_eq!(vec![
            GameEvent::GotItem { item_id },
            GameEvent::ActivatedItem { item_id },
        ], game.event_log);
    }
}
