import * as Util from './util.js';

export function flower_hit(hit_actor_name) {
  return `The flower stings ${hit_actor_name}.`;
}

export function fight(attacker_name, attack_verb, defender_name) {
  return `${Util.capitalize(attacker_name)} ${attack_verb} ${defender_name}.`;
}
