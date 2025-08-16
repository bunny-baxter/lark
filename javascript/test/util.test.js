import * as Util from '../src/util.js';

test("taxicab_distance", () => {
  expect(Util.taxicab_distance(2, 1, 1, 1)).toBe(1);
  expect(Util.taxicab_distance(0, 0, 3, 3)).toBe(6);
  expect(Util.taxicab_distance(-4, 8, 1, -1)).toBe(14);
});

test("capitalize", () => {
  expect(Util.capitalize("something")).toBe("Something");
  expect(Util.capitalize("Nothing")).toBe("Nothing");
  expect(Util.capitalize("")).toBe("");
});

test("remove_first", () => {
  const array = [1, 2, 3, 4];
  Util.remove_first(array, 3)
  expect(array).toEqual([1, 2, 4]);
  Util.remove_first(array, 3);
  expect(array).toEqual([1, 2, 4]);
});
