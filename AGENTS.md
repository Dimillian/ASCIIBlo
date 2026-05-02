# AGENTS.md

## Project Purpose

`ASCIIBlo` is a native Rust action RPG prototype built with `macroquad`.

The game should feel like a real action RPG first: continuous movement, combat, loot, progression, towns, travel, and readable UI. ASCII is a visual influence for glyphs and texture, not a terminal constraint. Keep the graphical renderer and gameplay loop central when extending the project.

## Quick Start

```bash
cargo run
cargo test
cargo fmt --check
```

For UI work, use the deterministic preview system instead of trying to inspect or capture the live game window:

```bash
cargo run -- --preview inventory --output /tmp/asciiblo-inventory.png
cargo run -- --preview-all /tmp/asciiblo-previews
```

## Code Map

- `src/main.rs`
  - App entry point, fixed-timestep loop, renderer wiring, and preview capture flow.
- `src/game.rs`
  - Authoritative runtime state and gameplay rules: player, monsters, combat, NPC interaction, loot pickup, shops, travel, UI mode state, and most gameplay tests.
- `src/world.rs`
  - Deterministic procedural world generation, biome layout, tile lookup, walkability, and biome-level progression.
- `src/content.rs`
  - Static content tables and generation rules for monsters, items, rarities, affixes, starter gear, shop stock, and drops.
- `src/render.rs`
  - Rendering only: world drawing, actors, combat feed, minimap, HUD, inventory, character sheet, shop, and travel screens.
- `src/preview.rs`
  - Code-driven screenshot modes used for repeatable UI previews and regression checks.

## Working Rules

- Prefer the current module boundaries.
  - Gameplay behavior belongs in `game.rs`.
  - Procedural geography belongs in `world.rs`.
  - Data tables and loot formulas belong in `content.rs`.
  - Presentation belongs in `render.rs`.
- Keep generation deterministic where it already is. World output and previews should remain reproducible from the same seed.
- Treat the preview system as part of the product. When changing UI layout, add or use a preview path and inspect the generated image.
- Preserve readability over cleverness. This project is still small; local explicit code is usually better than premature abstraction.
- Keep edits focused. Avoid unrelated rewrites while touching gameplay, rendering, or content balance.
- If a change affects progression, combat, spawning, loot, or UI navigation, add or update a targeted test near the affected module.

## Current Design Notes

- The world is infinite and deterministic, with a handcrafted safe town at the origin and irregular repeated biome provinces outside it.
- Biome level is based on distance from town, but biome identity is intentionally not a clean ring pattern.
- Monsters are spawned near the player and scale from local biome level, not player level directly.
- Loot is rolled from monster level:
  - rarity first
  - then a base item
  - then affixes or uniques depending on rarity
  - then item-level tier bonuses
- UI should be easy to parse at a glance. Inventory and combat-feed changes should favor spacing, grouping, wrapping, and contrast over dense text.

## Validation Checklist

Before handing off a change, run the smallest useful set that matches the work:

```bash
cargo fmt --check
cargo test
```

For UI changes, also render at least the relevant preview:

```bash
cargo run -- --preview inventory --output /tmp/asciiblo-inventory.png
```

Use `--preview-all` when a change may affect multiple windows or shared layout helpers.

## Common Pitfalls

- Do not judge UI changes only from the live window. Use generated previews so results are repeatable.
- Be careful when optimizing rendering. Measure or validate behavior before replacing a simple draw path with a more complex caching layer.
- Avoid making biomes purely concentric again; progression should exist without making the world feel like nested circles.
- Do not couple enemy or loot scaling to the player unless that is the explicit design change. The current game uses world/monster level as the source of progression pressure.
- When changing layout, check long strings and low-resolution cases. This UI has already had overlap bugs from optimistic spacing.

## Product Direction

When unsure, prefer changes that make the game feel:

1. More legible
2. More responsive
3. More like a real ARPG
4. More testable and previewable

The prototype is already past the “ASCII demo” stage. Build forward from the current engine, renderer, and gameplay loop rather than falling back to terminal-style shortcuts.
