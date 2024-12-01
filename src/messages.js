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
  return `${Util.capitalize(actor_name)} ${verb} ${item_name}.`;
}
