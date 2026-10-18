// Which recorded frames the README GIF keeps: the lead-in and a fixed window of machine time, after
// which the still-running lanes are stamped with their stored verdicts and the GIF holds.

export function gifTakes(frame: number, lead_s: number, window_s: number, fps: number): boolean {
  return frame <= (lead_s + window_s) * fps;
}
