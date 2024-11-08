import * as Model from './game_model.js';
import * as Util from './util.js';

const SCREEN_WIDTH = 1024;
const SCREEN_HEIGHT = 576;

const TILE_SIZE = 32;

const ACTOR_DEPTH = 1;
const UI_DEPTH = 2;

const UI_FONT_STYLE = Object.freeze({
  fontFamily: 'Rubik',
  fontSize: '18px',
  fill: '#ffffff',
});

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

  update(player_ref) {
    this.health_label.setText(`HP: ${player_ref.current_hp}`);
  }
}

class GameplayScene extends Phaser.Scene {
  game;
  cell_sprites;
  actor_sprites = {};
  ui_sprites;
  sprites_latest_turn = 0;

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

  _create_sprite_for_actor(actor_ref) {
    let test_char = null;
    if (actor_ref.behavior === Model.ActorBehavior.PLAYER_INPUT) {
      test_char = "@";
    } else if (actor_ref.behavior === Model.ActorBehavior.PATROL_VERTICALLY) {
      test_char = "v";
    };
    const [screen_x, screen_y] = this._tile_to_screen_coord(actor_ref.tile_x, actor_ref.tile_y);
    const sprite = this.add.text(screen_x, screen_y, test_char, PLACEHOLDER_SPRITE_STYLE);
    sprite.setDepth(ACTOR_DEPTH);
    sprite.setColor("#ffff00");
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
      this.actor_sprites[actor.id] = this._create_sprite_for_actor(actor);
    }

    this.ui_sprites = new UiSprites();
    this.ui_sprites.health_label = this.add.text(128, 400, "HP: XX", UI_FONT_STYLE);
    this.ui_sprites.health_label.setScrollFactor(0);
    this.ui_sprites.health_label.setDepth(UI_DEPTH);
    this.ui_sprites.update(this.game.current_floor.player_ref);
  }

  _update_sprites(tile_delta_x, tile_delta_y) {
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

    for (const actor of this.game.current_floor.actors) {
      const [screen_x, screen_y] = this._tile_to_screen_coord(actor.tile_x, actor.tile_y);
      this.actor_sprites[actor.id].setPosition(screen_x, screen_y);
    }

    this.ui_sprites.update(this.game.current_floor.player_ref);
  }

  on_key_down(event) {
    if (event.code === "KeyH") {
      this.game.execute_command(Model.Command.WALK_LEFT);
    } else if (event.code === "KeyJ") {
      this.game.execute_command(Model.Command.WALK_DOWN);
    } else if (event.code === "KeyK") {
      this.game.execute_command(Model.Command.WALK_UP);
    } else if (event.code === "KeyL") {
      this.game.execute_command(Model.Command.WALK_RIGHT);
    } else if (event.code === "Period") {
      this.game.execute_command(Model.Command.PASS);
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
