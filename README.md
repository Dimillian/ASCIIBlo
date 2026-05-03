# ASCIIBlo

![ASCIIBlo gameplay](assets/asciiblo-gameplay.png)

`ASCIIBlo` is a native Rust action RPG prototype with a graphical renderer inspired by ASCII games rather than constrained by the terminal.

Run it with:

```bash
cargo run
```

Generate deterministic UI previews without interacting with the live window:

```bash
cargo run -- --preview inventory --output /tmp/asciiblo-inventory.png
cargo run -- --preview-all /tmp/asciiblo-previews
```

Available single-screen preview modes currently include `gameplay`, `pickup`, `inventory`, `character`, `skill-book`, `world-map`, `shop-buy`, `shop-sell`, `trainer`, and `travel`.

Controls:

- Move: `WASD`
- Aim: mouse
- Basic attack: left click or `Space`
- Skills: `1` and `2` cast the two skills currently bound in the mastery screen
- Pick up loot: `E`
- Talk to NPCs: `F`
- Inventory: `Tab`, then `Up` / `Down` to move, `Enter` to equip, `Backspace` to drop
- Character sheet: `C`, then `Up` / `Down` and `Enter` to spend stat points
- Mastery screen: `B`, then `Left` / `Right` for panels, `Up` / `Down` to move, and `1` / `2` to bind
- World map: `M`, then `WASD` / arrow keys to pan, mouse wheel to zoom, `R` to recenter
- Quit: `Esc`

The current slice includes:

- Native windowed rendering with a smooth camera and layered UI
- Real-time cooldown-based movement and combat
- An infinite deterministic overworld with larger irregular biome provinces, new wilderness regions, generated settlements, roads, landmarks, and scattered props
- Visible biome levels, a framed minimap, a discovered-world map, and waypoint travel unlocked through discovered towns
- Monsters with different tempos, attacks, level scaling, biome-specific encounter tables, and hover target readouts
- Loot, item inspection, rarity, gear bonuses, inventory, stats, XP, levels, spendable stat points, use-based mastery, gold
- Villages and towns with visible floorplans, safe interiors, and tiered NPC services
- Floating damage text, crits, particles, a compact combat feed, pickup feedback, skill bursts, and level-up fireworks
- Dedicated modal UI for inventory, character progression, merchant buy/sell flows, trainer guidance, travel, and the world map

The visual language still uses glyphs, punctuation, and bright text as texture, but the game itself now runs through a real renderer instead of a terminal escape-sequence loop.
