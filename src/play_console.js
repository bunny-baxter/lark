import pkg from 'terminal-kit';
const { terminal } = pkg;

import * as Model from './game_model.js';
import * as UiShared from './ui_shared.js';


const game = new Model.Game();
game.enter_new_floor();
game.populate_test_level();

let inventory_menu = null;

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


class InventoryMenu {
  is_empty;
  cursor_index = 0;
  item_list;

  constructor(item_list) {
    this.item_list = item_list;
    this.is_empty = item_list.length === 0;
  }

  update_terminal() {
    terminal.moveTo(1, 12).wrapColumn({ x: 24, width: 48 });

    if (this.is_empty) {
      terminal.wrap("Inventory is empty.");
    } else {
      for (let i = 0; i < this.item_list.length; i++) {
        const item = this.item_list[i];
        let description = `${i + 1}. ${UiShared.get_item_description(item)}`;
        if (this.cursor_index === i) {
          description = `^k^#^W${description}^ `;
        }
        // biome-ignore lint/style/useTemplate:
        terminal.wrap(description + "\n");
      }
    }
  }

  move_cursor(delta) {
    if (this.is_empty) {
      return;
    }
    this.cursor_index += delta;
    if (this.cursor_index < 0) {
      this.cursor_index += this.item_list.length;
    } else if (this.cursor_index >= this.item_list.length) {
      this.cursor_index -= this.item_list.length;
    }
  }

  get_selected_item() {
    if (this.is_empty) {
      return null;
    }
    return this.item_list[this.cursor_index];
  }
}


function update_terminal() {
  terminal.clear();

  for (let y = 0; y < game.current_floor.size_tiles.h; y++) {
    const line_string = [];
    for (let x = 0; x < game.current_floor.size_tiles.w; x++) {
      const visual = get_visual_for_cell(x, y);
      let c = visual.character;
      if (visual.color !== UiShared.BasicColor.WHITE) {
        c = `^${ColorChars[visual.color]}${c}^`;
      }
      line_string.push(c);
      line_string.push(" ");
    }
    terminal.moveTo(2, y + 2, line_string.join(""));
  }

  terminal.moveTo(1, 2).wrapColumn({ x: 24, width: 48 });
  terminal.wrap(UiShared.format_messages(game));

  terminal.moveTo(1, 12, `HP: ${game.current_floor.player_ref.current_hp}`);

  if (inventory_menu) {
    inventory_menu.update_terminal();
  }

  terminal.moveTo(1, 1);
}


function handle_key_normal(name) {
  if (name === "h" || name === "LEFT") {
    game.execute_walk_or_fight(-1, 0);
  } else if (name === "j" || name === "DOWN") {
    game.execute_walk_or_fight(0, 1);
  } else if (name === "k" || name === "UP") {
    game.execute_walk_or_fight(0, -1);
  } else if (name === "l" || name === "RIGHT") {
    game.execute_walk_or_fight(1, 0);
  } else if (name === "g" || name === ",") {
    game.execute_get_first_item();
  } else if (name === "i") {
    console.assert(inventory_menu === null);
    inventory_menu = new InventoryMenu(game.current_floor.player_ref.inventory);
  }
}

function handle_key_inventory(name) {
  if (name === "j" || name === "DOWN") {
    inventory_menu.move_cursor(1);
  } else if (name === "k" || name === "UP") {
    inventory_menu.move_cursor(-1);
  } else if (name === "i" || name === "ESCAPE") {
    inventory_menu = null;
  }
}

terminal.on("key", (name, matches, data) => {
  if (name === "CTRL_C") {
    do_exit();
    return;
  }
  if (inventory_menu) {
    handle_key_inventory(name);
  } else {
    handle_key_normal(name);
  }
  update_terminal();
});


update_terminal();
