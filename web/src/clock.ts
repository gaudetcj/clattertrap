// Fixed-step pacing: simulated time is the step count times dt, and wall time only decides how many
// steps are owed this frame. The cap keeps a tab that was hidden for a while from freezing on return.

export function stepsDue(elapsed_s: number, done: number, dt_s: number, cap = 6): number {
  const owed = Math.floor(elapsed_s / dt_s) - done;
  return Math.max(0, Math.min(cap, owed));
}
