import {ActorBehavior, ActorTemplate, ConsumeItemEffect, ItemTemplate} from './content.js';
import * as Messages from './messages.js';
import * as Util from './util.js';

export const CellType = Object.freeze({
  OUT_OF_BOUNDS: Symbol("OUT_OF_BOUNDS"),
  EMPTY: Symbol("EMPTY"),
  FLOOR: Symbol("FLOOR"),
  DEFAULT_WALL: Symbol("DEFAULT_WALL"),
  FLOWER_HAZARD: Symbol("FLOWER_HAZARD"),
});

export const Phase = Object.freeze({
  IDLE: Symbol("IDLE"),
  READY: Symbol("READY"),
  ACTIVE: Symbol("ACTIVE"),
});

export const Condition = Object.freeze({
  DAZZLE: Symbol("DAZZLE"),
});

export const FLOWER_HAZARD_CYCLE_LENGTH = 5;
const HEALING_HERB_NEUTRAL_HEALING_AMOUNT = 20;
export const HEALING_HERB_BLESSED_MAX_HP_BONUS = 2;
export const HEALING_HERB_CURSED_DAMAGE_AMOUNT = 10;

export class CellData {
  type = CellType.EMPTY;

  turn_counter = 0;
  phase = null;
}

export class Actor {
  id;
  template;
  behavior_state = null;
  tile_x = 0;
  tile_y = 0;

  current_hp;
  attack_power;
  conditions = new Map();
  inventory = null;
  is_dead = false;

  constructor(id, template) {
    this.id = id;
    this.template = template;

    this.max_hp = template.max_hp;
    this.current_hp = template.max_hp;
    this.attack_power = template.starting_attack_power;
    // Only the player has an inventory for now.
    if (template === ActorTemplate.PLAYER) {
      this.inventory = [];
    }
  }

  has_condition(condition) {
    return this.conditions.has(condition);
  }

  add_condition(condition, turns) {
    if (!this.conditions.has(condition) || this.conditions.get(condition) < turns) {
      this.conditions.set(condition, turns);
    }
  }
}

export const Beatitude = Object.freeze({
  CURSED: Symbol("CURSED"),
  NEUTRAL: Symbol("NEUTRAL"),
  BLESSED: Symbol("BLESSED"),
});

export class Item {
  id;
  template;
  beatitude;

  tile_x = 0;
  tile_y = 0;
  is_destroyed = false;
  held_actor = null;
  equipped = false;

  constructor(id, template, beatitude) {
    this.id = id;
    this.template = template;
    this.beatitude = beatitude;
  }

  get_name() {
    let beatitude_prefix = "neutral";
    if (this.beatitude === Beatitude.CURSED) {
      beatitude_prefix = "cursed";
    } else if (this.beatitude === Beatitude.BLESSED) {
      beatitude_prefix = "blessed";
    }
    return `${beatitude_prefix} ${this.template.display_name}`
  }
}

export class Floor {
  parent_game;
  next_id = 0;
  size_tiles;
  cells;
  actors = [];
  items = [];
  player_ref = null;

  constructor(parent_game, width_tiles, height_tiles) {
    this.parent_game = parent_game;
    this.size_tiles = { w: width_tiles, h: height_tiles };
    this.cells = [];
    for (let x = 0; x < this.size_tiles.w; x++) {
      this.cells.push([]);
      for (let y = 0; y < this.size_tiles.h; y++) {
        const cell_data = new CellData();
        if (x === 0 || y === 0 || x === this.size_tiles.w - 1 || y === this.size_tiles.h - 1) {
          cell_data.type = CellType.DEFAULT_WALL;
        } else {
          cell_data.type = CellType.FLOOR;
        }
        this.cells[x].push(cell_data);
      }
    }
  }

  create_actor(template, x, y) {
    console.assert(template !== undefined);
    const actor = new Actor(this.next_id, template);
    actor.tile_x = x;
    actor.tile_y = y;
    this.next_id += 1;
    this.actors.push(actor);
    return actor;
  }

  create_player(x, y) {
    this.player_ref = this.create_actor(ActorTemplate.PLAYER, x, y);
  }

  _teleport_actor(actor_ref, to_x, to_y) {
    actor_ref.tile_x = to_x;
    actor_ref.tile_y = to_y;
    if (actor_ref.inventory) {
      for (const item_ref of actor_ref.inventory) {
        item_ref.tile_x = to_x;
        item_ref.tile_y = to_y;
      }
    }
  }

  actor_walk(actor_ref, delta_x, delta_y) {
    const next_x = actor_ref.tile_x + delta_x;
    const next_y = actor_ref.tile_y + delta_y;
    const next_cell_type = this.get_cell_type(next_x, next_y);
    if (next_cell_type === CellType.DEFAULT_WALL || next_cell_type === CellType.OUT_OF_BOUNDS) {
      return false;
    }
    if (this.find_actors_at(next_x, next_y).length > 0) {
      // Any actor in the space blocks movement.
      return false;
    }
    this._teleport_actor(actor_ref, next_x, next_y);
    return true;
  }

  _change_actor_hp(actor, delta) {
    actor.current_hp += delta;
    if (actor.current_hp <= 0) {
      actor.is_dead = true;
      Util.remove_first(this.actors, actor);
      this.parent_game.add_message(Messages.die(actor.template.display_name));
    } else if (actor.current_hp > actor.max_hp) {
      actor.current_hp = actor.max_hp;
    }
  }

  _actor_fight_actor(attacker_ref, defender_ref) {
    if (attacker_ref.has_condition(Condition.DAZZLE) && Util.rand_int(4) === 0) {
      this.parent_game.add_message(Messages.dazzle_miss(attacker_ref.template.display_name, defender_ref.template.display_name));
      return;
    }
    // TODO: Change message based on wielded weapon.
    this.parent_game.add_message(Messages.fight(
      attacker_ref.template.display_name, attacker_ref.template.attack_verb, defender_ref.template.display_name));
    this._change_actor_hp(defender_ref, -1 * attacker_ref.attack_power);
  }

  actor_fight(actor_ref, delta_x, delta_y) {
    const target_x = actor_ref.tile_x + delta_x;
    const target_y = actor_ref.tile_y + delta_y;
    for (const other_actor of this.find_actors_at(target_x, target_y)) {
      this._actor_fight_actor(actor_ref, other_actor);
    }
  }

  find_actors_at(tile_x, tile_y) {
    const result = [];
    for (const actor of this.actors) {
      if (actor.tile_x === tile_x && actor.tile_y === tile_y) {
        result.push(actor);
      }
    }
    return result;
  }

  find_loose_items_at(tile_x, tile_y) {
    const result = [];
    for (const item of this.items) {
      if (!item.held_actor && item.tile_x === tile_x && item.tile_y === tile_y) {
        result.push(item);
      }
    }
    return result;
  }

  create_item(template, beatitude, tile_x, tile_y) {
    console.assert(template !== undefined);
    const item = new Item(this.next_id, template, beatitude);
    item.tile_x = tile_x;
    item.tile_y = tile_y;
    this.next_id += 1;
    this.items.push(item);
    return item;
  }

  player_get_item(item_ref) {
    console.assert(item_ref.held_actor === null);
    console.assert(item_ref.tile_x === this.player_ref.tile_x && item_ref.tile_y === this.player_ref.tile_y);

    this.parent_game.add_message(Messages.get_item(this.player_ref.template.display_name, item_ref.get_name()));
    item_ref.held_actor = this.player_ref;
    this.player_ref.inventory.push(item_ref);
  }

  player_drop_item(item_ref) {
    console.assert(item_ref.held_actor === this.player_ref);

    this.parent_game.add_message(Messages.drop_item(this.player_ref.template.display_name, item_ref.get_name()));
    item_ref.held_actor = null;
    Util.remove_first(this.player_ref.inventory, item_ref);
  }

  player_toggle_equipment(item_ref) {
    console.assert(item_ref.held_actor === this.player_ref);

    if (item_ref.equipped) {
      this.parent_game.add_message(Messages.unequip_item(this.player_ref.template.display_name, item_ref.get_name(), item_ref.template.equipment_slot));
      item_ref.equipped = false;
      this.player_ref.attack_power -= item_ref.template.equipped_attack_power;
    } else {
      let old_item = null;
      for (const other_item of this.player_ref.inventory) {
        if (other_item.equipped && other_item.template.equipment_slot === item_ref.template.equipment_slot) {
          old_item = other_item;
          break;
        }
      }
      if (old_item) {
        this.player_toggle_equipment(old_item);
      }
      this.parent_game.add_message(Messages.equip_item(this.player_ref.template.display_name, item_ref.get_name(), item_ref.template.equipment_slot));
      item_ref.equipped = true;
      this.player_ref.attack_power += item_ref.template.equipped_attack_power;
    }
  }

  player_consume_item(item_ref) {
    console.assert(item_ref.held_actor === this.player_ref);
    console.assert(item_ref.template.consume_effect !== undefined);

    let message = Messages.consume_item_prefix(this.player_ref.template.display_name, item_ref.get_name());

    if (item_ref.template.consume_effect === ConsumeItemEffect.HEAL) {
      if (item_ref.beatitude === Beatitude.CURSED) {
        message += ` ${Messages.effect_cursed_herb(this.player_ref.template.display_name)}`;
        this._change_actor_hp(this.player_ref, -1 * HEALING_HERB_CURSED_DAMAGE_AMOUNT);
      } else {
        if (item_ref.beatitude === Beatitude.BLESSED) {
          this.player_ref.max_hp += HEALING_HERB_BLESSED_MAX_HP_BONUS;
        }
        message += ` ${Messages.effect_heals(this.player_ref.template.display_name)}`;
        this._change_actor_hp(this.player_ref, HEALING_HERB_NEUTRAL_HEALING_AMOUNT);
        if (item_ref.beatitude === Beatitude.BLESSED) {
          message += ` ${Messages.effect_gain_max_hp(this.player_ref.template.display_name)}`;
        }
      }
    }

    this.parent_game.add_message(message);
    item_ref.is_destroyed = true;
    Util.remove_first(this.items, item_ref);
    Util.remove_first(this.player_ref.inventory, item_ref);
  }

  is_out_of_bounds(x, y) {
    return x < 0 || y < 0 || x >= this.size_tiles.w || y >= this.size_tiles.h;
  }

  get_cell_type(x, y) {
    if (this.is_out_of_bounds(x, y)) {
      return CellType.OUT_OF_BOUNDS;
    }
    return this.cells[x][y].type;
  }

  set_cell(x, y, type) {
    if (this.is_out_of_bounds(x, y)) {
      return;
    }
    this.cells[x][y].type = type;

    if (type === CellType.FLOWER_HAZARD) {
      this.cells[x][y].phase = Phase.IDLE;
    }
  }

  _update_cell(x, y) {
    const cell_data = this.cells[x][y];

    if (cell_data.type === CellType.FLOWER_HAZARD) {
      cell_data.turn_counter += 1;
      const mod = cell_data.turn_counter % FLOWER_HAZARD_CYCLE_LENGTH;
      if (mod === FLOWER_HAZARD_CYCLE_LENGTH - 1) {
        cell_data.phase = Phase.ACTIVE;
        for (const actor of this.find_actors_at(x, y)) {
          this.parent_game.add_message(Messages.flower_hit(actor.template.display_name));
          this._change_actor_hp(actor, -1);
        }
      } else if (mod === FLOWER_HAZARD_CYCLE_LENGTH - 2) {
        cell_data.phase = Phase.READY;
      } else {
        cell_data.phase = Phase.IDLE;
      }
    }
  }

  _do_actors_turn(actor) {
    console.assert(actor.template.behavior !== ActorBehavior.PLAYER_INPUT);

    const distance_to_player = Util.taxicab_distance(actor.tile_x, actor.tile_y, this.player_ref.tile_x, this.player_ref.tile_y);
    const orthogonal_to_player = actor.tile_x === this.player_ref.tile_x || actor.tile_y === this.player_ref.tile_y;

    if (distance_to_player === 1) {
      this.actor_fight(actor, this.player_ref.tile_x - actor.tile_x, this.player_ref.tile_y - actor.tile_y);
      return;
    }

    if (actor.template.behavior === ActorBehavior.PATROL_VERTICALLY) {
      if (actor.behavior_state === null) {
        actor.behavior_state = 1;
      }
      const walked = this.actor_walk(actor, 0, actor.behavior_state);
      if (!walked) {
        actor.behavior_state *= -1;
      }
    } else if (actor.template.behavior === ActorBehavior.INFLICT_DAZZLE) {
      if (actor.behavior_state === null) {
        actor.behavior_state = 0;
      }
      actor.behavior_state -= 1;
      if (distance_to_player === 2 && orthogonal_to_player && actor.behavior_state <= 0) {
        this.parent_game.add_message(Messages.fairy_inflict_dazzle(actor.template.display_name, this.player_ref.template.display_name));
        this.player_ref.add_condition(Condition.DAZZLE, 4);
        actor.behavior_state = 3;
      } else {
        // Walk randomly.
        if (Util.rand_int(3) < 2) {
          const [delta_x, delta_y] = Util.choose_rand([[-1, 0], [1, 0], [0, -1], [0, 1]]);
          this.actor_walk(actor, delta_x, delta_y);
        }
      }
    }
  }

  do_end_of_turn() {
    for (const condition of this.player_ref.conditions.keys()) {
      const next_turns = this.player_ref.conditions.get(condition) - 1;
      if (next_turns === 0) {
        this.player_ref.conditions.delete(condition);
        if (condition === Condition.DAZZLE) {
          this.parent_game.add_message(Messages.dazzle_fades(this.player_ref.template.display_name));
        }
      } else {
        this.player_ref.conditions.set(condition, next_turns);
      }
    }
    for (const actor of this.actors) {
      if (actor === this.player_ref) {
        continue;
      }
      this._do_actors_turn(actor);
    }
    for (let x = 0; x < this.size_tiles.w; x++) {
      for (let y = 0; y < this.size_tiles.h; y++) {
        this._update_cell(x, y);
      }
    }
  }
}

export const Command = Object.freeze({
  PASS: Symbol("PASS"),
  WALK_UP: Symbol("WALK_UP"),
  WALK_DOWN: Symbol("WALK_DOWN"),
  WALK_LEFT: Symbol("WALK_LEFT"),
  WALK_RIGHT: Symbol("WALK_RIGHT"),
  FIGHT_UP: Symbol("FIGHT_UP"),
  FIGHT_DOWN: Symbol("FIGHT_DOWN"),
  FIGHT_LEFT: Symbol("FIGHT_LEFT"),
  FIGHT_RIGHT: Symbol("FIGHT_RIGHT"),
  GET_ITEM: Symbol("GET_ITEM"),
  DROP_ITEM: Symbol("DROP_ITEM"),
  TOGGLE_EQUIPMENT: Symbol("TOGGLE_EQUIPMENT"),
  CONSUME_ITEM: Symbol("CONSUME_ITEM"),
});

export class Game {
  turn = 0;
  current_floor = null;
  messages = []

  enter_new_floor() {
    this.current_floor = new Floor(this, 9, 9);
    this.current_floor.create_player(1, 1);
  }

  populate_test_level() {
    this.current_floor.set_cell(3, 4, CellType.DEFAULT_WALL);
    this.current_floor.set_cell(3, 5, CellType.DEFAULT_WALL);
    this.current_floor.set_cell(5, 5, CellType.DEFAULT_WALL);
    this.current_floor.set_cell(1, 4, CellType.FLOWER_HAZARD);

    this.current_floor.create_actor(ActorTemplate.HERON, 5, 1);
    this.current_floor.create_actor(ActorTemplate.STARLIGHT_FAIRY, 4, 5);

    this.current_floor.create_item(ItemTemplate.ORDINARY_SWORD, Beatitude.NEUTRAL, 1, 2);
    this.current_floor.create_item(ItemTemplate.HEALING_HERB, Beatitude.NEUTRAL, 2, 2);

    this.current_floor.create_item(ItemTemplate.ORDINARY_SWORD, Beatitude.CURSED, 1, 3);
    this.current_floor.create_item(ItemTemplate.HEALING_HERB, Beatitude.CURSED, 2, 3);

    this.current_floor.create_item(ItemTemplate.ORDINARY_SWORD, Beatitude.BLESSED, 1, 4);
    this.current_floor.create_item(ItemTemplate.HEALING_HERB, Beatitude.BLESSED, 2, 4);
  }

  _end_player_turn() {
    this.turn += 1;
    this.current_floor.do_end_of_turn();
  }

  execute_command(command, opt_param) {
    console.assert(command !== undefined);

    this.messages = [];

    if (command === Command.WALK_UP) {
      this.current_floor.actor_walk(this.current_floor.player_ref, 0, -1);
    } else if (command === Command.WALK_DOWN) {
      this.current_floor.actor_walk(this.current_floor.player_ref, 0, 1);
    } else if (command === Command.WALK_LEFT) {
      this.current_floor.actor_walk(this.current_floor.player_ref, -1, 0);
    } else if (command === Command.WALK_RIGHT) {
      this.current_floor.actor_walk(this.current_floor.player_ref, 1, 0);
    } else if (command === Command.FIGHT_UP) {
      this.current_floor.actor_fight(this.current_floor.player_ref, 0, -1);
    } else if (command === Command.FIGHT_DOWN) {
      this.current_floor.actor_fight(this.current_floor.player_ref, 0, 1);
    } else if (command === Command.FIGHT_LEFT) {
      this.current_floor.actor_fight(this.current_floor.player_ref, -1, 0);
    } else if (command === Command.FIGHT_RIGHT) {
      this.current_floor.actor_fight(this.current_floor.player_ref, 1, 0);
    } else if (command === Command.GET_ITEM) {
      console.assert(opt_param !== undefined);
      this.current_floor.player_get_item(opt_param);
    } else if (command === Command.DROP_ITEM) {
      console.assert(opt_param !== undefined);
      this.current_floor.player_drop_item(opt_param);
    } else if (command === Command.TOGGLE_EQUIPMENT) {
      console.assert(opt_param !== undefined);
      this.current_floor.player_toggle_equipment(opt_param);
    } else if (command === Command.CONSUME_ITEM) {
      console.assert(opt_param !== undefined);
      this.current_floor.player_consume_item(opt_param);
    }

    this._end_player_turn();
  }

  add_message(message) {
    this.messages.push(message);
  }

  get_messages() {
    return this.messages;
  }
}
