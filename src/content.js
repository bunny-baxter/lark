export const ActorBehavior = Object.freeze({
  PLAYER_INPUT: Symbol("PLAYER_INPUT"),
  PATROL_VERTICALLY: Symbol("PATROL_VERTICALLY"),
  INFLICT_DAZZLE: Symbol("INFLICT_DAZZLE"),
});

class ActorTemplateEntry {
  display_name;
  attack_verb;
  behavior;
  max_hp;
  starting_attack_power;

  constructor(config) {
    this.display_name = config.display_name;
    this.attack_verb = config.attack_verb;
    this.behavior = config.behavior;
    this.max_hp = config.max_hp;
    this.starting_attack_power = config.starting_attack_power
  }
}

export const ActorTemplate = Object.freeze({

  PLAYER: new ActorTemplateEntry({
    display_name: "Rogue",
    attack_verb: "punches",
    behavior: ActorBehavior.PLAYER_INPUT,
    max_hp: 12,
    starting_attack_power: 1,
  }),

  HERON: new ActorTemplateEntry({
    display_name: "heron",
    attack_verb: "pecks",
    behavior: ActorBehavior.PATROL_VERTICALLY,
    max_hp: 4,
    starting_attack_power: 1,
  }),

  STARLIGHT_FAIRY: new ActorTemplateEntry({
    display_name: "starlight fairy",
    attack_verb: "scratches",
    behavior: ActorBehavior.INFLICT_DAZZLE,
    max_hp: 5,
    starting_attack_power: 1,
  }),

});

class ItemTemplateEntry {
  display_name;
  equipment_slot;
  equipped_attack_power;

  constructor(config) {
    this.display_name = config.display_name;
    this.equipment_slot = config.equipment_slot || null;
    this.equipped_attack_power = config.equipped_attack_power || 0;
  }
}

export const ItemTemplate = Object.freeze({

  ORDINARY_STONE: new ItemTemplateEntry({
    display_name: "ordinary stone",
  }),

  ORDINARY_SWORD: new ItemTemplateEntry({
    display_name: "steel sword",
    equipment_slot: "weapon",
    equipped_attack_power: 2,
  }),

});

