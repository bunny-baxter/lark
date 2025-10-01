use crate::types::{ActorType, ItemType};

pub struct ActorBaseStats {
    pub max_hp: i32,
    pub attack_power: i32,
    pub defense_power: i32,
}

const PLAYER_BASE_STATS: ActorBaseStats = ActorBaseStats {
    max_hp: 10,
    attack_power: 1,
    defense_power: 0,
};

const TOAD_BASE_STATS: ActorBaseStats = ActorBaseStats {
    max_hp: 4,
    attack_power: 1,
    defense_power: 0,
};

pub fn get_base_stats(actor_type: ActorType) -> &'static ActorBaseStats {
    match actor_type {
        ActorType::Player => &PLAYER_BASE_STATS,
        ActorType::Toad => &TOAD_BASE_STATS,
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EquipSlot {
    Weapon,
    Headgear,
    Torso,
}

pub struct ItemData {
    pub equip_slot: EquipSlot,
    pub attack_bonus: Option<i32>,
    pub defense_bonus: Option<i32>,
}

const LUMP_OF_BLACKSTONE_DATA: ItemData = ItemData {
    equip_slot: EquipSlot::Weapon,
    attack_bonus: None,
    defense_bonus: None,
};

const BLACKSTONE_SPEAR_DATA: ItemData = ItemData {
    equip_slot: EquipSlot::Weapon,
    attack_bonus: Some(1),
    defense_bonus: None,
};

const CARMINE_CHAINMAIL_DATA: ItemData = ItemData {
    equip_slot: EquipSlot::Torso,
    attack_bonus: None,
    defense_bonus: Some(2),
};

const BLOODFLOWER_DATA: ItemData = ItemData {
    equip_slot: EquipSlot::Headgear,
    attack_bonus: None,
    defense_bonus: None,
};

pub fn get_item_data(item_type: ItemType) -> &'static ItemData {
    match item_type {
        ItemType::LumpOfBlackstone => &LUMP_OF_BLACKSTONE_DATA,
        ItemType::BlackstoneSpear => &BLACKSTONE_SPEAR_DATA,
        ItemType::CarmineChainmail => &CARMINE_CHAINMAIL_DATA,
        ItemType::Bloodflower => &BLOODFLOWER_DATA,
    }
}
