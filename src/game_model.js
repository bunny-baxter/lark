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

export const FLOWER_HAZARD_CYCLE_LENGTH = 5;

export class CellData {
  type = CellType.EMPTY;

  turn_counter = 0;
  phase = null;
}

export class Actor {
  id;
  tile_x = 0;
  tile_y = 0;

  current_hp = 10;
}

export class Floor {
  next_id = 0;
  size_tiles;
  cells;
  actors = [];
  player_ref = null;

  constructor(width_tiles, height_tiles) {
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

  create_player(x, y) {
    const actor = new Actor();
    actor.tile_x = x;
    actor.tile_y = y;
    actor.id = this.next_id;
    this.next_id += 1;
    this.actors.push(actor);
    this.player_ref = actor;
  }

  teleport_actor(actor_ref, to_x, to_y) {
    actor_ref.tile_x = to_x;
    actor_ref.tile_y = to_y;
  }

  actor_walk(actor_ref, delta_x, delta_y) {
    const next_x = actor_ref.tile_x + delta_x;
    const next_y = actor_ref.tile_y + delta_y;
    const next_cell_type = this.get_cell_type(next_x, next_y);
    if (next_cell_type === CellType.DEFAULT_WALL || next_cell_type === CellType.OUT_OF_BOUNDS) {
      return;
    }
    this.teleport_actor(actor_ref, next_x, next_y);
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
          actor.current_hp -= 1;
        }
      } else if (mod === FLOWER_HAZARD_CYCLE_LENGTH - 2) {
        cell_data.phase = Phase.READY;
      } else {
        cell_data.phase = Phase.IDLE;
      }
    }
  }

  do_end_of_turn() {
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
});

export class Game {
  turn = 0;
  current_floor = null;

  enter_new_floor() {
    this.current_floor = new Floor(9, 9);
    this.current_floor.create_player(1, 1);
  }

  populate_test_level() {
    this.current_floor.set_cell(3, 4, CellType.DEFAULT_WALL);
    this.current_floor.set_cell(3, 5, CellType.DEFAULT_WALL);
    this.current_floor.set_cell(1, 4, CellType.FLOWER_HAZARD);
  }

  _end_player_turn() {
    this.turn += 1;
    this.current_floor.do_end_of_turn();
  }

  execute_command(command) {
    if (command === Command.WALK_UP) {
      this.current_floor.actor_walk(this.current_floor.player_ref, 0, -1);
    } else if (command === Command.WALK_DOWN) {
      this.current_floor.actor_walk(this.current_floor.player_ref, 0, 1);
    } else if (command === Command.WALK_LEFT) {
      this.current_floor.actor_walk(this.current_floor.player_ref, -1, 0);
    } else if (command === Command.WALK_RIGHT) {
      this.current_floor.actor_walk(this.current_floor.player_ref, 1, 0);
    }

    this._end_player_turn();
  }
}
