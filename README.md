# ⚡ Laser Potato

A 3D laser puzzle game and level editor built with Rust and [Bevy](https://bevyengine.org/).

Navigate grid puzzles by pushing blocks, angling mirrors, and routing laser beams into target pyramids. Includes a full in-engine 3D level editor, automated puzzle solver, and standalone web player.

---

## 🎮 Running the Game

### 1. Level Editor & Engine (Default Target)
```bash
cargo run
```
Features:
- Full 3D grid editing, block placement (mirrors, crates, walls, laser sources, goal pyramids).
- Multi-layer Z stacking and floorplan resize/fill tools.
- Built-in multi-threaded solver with solution recording and automated playback.
- Interactive playtest mode.

### 2. Standalone Player & Level Picker
```bash
# Level Select Menu
cargo run --bin play

# Direct Level Bypass (CLI)
cargo run --bin play -- simple_1
cargo run --bin play -- levels/default_puzzle.json
```

---

## 🌐 Web / WebAssembly Version

### Local Web Testing
```bash
# With Trunk (live reload):
trunk serve --port 8080 --open

# Or build static release bundle:
./scripts/build_web.sh
python3 -m http.server 8080 -d dist
```

### URL Parameter Level Bypass
When running in the browser, pass `?level=<name_or_index>` to launch directly into a puzzle:
- `http://localhost:8080/?level=simple_1`
- `http://localhost:8080/?level=2`

---

## 🕹️ Controls

| Action | Key / Input |
| :--- | :--- |
| **Move & Push** | `W` / `S` / `Up` / `Down` |
| **Turn Left / Right** | `A` / `D` / `Left` / `Right` |
| **Undo Move** | `Z` |
| **Reset Puzzle** | `R` |
| **Return to Menu** | `Esc` |
| **Tilt 3D View** | Mouse Left-Click + Drag |
| **Zoom Camera** | Mouse Scroll Wheel |

---

## 📜 License

This project is licensed under the **GNU General Public License v3.0 or later** ([GPL-3.0-or-later](LICENSE)). See the [LICENSE](LICENSE) file for details.
