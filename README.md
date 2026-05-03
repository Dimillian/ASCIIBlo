# ASCIIBlo

![ASCIIBlo gameplay](assets/asciiblo-gameplay.png)

`ASCIIBlo` is a native Rust action RPG prototype with a graphical renderer inspired by ASCII games rather than constrained by the terminal.

Run it with:

```bash
cargo run
```

Normal runs choose a fresh random world seed. To replay a world exactly, pass one explicitly:

```bash
cargo run -- --seed 12345
cargo run -- --seed 0xA5C1_1B10
```

Generate deterministic UI previews without interacting with the live window:

```bash
cargo run -- --preview inventory --output /tmp/asciiblo-inventory.png
cargo run -- --preview-all /tmp/asciiblo-previews
```

Preview commands keep a stable seed by default so screenshots stay reproducible; add `--seed` only when you intentionally want a different preview world.

Available single-screen preview modes currently include `gameplay`, `lighting`, `pickup`, `inventory`, `character`, `skill-book`, `world-map`, `shop-buy`, `shop-sell`, `trainer`, `travel`, and `quest`.

Controls:

- Move: `WASD`
- Aim: mouse
- Basic attack: left click or `Space`
- Skills: `1` and `2` cast the two skills currently bound in the mastery screen
- Pick up loot: `E`
- Talk to NPCs: `F`
- Use bounty boards in towns: `F`
- Inventory: `Tab`, then `Up` / `Down` to move, `Enter` to equip, `Backspace` to drop
- Character sheet: `C`, then `Up` / `Down` and `Enter` to spend stat points
- Mastery screen: `B`, then `Left` / `Right` for panels, `Up` / `Down` to move, and `1` / `2` to bind
- World map: `M`, then `WASD` / arrow keys to pan, mouse wheel to zoom, `R` to recenter
- Quit: `Esc`

The current slice includes:

- Native windowed rendering with a smooth camera and layered UI
- Real-time cooldown-based movement and combat with bindable skills, visible cooldown states, and mana-driven spellcasting
- An infinite seed-driven overworld with larger irregular biome provinces, new wilderness regions, generated settlements, roads, landmarks, and scattered props
- Visible biome levels, a framed minimap, a discovered-world map, and waypoint travel unlocked through discovered towns
- Procedural town bounty boards with one active quest at a time, kill/bounty/contact/recovery objectives, guaranteed quest-owned targets, turn-ins, rewards, and directional guidance
- Monsters with different tempos, attacks, level scaling, biome-specific encounter tables, clustered packs, elite and boss ranks, and hover target readouts
- Loot, item inspection, rarity, gear bonuses, inventory, stats, XP, levels, spendable stat points, use-based mastery, unlockable abilities, gold
- Villages and towns with visible floorplans, safe interiors, and tiered NPC services
- Floating damage text, crits, particles, colored scene lighting, item and skill glows, a compact combat feed, pickup feedback, skill bursts, and level-up fireworks
- Dedicated modal UI for inventory, character progression, merchant buy/sell flows, trainer guidance, travel, and the world map

The visual language still uses glyphs, punctuation, and bright text as texture, but the game itself now runs through a real renderer instead of a terminal escape-sequence loop.
