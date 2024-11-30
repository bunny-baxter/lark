import * as Model from './game_model.js';

const SCREEN_WIDTH = 1024;
const SCREEN_HEIGHT = 576;

const TILE_SIZE = 32;

const ITEM_DEPTH = 1;
const ACTOR_DEPTH = 2;
const UI_DEPTH = 3;

const UI_FONT_STYLE = Object.freeze({
  fontFamily: 'Rubik',
  fontSize: '18px',
  fill: '#ffffff',
});
const UI_FONT_STYLE_WRAPPED = Object.freeze({ ...UI_FONT_STYLE, wordWrap: { width: 300 } });

const PLACEHOLDER_SPRITE_STYLE = Object.freeze({
  fontFamily: 'Rubik',
  fontSize: '32px',
  fill: '#ffffff',
  backgroundColor: '#000000',
});

const GRID_OFFSET_X = 128;
const GRID_OFFSET_y = 64;

class UiSprites {
  health_label = null;
  messages_label = null;

  constructor(gameplay_scene) {
    this.health_label = this._add_label(gameplay_scene, 128, 400, "HP: XX", UI_FONT_STYLE);
    this.messages_label = this._add_label(gameplay_scene, 500, GRID_OFFSET_y, "", UI_FONT_STYLE_WRAPPED);
    this.update(gameplay_scene.game);
  }

  _add_label(gameplay_scene, x, y, initial_text, style) {
    const label = gameplay_scene.add.text(x, y, initial_text, style);
    label.setScrollFactor(0);
    label.setDepth(UI_DEPTH);
    return label
  }

  update(game) {
    const player_ref = game.current_floor.player_ref;
    this.health_label.setText(`HP: ${player_ref.current_hp}`);

    let messages_text = "";
    for (const message of game.get_messages()) {
      messages_text += ` - ${message}\n`;
    }
    this.messages_label.setText(messages_text);
  }
}

function get_item_description(item) {
  return item.template.display_name;
}

class InventoryMenu {
  is_empty;
  cursor_index = 0;
  item_list;
  sprites = [];

  constructor(phaser_scene, item_list, x, y) {
    this.is_empty = item_list.length === 0;
    if (this.is_empty) {
      const sprite = phaser_scene.add.text(x, y, "Inventory is empty.", UI_FONT_STYLE);
      sprite.setScrollFactor(0);
      sprite.setDepth(UI_DEPTH);
      this.sprites.push(sprite);
    } else {
      this.item_list = item_list;
      for (let i = 0; i < this.item_list.length; i++) {
        const item = this.item_list[i];
        const description = `${i + 1}. ${get_item_description(item)}`;
        const sprite = phaser_scene.add.text(x, y + i * 24, description, UI_FONT_STYLE);
        sprite.setScrollFactor(0);
        sprite.setDepth(UI_DEPTH);
        this.sprites.push(sprite);
      }
      this.move_cursor(0); // Set selected color.
    }
  }

  move_cursor(delta) {
    if (this.is_empty) {
      return;
    }
    this.sprites[this.cursor_index].setColor("#ffffff");
    this.cursor_index += delta;
    if (this.cursor_index < 0) {
      this.cursor_index += this.item_list.length;
    } else if (this.cursor_index >= this.item_list.length) {
      this.cursor_index -= this.item_list.length;
    }
    this.sprites[this.cursor_index].setColor("#ffff00");
  }

  get_selected_item() {
    if (this.is_empty) {
      return null;
    }
    return this.item_list[this.cursor_index];
  }

  destroy() {
    for (const sprite of this.sprites) {
      sprite.destroy(true);
    }
  }
}

class GameplayScene extends Phaser.Scene {
  game;
  cell_sprites;
  object_sprites = new Map();
  ui_sprites;
  sprites_latest_turn = 0;
  inventory_menu = null;

  preload() {
  }

  _tile_to_screen_coord(tile_x, tile_y) {
    return [GRID_OFFSET_X + tile_x * TILE_SIZE, GRID_OFFSET_y + tile_y * TILE_SIZE];
  }

  _create_sprite_for_cell(tile_x, tile_y, cell_type) {
    let test_char = null;
    if (cell_type === Model.CellType.FLOOR) {
      test_char = ".";
    } else if (cell_type === Model.CellType.DEFAULT_WALL) {
      test_char = "#";
    } else if (cell_type === Model.CellType.FLOWER_HAZARD) {
      test_char = "f";
    };
    if (test_char) {
      const [screen_x, screen_y] = this._tile_to_screen_coord(tile_x, tile_y);
      return this.add.text(screen_x, screen_y, test_char, PLACEHOLDER_SPRITE_STYLE);
    }
    return null;
  }

  _create_sprite_for_object(object_ref, is_actor) {
    let test_char = null;
    if (object_ref.template === Model.ActorTemplate.PLAYER) {
      test_char = "@";
    } else if (object_ref.template === Model.ActorTemplate.HERON) {
      test_char = "v";
    } else if (object_ref.template === Model.ItemTemplate.ORDINARY_STONE) {
      test_char = "*";
    }
    const [screen_x, screen_y] = this._tile_to_screen_coord(object_ref.tile_x, object_ref.tile_y);
    const sprite = this.add.text(screen_x, screen_y, test_char, PLACEHOLDER_SPRITE_STYLE);
    sprite.setDepth(is_actor ? ACTOR_DEPTH : ITEM_DEPTH);
    sprite.setColor(is_actor ? "#ffff00" : "#88aa88");
    return sprite;
  }

  _init_game() {
    this.game = new Model.Game();
    this.game.enter_new_floor();
    this.game.populate_test_level();

    this.cell_sprites = [];
    for (let x = 0; x < this.game.current_floor.size_tiles.w; x++) {
      this.cell_sprites.push([]);
      for (let y = 0; y < this.game.current_floor.size_tiles.h; y++) {
        const cell_type = this.game.current_floor.get_cell_type(x, y);
        this.cell_sprites[x].push(this._create_sprite_for_cell(x, y, cell_type));
      }
    }

    for (const actor of this.game.current_floor.actors) {
      this.object_sprites.set(actor.id, this._create_sprite_for_object(actor, true));
    }
    for (const item of this.game.current_floor.items) {
      // Actor and Item IDs are unique between the two.
      this.object_sprites.set(item.id, this._create_sprite_for_object(item, false));
    }

    this.ui_sprites = new UiSprites(this);
  }

  _update_sprites() {
    for (let x = 0; x < this.game.current_floor.size_tiles.w; x++) {
      for (let y = 0; y < this.game.current_floor.size_tiles.h; y++) {
        const cell_data = this.game.current_floor.cells[x][y];
        const sprite = this.cell_sprites[x][y];
        if (cell_data.type === Model.CellType.FLOWER_HAZARD) {
          if (cell_data.phase === Model.Phase.ACTIVE) {
            sprite.setText("F");
          } else if (cell_data.phase === Model.Phase.READY) {
            sprite.setColor("#ff00ff");
          } else { // ===IDLE
            sprite.setColor("#ffffff");
            sprite.setText("f");
          }
        }
      }
    }

    const object_ids = new Set(this.object_sprites.keys());
    for (const object of [].concat(this.game.current_floor.actors, this.game.current_floor.items)) {
      const [screen_x, screen_y] = this._tile_to_screen_coord(object.tile_x, object.tile_y);
      this.object_sprites.get(object.id).setPosition(screen_x, screen_y);
      object_ids.delete(object.id);
    }
    for (const id of object_ids) {
      // These IDs were not seen. They must have been deleted.
      this.object_sprites.get(id).destroy(true);
      this.object_sprites.delete(id);
    }

    this.ui_sprites.update(this.game);
  }

  _execute_walk_or_fight(walk_command, fight_command, delta_x, delta_y) {
    console.assert(delta_x !== 0 || delta_y !== 0);
    const player_ref = this.game.current_floor.player_ref;
    const next_x = player_ref.tile_x + delta_x;
    const next_y = player_ref.tile_y + delta_y;
    if (this.game.current_floor.find_actors_at(next_x, next_y).length > 0) {
      this.game.execute_command(fight_command);
    } else {
      this.game.execute_command(walk_command);
    }
  }

  on_key_down(event) {
    // TODO: Should have separate functions to handle keypresses for menu vs. gameplay probably, since all the if(inventory_menu) blocks here are getting messy.
    if (event.code === "KeyH" || event.code === "ArrowLeft") {
      if (!this.inventory_menu) {
        this._execute_walk_or_fight(Model.Command.WALK_LEFT, Model.Command.FIGHT_LEFT, -1, 0);
      }
    } else if (event.code === "KeyJ" || event.code === "ArrowDown") {
      if (this.inventory_menu) {
        this.inventory_menu.move_cursor(1);
      } else {
        this._execute_walk_or_fight(Model.Command.WALK_DOWN, Model.Command.FIGHT_DOWN, 0, 1);
      }
    } else if (event.code === "KeyK" || event.code === "ArrowUp") {
      if (this.inventory_menu) {
        this.inventory_menu.move_cursor(-1);
      } else {
        this._execute_walk_or_fight(Model.Command.WALK_UP, Model.Command.FIGHT_UP, 0, -1);
      }
    } else if (event.code === "KeyL" || event.code === "ArrowRight") {
      if (!this.inventory_menu) {
        this._execute_walk_or_fight(Model.Command.WALK_RIGHT, Model.Command.FIGHT_RIGHT, 1, 0);
      }
    } else if (event.code === "Period") {
      if (!this.inventory_menu) {
        this.game.execute_command(Model.Command.PASS);
      }
    } else if (event.code === "KeyI") {
      if (this.inventory_menu) {
        this.inventory_menu.destroy();
        this.inventory_menu = null;
      } else {
        this.inventory_menu = new InventoryMenu(this, this.game.current_floor.player_ref.inventory, 500, 200);
      }
    } else if (event.code === "KeyG" || event.code === "Comma") {
      if (!this.inventory_menu) {
        const player_ref = this.game.current_floor.player_ref;
        const items = this.game.current_floor.find_loose_items_at(player_ref.tile_x, player_ref.tile_y);
        if (items.length > 0) {
          this.game.execute_command(Model.Command.GET_ITEM, items[0]);
        }
      }
    } else if (event.code === "KeyD") {
      if (this.inventory_menu && !this.inventory_menu.is_empty) {
        this.game.execute_command(Model.Command.DROP_ITEM, this.inventory_menu.get_selected_item());
        this.inventory_menu.destroy();
        this.inventory_menu = null;
      }
    } else if (event.code === "Escape") {
      if (this.inventory_menu) {
        this.inventory_menu.destroy();
        this.inventory_menu = null;
      }
    }

    if (this.sprites_latest_turn < this.game.turn) {
      this._update_sprites();
      this.sprites_latest_turn = this.game.turn;
    }
  }

  create() {
    this.input.keyboard.on('keydown', (event) => this.on_key_down(event));

    const test_text = this.add.text(0, 0, "Lark", UI_FONT_STYLE);
    test_text.setScrollFactor(0);
    test_text.setDepth(UI_DEPTH);

    this._init_game();
  }

  update(time, delta) {
  }
}

const config = {
  type: Phaser.AUTO,
  width: SCREEN_WIDTH,
  height: SCREEN_HEIGHT,
  scene: GameplayScene,
  pixelArt: true,
  disableContextMenu: true,
  audio: { noAudio: true },
};

const game = new Phaser.Game(config);
