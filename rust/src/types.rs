use cgmath::Vector2;

pub type TilePoint = Vector2<i32>;
pub type TileDelta = Vector2<i32>;
pub type TileSize = Vector2<usize>;

#[derive(Copy, Clone, Debug)]
pub enum ActorType {
    Player,
    Toad,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GameEvent {
    Bonk { actor_id: u32 },
    MeleeAttack { attacker_id: u32, defender_id: u32, damage: i32 },
}
