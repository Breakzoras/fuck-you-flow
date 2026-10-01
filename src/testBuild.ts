// The mark of a local test build.
//
// A build made for trying things on the maker's own PC carries the same
// version number as the public one, so the two could only be told apart by
// asking. The local build script now stamps its builds; the app shows the
// stamp under its name. A public build is made without the stamp and shows
// nothing: the stamp comes in through one build-time variable that only the
// local script sets, and it has to open with "test:" to count.

/** What the script writes in front of a stamp. */
export const TEST_BUILD_PREFIX = "test:";

/**
 * The stamp to show, or null for a public build. `raw` is the build-time
 * variable VITE_FYF_TEST_BUILD, for example "test:01-10 12:41 52484bd".
 * A release can confirm that it carries no stamp with one search of the
 * built files for `test:` followed by a date (test:[0-9][0-9]-[0-9][0-9] ).
 */
export function testBuildStamp(raw: string | undefined | null): string | null {
  const s = (raw ?? "").trim();
  if (!s.startsWith(TEST_BUILD_PREFIX)) return null;
  const stamp = s.slice(TEST_BUILD_PREFIX.length).trim();
  return stamp || null;
}
