import * as Util from './util.js';

const SCREEN_WIDTH = 1024;
const SCREEN_HEIGHT = 576;

const TILE_SIZE = 32;

const UI_DEPTH = 1;

const UI_FONT_STYLE = Object.freeze({
  fontFamily: 'Rubik',
  fontSize: '18px',
  fill: '#ffffff',
});

// Init web fonts
WebFont.load({
  custom: {
    families: [ 'Rubik' ]
  }
});

class CellData {
}

class CityExperimentScene extends Phaser.Scene {

  world_size_tiles = { w: 64, h: 64 };
  world_size_pixels = { w: this.world_size_tiles.w * TILE_SIZE, h: this.world_size_tiles.h * TILE_SIZE };

  world_cells;

  constructor(phaser_config) {
    super(phaser_config);

    this.world_cells = [];
    for (let x = 0; x < this.world_size_tiles.w; x++) {
      this.world_cells.push([]);
      for (let y = 0; y < this.world_size_tiles.h; y++) {
        this.world_cells[x].push(new CellData());
      }
    }
  }

  preload() {
  }

  on_mouse_down(pointer) {
    const down_x = Math.floor(pointer.x / TILE_SIZE);
    const down_y = Math.floor(pointer.y / TILE_SIZE);
    console.log("mouse down on cell", down_x, down_y);
  }

  create() {
    this.input.on('pointerdown', (pointer) => this.on_mouse_down(pointer));

    const test_text = this.add.text(0, 0, "Lark", UI_FONT_STYLE);
    test_text.setScrollFactor(0);
    test_text.setDepth(UI_DEPTH);
  }

  update(time, delta) {
  }
}

const config = {
  type: Phaser.AUTO,
  width: SCREEN_WIDTH,
  height: SCREEN_HEIGHT,
  scene: CityExperimentScene,
  pixelArt: true,
  disableContextMenu: true,
  audio: { noAudio: true },
};

const game = new Phaser.Game(config);
