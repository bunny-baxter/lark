use cgmath::Vector2;

pub type TilePoint = Vector2<i32>;
pub type TileDelta = Vector2<i32>;
pub type TileSize = Vector2<usize>;

#[derive(Copy, Clone, Debug)]
pub enum ActorType {
    Player,
    Toad,
}

#[derive(Copy, Clone, Debug)]
pub enum ItemType {
    LumpOfBlackstone,
    //LumpOfWhitestone,
    BlackstoneSpear,
    //WhitestoneSpear,
    //CarmineSword,
    //ViridianSword,
    CarmineChainmail,
    //ViridianChainmail,
    //CarmineHelm,
    //ViridianHelm,
    //FeatheredCavalier,
    Bloodflower,
    //Azureberry,
    //Indigoberry,
    //PoulticeOfPurple,
    //MoonlightKnife,
    //SunlightKnife,
    //ProwessRing,
    //VoidwalkingRing,
    //WandOfIce,
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
}
