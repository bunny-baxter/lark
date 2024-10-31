export const CellType = Object.freeze({
  OUT_OF_BOUNDS: Symbol("OUT_OF_BOUNDS"),
  EMPTY: Symbol("EMPTY"),
  FLOOR: Symbol("FLOOR"),
  DEFAULT_WALL: Symbol("DEFAULT_WALL"),
});

export class CellData {
  type = CellType.EMPTY;
}

export class Actor {
  id;
  tile_x = 0;
  tile_y = 0;
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
  }
}

export class Game {
  current_floor = null;

  enter_new_floor() {
    this.current_floor = new Floor(9, 9);
    this.current_floor.create_player(1, 1);
  }
}
