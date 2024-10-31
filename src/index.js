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

class GameplayScene extends Phaser.Scene {
  game;
  cell_sprites;
  actor_sprites = {};

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
    };
    if (test_char) {
      const [screen_x, screen_y] = this._tile_to_screen_coord(tile_x, tile_y);
      return this.add.text(screen_x, screen_y, test_char, PLACEHOLDER_SPRITE_STYLE);
    }
    return null;
  }

  _create_sprite_for_actor(actor_ref) {
    const [screen_x, screen_y] = this._tile_to_screen_coord(actor_ref.tile_x, actor_ref.tile_y);
    const sprite = this.add.text(screen_x, screen_y, "@", PLACEHOLDER_SPRITE_STYLE);
    sprite.setDepth(ACTOR_DEPTH);
    return sprite;
  }

  _init_game() {
    this.game = new Model.Game();
    this.game.enter_new_floor();

    this.cell_sprites = [];
    for (let x = 0; x < this.game.current_floor.size_tiles.w; x++) {
      this.cell_sprites.push([]);
      for (let y = 0; y < this.game.current_floor.size_tiles.h; y++) {
        const cell_type = this.game.current_floor.get_cell_type(x, y);
        this.cell_sprites[x].push(this._create_sprite_for_cell(x, y, cell_type));
      }
    }

    for (let i = 0; i < this.game.current_floor.actors.length; i++) {
      const actor = this.game.current_floor.actors[i];
      this.actor_sprites[actor.id] = this._create_sprite_for_actor(actor);
    }
  }

  _move_player(tile_delta_x, tile_delta_y) {
    const player = this.game.current_floor.player_ref;
    this.game.current_floor.actor_walk(player, tile_delta_x, tile_delta_y);
    const [screen_x, screen_y] = this._tile_to_screen_coord(player.tile_x, player.tile_y);
    this.actor_sprites[player.id].setPosition(screen_x, screen_y);
  }

  on_key_down(event) {
    if (event.code === "KeyH") {
      this._move_player(-1, 0);
    } else if (event.code === "KeyJ") {
      this._move_player(0, 1);
    } else if (event.code === "KeyK") {
      this._move_player(0, -1);
    } else if (event.code === "KeyL") {
      this._move_player(1, 0);
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
