## Rendering Rules

- **Rounded rect AA artifacts**: `rect_filled` with `CornerRadius` produces semi-transparent pixels at the edges (anti-aliasing) that appear as thin visible lines. To prevent this, clip the painter to the original rect and draw the fill on an expanded rect (`rect.expand(1.0)`), so AA falls outside the clip area:
  ```rust
  let clipped = painter.with_clip_rect(rect);
  clipped.rect_filled(rect.expand(1.0), rounding, fill);
  ```
  Apply this whenever drawing a rounded filled rect. For non-rounded rects (`CornerRadius::ZERO`) this is not needed.

- **Zero-width strokes**: Never call `rect_stroke` or `circle_stroke` when stroke width is 0 or stroke is `Stroke::NONE`. Always guard with `if stroke.width > 0.0`.
