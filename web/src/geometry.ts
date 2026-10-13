// Geometry helpers shared by the views. Units are centimetres, y up, angles counterclockwise in degrees.

export interface Point {
  x_cm: number;
  y_cm: number;
}

export interface Ends {
  left: Point;
  right: Point;
}

export function beamEnds(centre: Point, length_cm: number, angle_deg: number): Ends {
  const a = (angle_deg * Math.PI) / 180;
  const dx = (length_cm / 2) * Math.cos(a);
  const dy = (length_cm / 2) * Math.sin(a);
  return {
    left: { x_cm: centre.x_cm - dx, y_cm: centre.y_cm - dy },
    right: { x_cm: centre.x_cm + dx, y_cm: centre.y_cm + dy },
  };
}
