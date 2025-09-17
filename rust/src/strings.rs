use std::collections::HashMap;

use crate::types::{ActorType, GameEvent};

fn actor_type_to_name(actor_type: ActorType) -> &'static str {
    match actor_type {
        ActorType::Player => "player",
        ActorType::Toad => "mortal toad",
    }
}

fn get_actor_name<'a, 'b>(actor_id: u32, player_name: &'a str, type_table: &'b HashMap<u32, ActorType>) -> &'a str {
    match type_table.get(&actor_id) {
        Some(ActorType::Player) => player_name,
        Some(&actor_type) => actor_type_to_name(actor_type),
        None => "unknown actor",
    }
}

pub fn get_string(event: GameEvent, player_name: &str, type_table: &HashMap<u32, ActorType>) -> String {
    match event {
        GameEvent::Bonk { actor_id } => format!("{} bonks the wall", get_actor_name(actor_id, player_name, type_table)),
        GameEvent::MeleeAttack { attacker_id, defender_id, damage } => {
            format!("{} \u{2020}{} {}", get_actor_name(attacker_id, player_name, type_table), damage, get_actor_name(defender_id, player_name, type_table))
        },
    }
}
