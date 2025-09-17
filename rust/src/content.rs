use crate::types::ActorType;

pub struct ActorBaseStats {
    pub max_hp: i32,
}

const PLAYER_BASE_STATS: ActorBaseStats = ActorBaseStats {
    max_hp: 10,
};

const TOAD_BASE_STATS: ActorBaseStats = ActorBaseStats {
    max_hp: 4,
};

pub fn get_base_stats(actor_type: ActorType) -> &'static ActorBaseStats {
    match actor_type {
        ActorType::Player => &PLAYER_BASE_STATS,
        ActorType::Toad => &TOAD_BASE_STATS,
    }
}
