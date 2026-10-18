// The lockstep policy for several machines on one clock: every running lane takes the steps due,
// a lane that has ended takes none and keeps its stamp, and the grid ends when every lane has.

export function plan(due: number, ended: boolean[]): number[] {
  return ended.map((e) => (e ? 0 : due));
}

export function gridEnded(ended: boolean[]): boolean {
  return ended.every((e) => e);
}
