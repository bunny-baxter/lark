export const ActorBehavior = Object.freeze({
  PLAYER_INPUT: Symbol("PLAYER_INPUT"),
  PASSIVE: Symbol("PASSIVE"),
  PATROL_VERTICALLY: Symbol("PATROL_VERTICALLY"),
  INFLICT_DAZZLE: Symbol("INFLICT_DAZZLE"),
  APPROACH_WHEN_NEAR: Symbol("APPROACH_WHEN_NEAR"),
});

class ActorTemplateEntry {
  display_name;
  attack_verb;
  behavior;
  max_hp;
  starting_attack_power;
  starting_defense;
  swims;
  basic_harvest_item;
  magic_harvest_item;

  constructor(config) {
    this.display_name = config.display_name;
    this.attack_verb = config.attack_verb;
    this.behavior = config.behavior;
    this.max_hp = config.max_hp;
    this.starting_attack_power = config.starting_attack_power
    this.starting_defense = config.starting_defense || 0;
    this.swims = !!config.swims;
    this.basic_harvest_item = config.basic_harvest_item;
    this.magic_harvest_item = config.magic_harvest_item;
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

  MERMAID: new ActorTemplateEntry({
    display_name: "mermaid",
    attack_verb: "slaps",
    behavior: ActorBehavior.APPROACH_WHEN_NEAR,
    max_hp: 6,
    starting_attack_power: 2,
    swims: true,
  }),

  BERRY_SHRUB: new ActorTemplateEntry({
    display_name: "darkberry shrub",
    behavior: ActorBehavior.PASSIVE,
    max_hp: 6,
    starting_defense: 1,
    basic_harvest_item: "DARKBERRY-3",
    magic_harvest_item: "DARKBERRY-5",
  }),

});

export const ItemEffect = Object.freeze({
  BASIC_HARVEST: Symbol("BASIC_HARVEST"),
  MAGIC_HARVEST: Symbol("MAGIC_HARVEST"),
  HEAL: Symbol("HEAL"),
  HEAL_FOOD: Symbol("HEAL_FOOD"),
  ICE_DAMAGE: Symbol("ICE_DAMAGE"),
});

export const ItemActivateTargeting = Object.freeze({
  DIRECTION: Symbol("DIRECTION"),
});

export const ItemActivateRange = Object.freeze({
  ADJACENT: Symbol("ADJACENT"),
  INFINITE: Symbol("INFINITE"),
});

export const EquippedSpecialEffect = Object.freeze({
  SWIMMING: Symbol("SWIMMING"),
});

class ItemTemplateEntry {
  display_name;
  equipment_slot;
  weapon_attack_verb;
  equipped_attack_power;
  equipped_defense;
  equipped_special_effect;
  consume_effect;
  activate_effect;
  activate_targeting;
  activate_range;
  activate_charges;

  constructor(config) {
    this.display_name = config.display_name;
    this.equipment_slot = config.equipment_slot || null;
    this.weapon_attack_verb = config.weapon_attack_verb || null;
    this.equipped_attack_power = config.equipped_attack_power || 0;
    this.equipped_defense = config.equipped_defense || 0;
    this.equipped_special_effect = config.equipped_special_effect || null;
    this.consume_effect = config.consume_effect || null;
    this.activate_effect = config.activate_effect || null;
    this.activate_targeting = config.activate_targeting || null;
    this.activate_range = config.activate_range || ItemActivateRange.INFINITE;
    this.activate_charges = config.activate_charges || -1;
  }
}

export const ItemTemplate = Object.freeze({

  ORDINARY_STONE: new ItemTemplateEntry({
    display_name: "ordinary stone",
  }),

  ORDINARY_SWORD: new ItemTemplateEntry({
    display_name: "steel sword",
    equipment_slot: "weapon",
    weapon_attack_verb: "slashes",
    equipped_attack_power: 2,
  }),

  POWERFUL_SWORD: new ItemTemplateEntry({
    display_name: "starmetal sword",
    equipment_slot: "weapon",
    weapon_attack_verb: "slashes",
    equipped_attack_power: 4,
  }),

  ORDINARY_CHAINMAIL: new ItemTemplateEntry({
    display_name: "steel chainmail",
    equipment_slot: "body",
    equipped_defense: 2,
  }),

  HEALING_HERB: new ItemTemplateEntry({
    display_name: "healing herb",
    consume_effect: ItemEffect.HEAL,
  }),

  DARKBERRY: new ItemTemplateEntry({
    display_name: "darkberry",
    consume_effect: ItemEffect.HEAL_FOOD,
  }),

  SWIMMING_RING: new ItemTemplateEntry({
    display_name: "ring of swimming",
    equipment_slot: "ring",
    equipped_special_effect: EquippedSpecialEffect.SWIMMING,
  }),

  FENCING_RING: new ItemTemplateEntry({
    display_name: "ring of fencing",
    equipment_slot: "ring",
    equipped_attack_power: 1,
  }),

  ICE_WAND: new ItemTemplateEntry({
    display_name: "wand of freezing",
    activate_effect: ItemEffect.ICE_DAMAGE,
    activate_targeting: ItemActivateTargeting.DIRECTION,
    activate_charges: 5,
  }),

  STEEL_KNIFE: new ItemTemplateEntry({
    display_name: "steel knife",
    equipment_slot: "weapon",
    weapon_attack_verb: "stabs",
    equipped_attack_power: 1,
    activate_effect: ItemEffect.BASIC_HARVEST,
    activate_targeting: ItemActivateTargeting.DIRECTION,
    activate_range: ItemActivateRange.ADJACENT,
  }),

  SILVER_KNIFE: new ItemTemplateEntry({
    display_name: "silver knife",
    equipment_slot: "weapon",
    weapon_attack_verb: "stabs",
    equipped_attack_power: 1,
    activate_effect: ItemEffect.MAGIC_HARVEST,
    activate_targeting: ItemActivateTargeting.DIRECTION,
    activate_range: ItemActivateRange.ADJACENT,
  }),

});
