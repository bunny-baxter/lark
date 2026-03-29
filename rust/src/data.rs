use cgmath::Vector2;

pub type TilePoint = Vector2<i32>;
pub type TileDelta = Vector2<i32>;
pub type TileSize = Vector2<usize>;

pub const NEIGHBORS: &[(i32, i32)] = &[
    (-1, 0),
    (0, -1),
    (1, 0),
    (0, 1),
];

#[repr(C)]
#[allow(unused)]
#[derive(Copy, Clone, Default, Eq, PartialEq, Debug)]
pub enum CellType {
    OutOfBounds = -1,
    #[default]
    Empty = 0,
    DefaultFloor = 1,
    FloorMoss,
    FloorThyme,
    DefaultWall,
    RoomExit,
    Water,
}

#[allow(unused)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ActorType {
    Player,
    Toad,
    MouseWarrior,
    MouseSkirmisher,
    ToothyStarling,
    DustySkeleton,
    BlueJelly,
}

pub struct ActorBaseStats {
    pub max_hp: i32,
    pub attack_power: i32,
    pub defense_power: i32,
}

const PLAYER_STATS: ActorBaseStats = ActorBaseStats {
    max_hp: 24,
    attack_power: 2,
    defense_power: 0,
};

const TOAD_STATS: ActorBaseStats = ActorBaseStats {
    max_hp: 4,
    attack_power: 2,
    defense_power: 0,
};

const MOUSE_WARRIOR_STATS: ActorBaseStats = ActorBaseStats {
    max_hp: 16,
    attack_power: 5,
    defense_power: 2,
};

const MOUSE_SKIRMISHER_STATS: ActorBaseStats = ActorBaseStats {
    max_hp: 10,
    attack_power: 3,
    defense_power: 1,
};

const TOOTHY_STARLING_STATS: ActorBaseStats = ActorBaseStats {
    max_hp: 8,
    attack_power: 3,
    defense_power: 0,
};

const DUSTY_SKELETON_STATS: ActorBaseStats = ActorBaseStats {
    max_hp: 9,
    attack_power: 5,
    defense_power: 1,
};

const BLUE_JELLY_STATS: ActorBaseStats = ActorBaseStats {
    max_hp: 6,
    attack_power: 3,
    defense_power: 0,
};

pub fn get_base_stats(actor_type: ActorType) -> &'static ActorBaseStats {
    match actor_type {
        ActorType::Player => &PLAYER_STATS,
        ActorType::Toad => &TOAD_STATS,
        ActorType::MouseWarrior => &MOUSE_WARRIOR_STATS,
        ActorType::MouseSkirmisher => &MOUSE_SKIRMISHER_STATS,
        ActorType::ToothyStarling => &TOOTHY_STARLING_STATS,
        ActorType::DustySkeleton => &DUSTY_SKELETON_STATS,
        ActorType::BlueJelly => &BLUE_JELLY_STATS,
    }
}

#[allow(unused)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ItemType {
    LumpOfBlackstone,
    //LumpOfWhitestone,
    BlackstoneSpear,
    //WhitestoneSpear,
    CarmineSword,
    //ViridianSword,
    BoneLamellar,
    CarmineChainmail,
    //ViridianChainmail,
    CarmineHelm,
    //ViridianHelm,
    FeatheredCavalier,
    Bloodflower,
    //Azureberry,
    //Indigoberry,
    //PoulticeOfPurple,
    MoonlightKnife,
    //SunlightKnife,
    //ProwessRing,
    //VoidwalkingRing,
    WandOfIce,
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
    pub max_hp_bonus: Option<i32>,
    pub initial_wand_charges: Option<i32>,
}

const LUMP_OF_BLACKSTONE_DATA: ItemData = ItemData {
    equip_slot: EquipSlot::Weapon,
    attack_bonus: None,
    defense_bonus: None,
    max_hp_bonus: None,
    initial_wand_charges: None,
};

const BLACKSTONE_SPEAR_DATA: ItemData = ItemData {
    equip_slot: EquipSlot::Weapon,
    attack_bonus: Some(3),
    defense_bonus: None,
    max_hp_bonus: None,
    initial_wand_charges: None,
};

const CARMINE_CHAINMAIL_DATA: ItemData = ItemData {
    equip_slot: EquipSlot::Torso,
    attack_bonus: None,
    defense_bonus: Some(2),
    max_hp_bonus: None,
    initial_wand_charges: None,
};

const BLOODFLOWER_DATA: ItemData = ItemData {
    equip_slot: EquipSlot::Headgear,
    attack_bonus: None,
    defense_bonus: None,
    max_hp_bonus: None,
    initial_wand_charges: None,
};

const WAND_OF_ICE_DATA: ItemData = ItemData {
    equip_slot: EquipSlot::Weapon,
    attack_bonus: None,
    defense_bonus: None,
    max_hp_bonus: None,
    initial_wand_charges: Some(4),
};

const CARMINE_SWORD_DATA: ItemData = ItemData {
    equip_slot: EquipSlot::Weapon,
    attack_bonus: Some(5),
    defense_bonus: None,
    max_hp_bonus: None,
    initial_wand_charges: None,
};

const MOONLIGHT_KNIFE_DATA: ItemData = ItemData {
    equip_slot: EquipSlot::Weapon,
    attack_bonus: Some(1),
    defense_bonus: None,
    max_hp_bonus: None,
    initial_wand_charges: None,
};

const BONE_LAMELLAR_DATA: ItemData = ItemData {
    equip_slot: EquipSlot::Torso,
    attack_bonus: None,
    defense_bonus: Some(1),
    max_hp_bonus: None,
    initial_wand_charges: None,
};

const FEATHERED_CAVALIER_DATA: ItemData = ItemData {
    equip_slot: EquipSlot::Headgear,
    attack_bonus: None,
    defense_bonus: None,
    max_hp_bonus: Some(4),
    initial_wand_charges: None,
};

const CARMINE_HELM_DATA: ItemData = ItemData {
    equip_slot: EquipSlot::Headgear,
    attack_bonus: None,
    defense_bonus: None,
    max_hp_bonus: Some(8),
    initial_wand_charges: None,
};

pub fn get_item_data(item_type: ItemType) -> &'static ItemData {
    match item_type {
        ItemType::LumpOfBlackstone => &LUMP_OF_BLACKSTONE_DATA,
        ItemType::BlackstoneSpear => &BLACKSTONE_SPEAR_DATA,
        ItemType::CarmineSword => &CARMINE_SWORD_DATA,
        ItemType::MoonlightKnife => &MOONLIGHT_KNIFE_DATA,
        ItemType::BoneLamellar => &BONE_LAMELLAR_DATA,
        ItemType::FeatheredCavalier => &FEATHERED_CAVALIER_DATA,
        ItemType::CarmineHelm => &CARMINE_HELM_DATA,
        ItemType::CarmineChainmail => &CARMINE_CHAINMAIL_DATA,
        ItemType::Bloodflower => &BLOODFLOWER_DATA,
        ItemType::WandOfIce => &WAND_OF_ICE_DATA,
    }
}

pub const STEEL_THISTLE_CYCLE_MAX: i32 = 4;

#[allow(unused)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MiscEntityType {
    SteelThistle,
    TreasureChest,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GameEvent {
    Bonk { actor_id: u32 },
    MeleeAttack { attacker_id: u32, defender_id: u32, damage: i32 },
    Death { actor_id: u32 },
    GotItem { item_id: u32 },
    DroppedItem { item_id: u32 },
    EquippedItem { item_id: u32 },
    UnequippedItem { item_id: u32 },
    AteItem { item_id: u32 },
    ItemNotEdible { item_id: u32 },
    EffectHealed { actor_id: u32 },
    SlowedByWater { actor_id: u32 },
    ActivatedItem { item_id: u32 },
    EffectIceDamage { actor_id: u32, damage: i32 },
    NoEffect { item_id: u32 },
    SteelThistleHit { actor_id: u32, damage: i32 },
    ThrownStoneDamage { actor_id: u32, damage: i32 },
    JavelinDamage { actor_id: u32, damage: i32 },
    WandExpended { item_id: u32 },
    ItemIsHere { item_id: u32 },
    Winner,
}
