export function rand_int(min, max_exclusive) {
  if (!max_exclusive) {
    // Single param version.
    max_exclusive = min;
    min = 0;
  }
  return Math.floor(Math.random() * (max_exclusive - min)) + min;
}

export function taxicab_distance(x1, y1, x2, y2) {
  return Math.abs(x1 - x2) + Math.abs(y1 - y2);
}

export function capitalize(str) {
  if (!str) {
    return str;
  }
  return str.charAt(0).toUpperCase() + str.slice(1);
}

export function remove_first(array, element) {
  const i = array.indexOf(element);
  if (i >= 0) {
    array.splice(i, 1);
  }
}
