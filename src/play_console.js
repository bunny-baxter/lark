import pkg from 'terminal-kit';
const { terminal } = pkg;

import * as Model from './game_model.js';
import * as UiShared from './ui_shared.js';


const game = new Model.Game();
game.enter_new_floor();
game.populate_test_level();

terminal.fullscreen();
terminal.grabInput();


function do_exit() {
  terminal.fullscreen(false);
  terminal.grabInput(false);
  process.exit();
}


function get_visual_for_cell(x, y) {
  const actors = game.current_floor.find_actors_at(x, y);
  if (actors.length) {
    return UiShared.get_visual_for_actor(actors[0].template);
  }
  const items = game.current_floor.find_loose_items_at(x, y);
  if (items.length) {
    return UiShared.get_visual_for_item(items[0].template);
  }
  return UiShared.get_visual_for_cell_type(game.current_floor.get_cell_type(x, y));
}


const ColorChars = Object.freeze({
  [UiShared.BasicColor.WHITE]: "W",
  [UiShared.BasicColor.YELLOW]: "Y",
  [UiShared.BasicColor.GRAY]: "c",
});


function update() {
  terminal.clear();

  for (let y = 0; y < game.current_floor.size_tiles.h; y++) {
    let line_string = ""
    for (let x = 0; x < game.current_floor.size_tiles.w; x++) {
      const visual = get_visual_for_cell(x, y);
      let c = visual.character;
      if (visual.color !== UiShared.BasicColor.WHITE) {
        c = `^${ColorChars[visual.color]}${c}^`;
      }
      line_string += c;
      line_string += " ";
    }
    terminal.moveTo(2, y + 2, line_string);
  }

  terminal.moveTo(1, 1).wrapColumn({ x: 24, y: 4, width: 48 });
  terminal.wrap(UiShared.format_messages(game));
}


terminal.on("key", (name, matches, data) => {
	if (name === "CTRL_C") {
    do_exit();
    return;
  }
  if (name === "h" || name === "LEFT") {
    game.execute_walk_or_fight(-1, 0);
  } else if (name === "j" || name === "DOWN") {
    game.execute_walk_or_fight(0, 1);
  } else if (name === "k" || name === "UP") {
    game.execute_walk_or_fight(0, -1);
  } else if (name === "l" || name === "RIGHT") {
    game.execute_walk_or_fight(1, 0);
  }
  update();
});


update();
