import * as Util from './util.js';

export function flower_hit(hit_actor_name) {
  return `The flower stings ${hit_actor_name}.`;
}

export function fight(attacker_name, attack_verb, defender_name) {
  return `${Util.capitalize(attacker_name)} ${attack_verb} ${defender_name}.`;
}

export function die(actor_name) {
  return `${Util.capitalize(actor_name)} dies.`;
}

export function get_item(actor_name, item_name) {
  return `${Util.capitalize(actor_name)} picks up ${item_name}.`;
}

export function drop_item(actor_name, item_name) {
  return `${Util.capitalize(actor_name)} drops ${item_name}.`;
}

export function equip_item(actor_name, item_name, equipment_slot) {
  let verb = "equips";
  if (equipment_slot === "weapon") {
    verb = "wields";
  }
  return `${Util.capitalize(actor_name)} ${verb} the ${item_name}.`;
}

export function unequip_item(actor_name, item_name, equipment_slot) {
  let verb = "removes";
  if (equipment_slot === "weapon") {
    verb = "puts away";
  }
  return `${Util.capitalize(actor_name)} ${verb} the ${item_name}.`;
}

export function consume_item_prefix(actor_name, item_name) {
  return `${Util.capitalize(actor_name)} eats the ${item_name}.`;
}

export function effect_heals(actor_name) {
  return `${Util.capitalize(actor_name)}'s hp is restored.`;
}

export function effect_cursed_herb(actor_name) {
  return `The curse drains ${actor_name}'s hp.`;
}

export function effect_gain_max_hp(actor_name) {
  return `${Util.capitalize(actor_name)} feels hearty.`;
}

export function fairy_inflict_dazzle(actor_name, target_name) {
  return `${Util.capitalize(actor_name)} dazzles ${target_name} with fairy lights.`;
}

export function dazzle_miss(actor_name, target_name) {
  return `${Util.capitalize(actor_name)} is distracted by fairy lights and misses ${target_name}.`;
}

export function dazzle_fades(actor_name) {
  return `The fairy lights fade away from ${actor_name}.`;
}
