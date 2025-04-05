import {ActorTemplate, ItemTemplate} from '../src/content';
import * as Model from '../src/game_model';

let game;
let floor;

beforeEach(() => {
  game = new Model.Game();
  game.enter_new_floor();
  floor = game.current_floor;
});

test("actor walk", () => {
  expect([floor.player_ref.tile_x, floor.player_ref.tile_y]).toEqual([1, 1]);
  expect(game.turn).toBe(0);
  game.execute_command(Model.Command.WALK_RIGHT);
  expect([floor.player_ref.tile_x, floor.player_ref.tile_y]).toEqual([2, 1]);
  expect(game.turn).toBe(1);
});

test("actor doesn't walk into walls", () => {
  floor.set_cell(2, 1, Model.CellType.DEFAULT_WALL);
  expect([floor.player_ref.tile_x, floor.player_ref.tile_y]).toEqual([1, 1]);
  game.execute_command(Model.Command.WALK_RIGHT);
  expect([floor.player_ref.tile_x, floor.player_ref.tile_y]).toEqual([1, 1]);
});

test("flower hazard does damage on cycle", () => {
  const initial_hp = floor.player_ref.current_hp;
  floor.set_cell(1, 1, Model.CellType.FLOWER_HAZARD);
  for (let i = 0; i < Model.FLOWER_HAZARD_CYCLE_LENGTH - 1; i++) {
    game.execute_command(Model.Command.PASS);
  }
  const hp_after_hit = floor.player_ref.current_hp;
  expect(hp_after_hit).toBeLessThan(initial_hp);
  expect(game.get_messages().length).toBe(1);
  // Should not hit on every turn.
  game.execute_command(Model.Command.PASS);
  expect(floor.player_ref.current_hp).toBe(hp_after_hit);
  expect(game.get_messages().length).toBe(0);
});

test("player can walk into shallow but not deep water", () => {
  floor.set_cell(2, 1, Model.CellType.SHALLOW_WATER);
  floor.set_cell(3, 1, Model.CellType.DEEP_WATER);
  game.execute_command(Model.Command.WALK_RIGHT);
  game.execute_command(Model.Command.WALK_RIGHT);
  expect([floor.player_ref.tile_x, floor.player_ref.tile_y]).toEqual([2, 1]);
});

test("player slowed by shallow water", () => {
  floor.set_cell(2, 1, Model.CellType.SHALLOW_WATER);
  expect(game.turn).toBe(0);
  game.execute_command(Model.Command.WALK_RIGHT);
  expect(game.turn).toBe(2);
  game.execute_command(Model.Command.WALK_RIGHT);
  expect(game.turn).toBe(3);
});

test("player with swimming ring moves through water", () => {
  const item_ref = floor.create_item(ItemTemplate.SWIMMING_RING, Model.Beatitude.NEUTRAL, 1, 1);
  game.execute_command(Model.Command.GET_ITEM, item_ref);
  game.execute_command(Model.Command.TOGGLE_EQUIPMENT, item_ref);

  floor.set_cell(2, 1, Model.CellType.SHALLOW_WATER);
  floor.set_cell(3, 1, Model.CellType.DEEP_WATER);

  expect(game.turn).toBe(2);
  game.execute_command(Model.Command.WALK_RIGHT);
  game.execute_command(Model.Command.WALK_RIGHT);
  game.execute_command(Model.Command.WALK_RIGHT);
  expect([floor.player_ref.tile_x, floor.player_ref.tile_y]).toEqual([4, 1]);
  expect(game.turn).toBe(5);
});

test("heron enemy moves vertically", () => {
  floor.set_cell(3, 4, Model.CellType.DEFAULT_WALL);
  const heron_ref = floor.create_actor(ActorTemplate.HERON, 3, 1);
  const path = [[3, 2], [3, 3], [3, 3], [3, 2], [3, 1], [3, 1], [3, 2]];
  for (let i = 0; i < path.length; i++) {
    game.execute_command(Model.Command.PASS);
    expect([heron_ref.tile_x, heron_ref.tile_y]).toEqual(path[i]);
  }
});

test("starlight fairy dazzles player at range", () => {
  const fairy_ref = floor.create_actor(ActorTemplate.STARLIGHT_FAIRY, 3, 1);
  expect(floor.player_ref.has_condition(Model.Condition.DAZZLE)).toBe(false);

  game.execute_command(Model.Command.PASS);
  // Fairy inflicts dazzle at range 2.
  expect(floor.player_ref.has_condition(Model.Condition.DAZZLE)).toBe(true);

  // But attacks when adjacent.
  expect(floor.player_ref.current_hp).toBe(floor.player_ref.max_hp);
  game.execute_command(Model.Command.WALK_RIGHT);
  expect(floor.player_ref.current_hp).toBeLessThan(floor.player_ref.max_hp);

  game.execute_command(Model.Command.PASS);
  game.execute_command(Model.Command.PASS);
  game.execute_command(Model.Command.PASS);
  // Condition wears off.
  expect(floor.player_ref.has_condition(Model.Condition.DAZZLE)).toBe(false);
});

test("slow player skips turns", () => {
  floor.player_ref.add_condition(Model.Condition.SLOW, 4);
  expect(game.turn).toBe(0);
  expect(floor.player_ref.has_condition(Model.Condition.SLOW)).toBe(true);
  game.execute_command(Model.Command.PASS);
  expect(game.turn).toBe(2);
  expect(floor.player_ref.has_condition(Model.Condition.SLOW)).toBe(true);
  game.execute_command(Model.Command.PASS);
  expect(game.turn).toBe(4);
  expect(floor.player_ref.has_condition(Model.Condition.SLOW)).toBe(false);
  game.execute_command(Model.Command.PASS);
  expect(game.turn).toBe(5);
});

test("adjacent enemy attacks player", () => {
  const heron_ref = floor.create_actor(ActorTemplate.HERON, 2, 1);
  const initial_hp = floor.player_ref.current_hp;
  game.execute_command(Model.Command.PASS);
  // Enemy has not moved, and instead attacked the player.
  expect([heron_ref.tile_x, heron_ref.tile_y]).toEqual([2, 1]);
  expect(floor.player_ref.current_hp).toBeLessThan(initial_hp);
  expect(game.get_messages().length).toBe(1);
});

test("cannot move into other actor's space", () => {
  const heron_ref = floor.create_actor(ActorTemplate.HERON, 2, 1);
  game.execute_command(Model.Command.WALK_RIGHT);
  // Player did not move.
  expect([floor.player_ref.tile_x, floor.player_ref.tile_y]).toEqual([1, 1]);
});

test("player attacks enemy", () => {
  const heron_ref = floor.create_actor(ActorTemplate.HERON, 2, 1);
  const initial_hp = heron_ref.current_hp;
  game.execute_command(Model.Command.FIGHT_RIGHT);
  expect(heron_ref.current_hp).toBeLessThan(initial_hp);
  // Message 1: player attacks heron. Message 2: heron attacks player.
  expect(game.get_messages().length).toBe(2);
});

test("player defeats enemy", () => {
  const heron_ref = floor.create_actor(ActorTemplate.HERON, 2, 1);
  expect(heron_ref.is_dead).toBe(false);
  while (heron_ref.current_hp > 0) {
    game.execute_command(Model.Command.FIGHT_RIGHT);
  }
  expect(heron_ref.is_dead).toBe(true);
  expect(floor.find_actors_at(2, 1).length).toBe(0);
});

test("basic item manipulation", () => {
  const item_ref = floor.create_item(ItemTemplate.ORDINARY_STONE, Model.Beatitude.NEUTRAL, 1, 1);
  expect([item_ref.tile_x, item_ref.tile_y]).toEqual([1, 1]);
  expect(item_ref.held_actor).toBe(null);
  expect(floor.player_ref.inventory).toEqual([]);

  game.execute_command(Model.Command.GET_ITEM, item_ref);
  expect(item_ref.held_actor).toBe(floor.player_ref);
  expect(floor.player_ref.inventory).toEqual([item_ref]);

  // Item moves with player.
  game.execute_command(Model.Command.WALK_DOWN);
  expect([item_ref.tile_x, item_ref.tile_y]).toEqual([1, 2]);

  game.execute_command(Model.Command.DROP_ITEM, item_ref);
  expect(item_ref.held_actor).toBe(null);
  expect(floor.player_ref.inventory).toEqual([]);

  // Item stays behind.
  game.execute_command(Model.Command.WALK_UP);
  expect([item_ref.tile_x, item_ref.tile_y]).toEqual([1, 2]);
});

test("equip sword", () => {
  const item_ref = floor.create_item(ItemTemplate.ORDINARY_SWORD, Model.Beatitude.NEUTRAL, 1, 1);
  expect(floor.player_ref.attack_power).toBe(1);
  expect(item_ref.equipped).toBe(false);

  game.execute_command(Model.Command.GET_ITEM, item_ref);
  game.execute_command(Model.Command.TOGGLE_EQUIPMENT, item_ref);
  expect(floor.player_ref.attack_power).toBe(1 + ItemTemplate.ORDINARY_SWORD.equipped_attack_power);
  expect(item_ref.equipped).toBe(true);

  game.execute_command(Model.Command.TOGGLE_EQUIPMENT, item_ref);
  expect(floor.player_ref.attack_power).toBe(1);
  expect(item_ref.equipped).toBe(false);
});

test("swap sword with another sword", () => {
  const item_ref1 = floor.create_item(ItemTemplate.ORDINARY_SWORD, Model.Beatitude.NEUTRAL, 1, 1);
  const item_ref2 = floor.create_item(ItemTemplate.ORDINARY_SWORD, Model.Beatitude.NEUTRAL, 1, 1);
  game.execute_command(Model.Command.GET_ITEM, item_ref1);
  game.execute_command(Model.Command.GET_ITEM, item_ref2);

  game.execute_command(Model.Command.TOGGLE_EQUIPMENT, item_ref1);
  expect(item_ref1.equipped).toBe(true);
  expect(item_ref2.equipped).toBe(false);

  game.execute_command(Model.Command.TOGGLE_EQUIPMENT, item_ref2);
  expect(item_ref1.equipped).toBe(false);
  expect(item_ref2.equipped).toBe(true);
});

test("sword increases player's attack power", () => {
  const sword_ref = floor.create_item(ItemTemplate.ORDINARY_SWORD, Model.Beatitude.NEUTRAL, 1, 1);
  const heron_ref = floor.create_actor(ActorTemplate.HERON, 2, 1);
  const heron_starting_hp = heron_ref.current_hp;
  game.execute_command(Model.Command.GET_ITEM, sword_ref);
  game.execute_command(Model.Command.TOGGLE_EQUIPMENT, sword_ref);
  game.execute_command(Model.Command.FIGHT_RIGHT);
  expect(heron_ref.current_hp).toBe(heron_starting_hp - (1 + ItemTemplate.ORDINARY_SWORD.equipped_attack_power));
});

test("cursed sword reduces luck", () => {
  const sword_ref = floor.create_item(ItemTemplate.ORDINARY_SWORD, Model.Beatitude.CURSED, 1, 1);
  expect(floor.player_ref.luck).toBe(0);
  game.execute_command(Model.Command.GET_ITEM, sword_ref);
  expect(floor.player_ref.luck).toBe(-1 * Model.CURSED_ITEM_LUCK_PENALTY);
  game.execute_command(Model.Command.TOGGLE_EQUIPMENT, sword_ref);
  expect(floor.player_ref.luck).toBe(-1 * (Model.CURSED_ITEM_LUCK_PENALTY + Model.CURSED_EQUIPMENT_LUCK_PENALTY));
  game.execute_command(Model.Command.TOGGLE_EQUIPMENT, sword_ref);
  expect(floor.player_ref.luck).toBe(-1 * Model.CURSED_ITEM_LUCK_PENALTY);
  game.execute_command(Model.Command.DROP_ITEM, sword_ref);
  expect(floor.player_ref.luck).toBe(0);
});

test("armor increases defense", () => {
  const item_ref = floor.create_item(ItemTemplate.ORDINARY_CHAINMAIL, Model.Beatitude.NEUTRAL, 1, 1);
  game.execute_command(Model.Command.GET_ITEM, item_ref);
  game.execute_command(Model.Command.TOGGLE_EQUIPMENT, item_ref);

  const heron_ref = floor.create_actor(ActorTemplate.HERON, 2, 1);
  const initial_hp = floor.player_ref.current_hp;
  game.execute_command(Model.Command.PASS);
  // Armor decreases damage to 0.
  expect(floor.player_ref.current_hp).toBe(initial_hp);
});

test("eating healing herb heals", () => {
  const item_ref = floor.create_item(ItemTemplate.HEALING_HERB, Model.Beatitude.NEUTRAL, 1, 1);
  floor.player_ref.current_hp = 1;
  expect(item_ref.is_destroyed).toBe(false);
  game.execute_command(Model.Command.GET_ITEM, item_ref);
  game.execute_command(Model.Command.CONSUME_ITEM, item_ref);
  expect(floor.player_ref.current_hp).toBe(floor.player_ref.max_hp);
  expect(floor.player_ref.inventory.length).toBe(0);
  expect(floor.items.indexOf(item_ref)).toBe(-1);
  expect(item_ref.is_destroyed).toBe(true);
});

test("eating blessed healing herb adds max hp", () => {
  const item_ref = floor.create_item(ItemTemplate.HEALING_HERB, Model.Beatitude.BLESSED, 1, 1);
  const initial_max_hp = floor.player_ref.max_hp;
  game.execute_command(Model.Command.GET_ITEM, item_ref);
  game.execute_command(Model.Command.CONSUME_ITEM, item_ref);
  expect(floor.player_ref.max_hp).toBe(initial_max_hp + Model.HEALING_HERB_BLESSED_MAX_HP_BONUS);
  expect(floor.player_ref.current_hp).toBe(floor.player_ref.max_hp);
});

test("eating cursed healing herb hurts", () => {
  const item_ref = floor.create_item(ItemTemplate.HEALING_HERB, Model.Beatitude.CURSED, 1, 1);
  game.execute_command(Model.Command.GET_ITEM, item_ref);
  game.execute_command(Model.Command.CONSUME_ITEM, item_ref);
  expect(floor.player_ref.current_hp).toBe(floor.player_ref.max_hp - Model.HEALING_HERB_CURSED_DAMAGE_AMOUNT);
});

test("attack with ice wand", () => {
  const item_ref = floor.create_item(ItemTemplate.ICE_WAND, Model.Beatitude.NEUTRAL, 1, 1);
  game.execute_command(Model.Command.GET_ITEM, item_ref);
  const enemy_ref = floor.create_actor(ActorTemplate.HERON, 1, 2);

  const initial_hp = enemy_ref.current_hp;
  game.execute_command(Model.Command.ACTIVATE_ITEM_DOWN, item_ref);
  expect(enemy_ref.current_hp).toBeLessThan(initial_hp);
});

test("ice wand freezes water", () => {
  floor.set_cell(2, 1, Model.CellType.SHALLOW_WATER);
  floor.set_cell(3, 1, Model.CellType.DEEP_WATER);
  const item_ref = floor.create_item(ItemTemplate.ICE_WAND, Model.Beatitude.NEUTRAL, 1, 1);
  game.execute_command(Model.Command.GET_ITEM, item_ref);
  game.execute_command(Model.Command.ACTIVATE_ITEM_RIGHT, item_ref);
  expect(floor.get_cell_type(2, 1)).toBe(Model.CellType.ICE);
  expect(floor.get_cell_type(3, 1)).toBe(Model.CellType.ICE);
});

test("knife harvests berries", () => {
  const item_ref = floor.create_item(ItemTemplate.STEEL_KNIFE, Model.Beatitude.NEUTRAL, 1, 1);
  game.execute_command(Model.Command.GET_ITEM, item_ref);
  const shrub_ref = floor.create_actor(ActorTemplate.BERRY_SHRUB, 2, 1);

  expect(floor.find_loose_items_at(1, 1).length).toBe(0);
  game.execute_command(Model.Command.ACTIVATE_ITEM_RIGHT, item_ref);
  const berries = floor.find_loose_items_at(1, 1);
  expect(berries.length).toBe(3);
  expect(berries[0].template).toBe(ItemTemplate.DARKBERRY);

  // Second harvest does nothing.
  game.execute_command(Model.Command.ACTIVATE_ITEM_RIGHT, item_ref);
  expect(floor.find_loose_items_at(1, 1).length).toBe(3);
});
