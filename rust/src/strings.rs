use std::collections::HashMap;

use crate::content::{EquipSlot, get_item_data};
use crate::types::{ActorType, ItemType, GameEvent};

pub const EMPTY_INVENTORY: &str = "nothing is being carried";

pub enum NamedType {
    ActorType { actor_type: ActorType },
    ItemType { item_type: ItemType },
}

fn actor_type_to_name(actor_type: ActorType) -> &'static str {
    match actor_type {
        ActorType::Player => "((player))",
        ActorType::Toad => "mortal toad",
    }
}

pub fn item_type_to_name(item_type: ItemType) -> &'static str {
    match item_type {
        ItemType::LumpOfBlackstone => "lump of blackstone",
        ItemType::BlackstoneSpear => "blackstone spear",
        ItemType::CarmineChainmail => "carmine chainmail",
        ItemType::Bloodflower => "bloodflower",
    }
}

fn get_actor_name<'a, 'b>(actor_id: u32, player_name: &'a str, type_table: &'b HashMap<u32, NamedType>) -> &'a str {
    match type_table.get(&actor_id) {
        Some(NamedType::ActorType { actor_type: ActorType::Player }) => player_name,
        Some(NamedType::ActorType { actor_type }) => actor_type_to_name(*actor_type),
        Some(NamedType::ItemType { .. }) => "((item instead of actor))",
        None => "((unknown actor))",
    }
}

pub fn get_item_name(item_id: u32, type_table: &HashMap<u32, NamedType>) -> &'static str {
    match type_table.get(&item_id) {
        Some(NamedType::ItemType { item_type }) => item_type_to_name(*item_type),
        Some(NamedType::ActorType { .. }) => "((actor instead of item))",
        None => "((unknown item))",
    }
}

pub fn get_equipped_past_verb(item_id: u32, type_table: &HashMap<u32, NamedType>) -> &'static str {
    let item_type = type_table.get(&item_id);
    match item_type {
        Some(NamedType::ItemType { item_type }) => {
            let equip_slot = get_item_data(*item_type).equip_slot;
            match equip_slot {
                EquipSlot::Weapon => "wielded",
                EquipSlot::Headgear => "donned",
                EquipSlot::Torso => "put on",
            }
        },
        _ => "((unknown))",
    }
}

pub fn get_equipped_participle(item_id: u32, type_table: &HashMap<u32, NamedType>) -> &'static str {
    let item_type = type_table.get(&item_id);
    match item_type {
        Some(NamedType::ItemType { item_type }) => {
            let equip_slot = get_item_data(*item_type).equip_slot;
            match equip_slot {
                EquipSlot::Weapon => "wielded",
                _ => "worn",
            }
        },
        _ => "((unknown))",
    }
}

pub fn get_unequipped_past_verb(item_id: u32, type_table: &HashMap<u32, NamedType>) -> &'static str {
    let item_type = type_table.get(&item_id);
    match item_type {
        Some(NamedType::ItemType { item_type }) => {
            let equip_slot = get_item_data(*item_type).equip_slot;
            match equip_slot {
                EquipSlot::Weapon => "put away",
                EquipSlot::Headgear => "doffed",
                EquipSlot::Torso => "took off",
            }
        },
        _ => "((unknown))",
    }
}

pub fn get_string(event: GameEvent, player_name: &str, type_table: &HashMap<u32, NamedType>) -> String {
    match event {
        GameEvent::Bonk { actor_id } => format!("{} hits a wall", get_actor_name(actor_id, player_name, type_table)),
        GameEvent::MeleeAttack { attacker_id, defender_id, damage } => {
            format!("{} \u{2020}{} {}", get_actor_name(attacker_id, player_name, type_table), damage, get_actor_name(defender_id, player_name, type_table))
        },
        GameEvent::Death { actor_id } => format!("{} dies", get_actor_name(actor_id, player_name, type_table)),
        GameEvent::GotItem { item_id } => format!("got {}", get_item_name(item_id, type_table)),
        GameEvent::DroppedItem { item_id } => format!("dropped {}", get_item_name(item_id, type_table)),
        GameEvent::EquippedItem { item_id } => format!("{} {}", get_equipped_past_verb(item_id, type_table), get_item_name(item_id, type_table)),
        GameEvent::UnequippedItem { item_id } => format!("{} {}", get_unequipped_past_verb(item_id, type_table), get_item_name(item_id, type_table)),
        GameEvent::AteItem { item_id } => format!("ate {}", get_item_name(item_id, type_table)),
        GameEvent::ItemNotEdible { item_id } => format!("chewed on {}, ineffectually", get_item_name(item_id, type_table)),
        GameEvent::EffectHealed { actor_id } => format!("{} is healed", get_actor_name(actor_id, player_name, type_table)),
        GameEvent::SlowedByWater { actor_id } => format!("{} is slowed wading in the water", get_actor_name(actor_id, player_name, type_table)),
    }
}
