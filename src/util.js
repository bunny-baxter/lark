export function rand_int(min, max_exclusive) {
  if (!max_exclusive) {
    // Single param version.
    max_exclusive = min;
    min = 0;
  }
  return Math.floor(Math.random() * (max_exclusive - min)) + min;
}
