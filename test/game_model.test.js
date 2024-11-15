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

test("heron enemy moves vertically", () => {
  floor.set_cell(3, 4, Model.CellType.DEFAULT_WALL);
  const heron_ref = floor.create_actor(Model.ActorTemplate.HERON, 3, 1);
  const path = [[3, 2], [3, 3], [3, 3], [3, 2], [3, 1], [3, 1], [3, 2]];
  for (let i = 0; i < path.length; i++) {
    game.execute_command(Model.Command.PASS);
    expect([heron_ref.tile_x, heron_ref.tile_y]).toEqual(path[i]);
  }
});

test("adjacent enemy attacks player", () => {
  const heron_ref = floor.create_actor(Model.ActorTemplate.HERON, 2, 1);
  const initial_hp = floor.player_ref.current_hp;
  game.execute_command(Model.Command.PASS);
  // Enemy has not moved, and instead attacked the player.
  expect([heron_ref.tile_x, heron_ref.tile_y]).toEqual([2, 1]);
  expect(floor.player_ref.current_hp).toBeLessThan(initial_hp);
  expect(game.get_messages().length).toBe(1);
});
