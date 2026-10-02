# Matugen contract

Probed on: 2026-10-01
Probe machine: Windows 11 VM
Matugen version: 4.2.0
Binary path: %USERPROFILE%\.cargo\bin\matugen.exe

## Invocation

    matugen image <path> -j hex --dry-run \
      --mode <dark|light|smart> \
      --type <scheme-*> \
      --contrast <-1.0 .. 1.0> \
      --source-color-index 0

Required: -j hex, --source-color-index (or --prefer, or --fallback-color).
Without one of the three source-color flags, matugen opens an
interactive selector and blocks.

## Output

JSON on stdout. Top-level keys:

- base16 — muted derivation of source color. Not used.
- colors — M3 roles. This is what Murf reads.
- image — source path with leading /?/ prefix. Not used.
- is_dark_mode — boolean. Not used.
- mode — "light" | "dark". Not used.
- palettes — tonal scale per hue. Reserved for v0.3.

## Value path

colors.<role>.default.color

Each role also has .dark.color and .light.color variants in the
same payload. `.default` reflects the requested --mode.

## Roles (48)

background, error, error_container, inverse_on_surface,
inverse_primary, inverse_surface, on_background, on_error,
on_error_container, on_primary, on_primary_container,
on_primary_fixed, on_primary_fixed_variant, on_secondary,
on_secondary_container, on_secondary_fixed,
on_secondary_fixed_variant, on_surface, on_surface_variant,
on_tertiary, on_tertiary_container, on_tertiary_fixed,
on_tertiary_fixed_variant, outline, outline_variant,
primary, primary_container, primary_fixed, primary_fixed_dim,
scrim, secondary, secondary_container, secondary_fixed,
secondary_fixed_dim, shadow, source_color, surface,
surface_bright, surface_container, surface_container_high,
surface_container_highest, surface_container_low,
surface_container_lowest, surface_dim, surface_tint,
surface_variant, tertiary, tertiary_container, tertiary_fixed,
tertiary_fixed_dim

## Type values

scheme-content, scheme-expressive, scheme-fidelity,
scheme-fruit-salad, scheme-monochrome, scheme-neutral,
scheme-rainbow, scheme-tonal-spot, scheme-vibrant, scheme-smart

Note: prefixed with "scheme-". Murf's config uses unprefixed names
and the bridge prepends.

## Behavior confirmed

- CLI --mode overrides default mode.
- Solid-color images succeed.
- Config directory is not created on install.
- -j hex emits hex strings ("#rrggbb").