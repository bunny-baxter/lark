import * as Content from './content.js';
import * as Model from './game_model.js';


export const BasicColor = Object.freeze({
  WHITE: Symbol("WHITE"),
  YELLOW: Symbol("YELLOW"),
  RED: Symbol("RED"),
  GRAY: Symbol("GRAY"),
  MAGENTA: Symbol("MAGENTA"),
  CYAN: Symbol("CYAN"),
  BLUE: Symbol("BLUE"),
  YELLOW_GREEN: Symbol("YELLOW_GREEN"),
  BLUE_GREEN: Symbol("BLUE_GREEN"),
});

const ConditionLabels = Object.freeze({
  [Model.Condition.DAZZLE]: "dazzled",
  [Model.Condition.SLOW]: "slow",
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
  if (template === Content.ActorTemplate.MERMAID) return "m";
  if (template === Content.ActorTemplate.BERRY_SHRUB) return "b";
  return "X";
}

export function get_visual_for_actor(template) {
  let color = BasicColor.YELLOW;
  if (template === Content.ActorTemplate.BERRY_SHRUB) {
    color = BasicColor.BLUE_GREEN;
  }
  return new CellVisual(get_char_for_actor(template), color);
}


function get_char_for_item(template) {
  if (template === Content.ItemTemplate.ORDINARY_STONE) return "*";
  if (template === Content.ItemTemplate.ORDINARY_SWORD) return "/";
  if (template === Content.ItemTemplate.POWERFUL_SWORD) return "|";
  if (template === Content.ItemTemplate.ORDINARY_CHAINMAIL) return "[";
  if (template === Content.ItemTemplate.DRIED_BLOODFLOWER) return "%";
  if (template === Content.ItemTemplate.SWIMMING_RING || template === Content.ItemTemplate.FENCING_RING) return "o";
  if (template === Content.ItemTemplate.ICE_WAND) return "!";
  if (template === Content.ItemTemplate.STEEL_KNIFE || template === Content.ItemTemplate.SILVER_KNIFE) return "-";
  if (template === Content.ItemTemplate.DARKBERRY) return ":";
  return "X";
}

export function get_visual_for_item(template) {
  return new CellVisual(get_char_for_item(template), BasicColor.GRAY);
}


export const FLOWER_HAZARD_ACTIVE_CHAR = get_char_for_cell_type(Model.CellType.FLOWER_HAZARD).toUpperCase();

export function get_char_for_cell_type(cell_type) {
  if (cell_type === Model.CellType.FLOOR) return ".";
  if (cell_type === Model.CellType.MOSS) return ",";
  if (cell_type === Model.CellType.THYME) return "\"";
  if (cell_type === Model.CellType.BLOODFLOWER_PLANT) return "\"";
  if (cell_type === Model.CellType.DEFAULT_WALL) return "#";
  if (cell_type === Model.CellType.FLOWER_HAZARD) return "f";
  if (cell_type === Model.CellType.SHALLOW_WATER) return "~";
  if (cell_type === Model.CellType.DEEP_WATER) return "~";
  if (cell_type === Model.CellType.ICE) return "=";
  return null;
}

export function get_visual_for_cell_type(cell_type) {
  let color = BasicColor.WHITE;
  if (cell_type === Model.CellType.SHALLOW_WATER || cell_type === Model.CellType.ICE) {
    color = BasicColor.CYAN;
  } else if (cell_type === Model.CellType.DEEP_WATER) {
    color = BasicColor.BLUE;
  } else if (cell_type === Model.CellType.MOSS) {
    color = BasicColor.BLUE_GREEN;
  } else if (cell_type === Model.CellType.THYME) {
    color = BasicColor.YELLOW_GREEN;
  } else if (cell_type === Model.CellType.BLOODFLOWER_PLANT) {
    color = BasicColor.RED;
  }
  return new CellVisual(get_char_for_cell_type(cell_type), color);
}


export function format_conditions(game) {
  const conditions_text = [];
  for (const condition of game.current_floor.player_ref.conditions.keys()) {
    conditions_text.push(ConditionLabels[condition]);
  }
  return conditions_text.join(" ");
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
