export const ActorBehavior = Object.freeze({
  PLAYER_INPUT: Symbol("PLAYER_INPUT"),
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
  swims;

  constructor(config) {
    this.display_name = config.display_name;
    this.attack_verb = config.attack_verb;
    this.behavior = config.behavior;
    this.max_hp = config.max_hp;
    this.starting_attack_power = config.starting_attack_power
    this.swims = !!config.swims;
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

});

export const ItemEffect = Object.freeze({
  HEAL: Symbol("HEAL"),
  ICE_DAMAGE: Symbol("ICE_DAMAGE"),
});

export const ItemActivateTargeting = Object.freeze({
  DIRECTION: Symbol("DIRECTION"),
});

export const EquippedSpecialEffect = Object.freeze({
  SWIMMING: Symbol("SWIMMING"),
});

class ItemTemplateEntry {
  display_name;
  equipment_slot;
  weapon_attack_verb;
  equipped_attack_power;
  equipped_special_effect;
  consume_effect;

  constructor(config) {
    this.display_name = config.display_name;
    this.equipment_slot = config.equipment_slot || null;
    this.weapon_attack_verb = config.weapon_attack_verb || null;
    this.equipped_attack_power = config.equipped_attack_power || 0;
    this.equipped_special_effect = config.equipped_special_effect || null;
    this.consume_effect = config.consume_effect || null;
    this.activate_effect = config.activate_effect || null;
    this.activate_targeting = config.activate_targeting || null;
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

  HEALING_HERB: new ItemTemplateEntry({
    display_name: "healing herb",
    consume_effect: ItemEffect.HEAL,
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

});
