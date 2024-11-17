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

export const ActorBehavior = Object.freeze({
  PLAYER_INPUT: Symbol("PLAYER_INPUT"),
  PATROL_VERTICALLY: Symbol("PATROL_VERTICALLY"),
});

class ActorTemplateEntry {
  display_name;
  attack_verb;
  behavior;
  max_hp;

  constructor(display_name, attack_verb, behavior, max_hp) {
    this.display_name = display_name;
    this.attack_verb = attack_verb;
    this.behavior = behavior;
    this.max_hp = max_hp;
  }
}

export const ActorTemplate = Object.freeze({
  PLAYER: new ActorTemplateEntry("Rogue", "punches", ActorBehavior.PLAYER_INPUT, 12),
  HERON: new ActorTemplateEntry("heron", "pecks", ActorBehavior.PATROL_VERTICALLY, 4),
});

export const FLOWER_HAZARD_CYCLE_LENGTH = 5;

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
  is_dead = false;

  constructor(id, template) {
    this.id = id;
    this.template = template;

    this.current_hp = template.max_hp;
  }
}

export class Floor {
  parent_game;
  next_id = 0;
  size_tiles;
  cells;
  actors = [];
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

  _hurt_actor(actor, n_damage) {
    actor.current_hp -= 1;
    if (actor.current_hp <= 0) {
      actor.is_dead = true;
      Util.remove_first(this.actors, actor);
      this.parent_game.add_message(Messages.die(actor.template.display_name));
    }
  }

  _actor_fight_actor(attacker_ref, defender_ref) {
    this.parent_game.add_message(Messages.fight(
      attacker_ref.template.display_name, attacker_ref.template.attack_verb, defender_ref.template.display_name));
    this._hurt_actor(defender_ref, 1);
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
          this._hurt_actor(actor, 1);
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

    if (Util.taxicab_distance(actor.tile_x, actor.tile_y, this.player_ref.tile_x, this.player_ref.tile_y) === 1) {
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
    }
  }

  do_end_of_turn() {
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
    this.current_floor.set_cell(1, 4, CellType.FLOWER_HAZARD);

    this.current_floor.create_actor(ActorTemplate.HERON, 3, 1);
  }

  _end_player_turn() {
    this.turn += 1;
    this.current_floor.do_end_of_turn();
  }

  execute_command(command) {
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
