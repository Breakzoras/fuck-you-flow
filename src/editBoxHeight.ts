// How tall the edit box of a transcript opens.
//
// It used to open at its smallest size whatever the transcript, so a long one
// had to be dragged open by hand before it could be read. Now it opens tall
// enough to show the whole text, and stops short of the window so the Save
// and Cancel buttons under it stay in sight.

/** The smallest the box gets, same as the stylesheet's minimum. */
export const EDIT_BOX_FLOOR = 96;
/** Room kept under the box for its buttons and the edges of the page. */
export const EDIT_BOX_RESERVE = 200;

/**
 * `textHeight` is what the text needs to show without scrolling,
 * `windowHeight` is the height of the window it lives in.
 */
export function editBoxHeight(textHeight: number, windowHeight: number): number {
  const ceiling = Math.max(EDIT_BOX_FLOOR, windowHeight - EDIT_BOX_RESERVE);
  return Math.min(Math.max(Math.ceil(textHeight), EDIT_BOX_FLOOR), ceiling);
}
