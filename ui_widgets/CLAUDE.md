## Rendering Rules

- **Zero-width strokes**: Never call `rect_stroke` or `circle_stroke` when stroke width is 0 or stroke is `Stroke::NONE`. Always guard with `if stroke.width > 0.0`.
