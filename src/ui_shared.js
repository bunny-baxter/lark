import * as Content from './content.js';
import * as Model from './game_model.js';


export const BasicColor = Object.freeze({
  WHITE: Symbol("WHITE"),
  YELLOW: Symbol("YELLOW"),
  GRAY: Symbol("GRAY"),
  MAGENTA: Symbol("MAGENTA"),
});

export class CellVisual {
  constructor(character, color) {
    this.character = character;
    this.color = color;
  }
}


function get_char_for_actor(template) {
  if (template === Content.ActorTemplate.PLAYER) return "@";
  if (template === Content.ActorTemplate.HERON) return "h";
  if (template === Content.ActorTemplate.STARLIGHT_FAIRY) return "y";
  return "X";
}

export function get_visual_for_actor(template) {
  return new CellVisual(get_char_for_actor(template), BasicColor.YELLOW);
}


function get_char_for_item(template) {
  if (template === Content.ItemTemplate.ORDINARY_STONE) return "*";
  if (template === Content.ItemTemplate.ORDINARY_SWORD) return "/";
  if (template === Content.ItemTemplate.HEALING_HERB) return "%";
  return "X";
}

export function get_visual_for_item(template) {
  return new CellVisual(get_char_for_item(template), BasicColor.GRAY);
}


function get_char_for_cell_type(cell_type) {
  if (cell_type === Model.CellType.FLOOR) return ".";
  if (cell_type === Model.CellType.DEFAULT_WALL) return "#";
  if (cell_type === Model.CellType.FLOWER_HAZARD) return "f";
  return null;
}

export function get_visual_for_cell_type(cell_type) {
  return new CellVisual(get_char_for_cell_type(cell_type), BasicColor.WHITE);
}


export function format_messages(game) {
  const messages = [];
  for (const message of game.get_messages()) {
    messages.push(` - ${message}`);
  }
  return messages.join("\n");
}

export function get_item_description(item) {
  let description = item.get_name();
  if (item.equipped) {
    description += " (equipped)";
  }
  return description;
}
