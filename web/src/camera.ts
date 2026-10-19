// Maps world metres (origin bottom-left, y up) onto canvas pixels (origin top-left, y down),
// fitting the whole level into the canvas with equal margins on the slack axis.

export interface Px {
  x: number;
  y: number;
}

export class Camera {
  private constructor(
    readonly scale_px_per_m: number,
    private readonly origin_x_px: number,
    private readonly origin_y_px: number,
  ) {}

  static fit(width_cm: number, height_cm: number, canvas_w_px: number, canvas_h_px: number): Camera {
    const width_m = width_cm / 100;
    const height_m = height_cm / 100;
    const scale = Math.min(canvas_w_px / width_m, canvas_h_px / height_m);
    const origin_x = Math.round((canvas_w_px - width_m * scale) / 2);
    const origin_y = Math.round(canvas_h_px - (canvas_h_px - height_m * scale) / 2);
    return new Camera(scale, origin_x, origin_y);
  }

  toPx(x_m: number, y_m: number): Px {
    return { x: this.origin_x_px + x_m * this.scale_px_per_m, y: this.origin_y_px - y_m * this.scale_px_per_m };
  }

  px(length_m: number): number {
    return length_m * this.scale_px_per_m;
  }

  // The world point under a canvas pixel.
  toWorld(x_px: number, y_px: number): { x_m: number; y_m: number } {
    return { x_m: (x_px - this.origin_x_px) / this.scale_px_per_m, y_m: (this.origin_y_px - y_px) / this.scale_px_per_m };
  }
}
