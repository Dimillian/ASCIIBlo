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
- `src/game/`
  - Authoritative runtime state and gameplay rules.
  - `mod.rs`: `Game` aggregate, lifecycle/update loop, shared constants, and public surface.
  - `types.rs`: gameplay data types such as player, monsters, loot, stats, equipment, and combat effects.
  - `input.rs`: live input collection and edge-trigger reset handling.
  - `combat.rs`: attacks, skills, projectiles, monster AI, damage, deaths, and particles.
  - `spawning.rs`: monster spawn, cull, and refill rules.
  - `menus.rs`: gameplay-side control flow for character, skill book, shops, travel, and world map state.
  - `inventory.rs`: loot pickup, equip, buy, sell, and drop behavior.
  - `progression.rs`: use-based mastery XP, discipline level-ups, unlock notifications, and agility travel XP.
  - `tests.rs`: focused gameplay coverage and smoke flows.
- `src/world.rs`
  - Deterministic procedural world generation, biome layout, tile lookup, walkability, and biome-level progression.
- `src/content.rs`
  - Static content tables and generation rules for monsters, items, rarities, affixes, starter gear, shop stock, and drops.
- `src/render.rs`
  - Rendering only: world drawing, actors, combat feed, minimap, HUD, inventory, character sheet, shop, and travel screens.
- `src/ui/`
  - Individual UI surfaces and shared widgets for HUD, inventory, character, skill book, shops, trainer, travel, and world map.
- `src/preview.rs`
  - Code-driven screenshot modes used for repeatable UI previews and regression checks.

## Working Rules

- Prefer the current module boundaries.
  - Core orchestration belongs in `game/mod.rs`.
  - Combat behavior belongs in `game/combat.rs`.
  - Spawn/refill behavior belongs in `game/spawning.rs`.
  - Menu-side gameplay behavior belongs in `game/menus.rs`.
  - Inventory economy belongs in `game/inventory.rs`.
  - Mastery/progression behavior belongs in `game/progression.rs`.
  - Shared gameplay data types belong in `game/types.rs`.
  - Procedural geography belongs in `world.rs`.
  - Data tables and loot formulas belong in `content.rs`.
  - Presentation belongs in `render.rs` and `ui/`.
- Keep generation deterministic where it already is. World output and previews should remain reproducible from the same seed.
- Treat the preview system as part of the product. When changing UI layout, add or use a preview path and inspect the generated image.
- Preserve readability over cleverness. This project is still small; local explicit code is usually better than premature abstraction.
- Keep edits focused. Avoid unrelated rewrites while touching gameplay, rendering, or content balance.
- If a change affects progression, combat, spawning, loot, or UI navigation, add or update a targeted test in `src/game/tests.rs` or the nearest affected module.

## Current Design Notes

- The world is infinite and deterministic, with a handcrafted safe town at the origin plus generated villages, towns, roads, landmarks, and irregular repeated biome provinces outside it.
- Biome level is based on distance from town, but biome identity is intentionally not a clean ring pattern.
- Towns unlock waypoint travel when discovered; villages are local services only.
- Monsters are spawned in nearby local packs and scale from local biome level, not player level directly.
- Character leveling grants stat points, while disciplines advance from use, unlock abilities, and feed passive bonuses such as Magic-based mana regeneration.
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

Use `lighting`, `skill-book`, or another targeted mode when changing a specific surface, and use `--preview-all` when a change may affect multiple windows or shared layout helpers.

## Common Pitfalls

- Do not judge UI changes only from the live window. Use generated previews so results are repeatable.
- Be careful when optimizing rendering. Measure or validate behavior before replacing a simple draw path with a more complex caching layer.
- Avoid making biomes purely concentric again; progression should exist without making the world feel like nested circles.
- Do not couple enemy or loot scaling to the player unless that is the explicit design change. The current game uses world/monster level as the source of progression pressure.
- Do not blur the distinction between towns and villages; waypoint travel is intentionally town-only.
- When changing layout, check long strings and low-resolution cases. This UI has already had overlap bugs from optimistic spacing.

## Product Direction

When unsure, prefer changes that make the game feel:

1. More legible
2. More responsive
3. More like a real ARPG
4. More testable and previewable

The prototype is already past the “ASCII demo” stage. Build forward from the current engine, renderer, and gameplay loop rather than falling back to terminal-style shortcuts.
