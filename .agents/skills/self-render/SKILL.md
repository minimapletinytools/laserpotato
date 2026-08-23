---
name: self-render
description: Run the Laser Potato engine in automated screenshot mode, capture in-engine 3D graphics/UI to a PNG, and inspect the rendered image with view_file to verify visual rendering, culling, and materials.
---

# Self-Render and Visual Verification Skill

This skill allows agents to independently run the *Laser Potato* 3D game engine and level editor, capture a pixel-perfect in-engine screenshot to disk, and visually inspect the rendered output using the `view_file` tool.

## When to Use

Activate this skill when you need to:
- Visually verify 3D procedural meshes, triangle winding order, and back-face culling.
- Check PBR materials, textures (such as polka dot patterns), and dynamic shader colors.
- Inspect laser ray propagation, reflection angles, beam core/sheath meshes, and hit flares.
- Validate Bevy UI layouts, button placements, inspector panels, and HUD banners.
- Confirm level editor grid gizmos, axis coordinate labels, and 3D preview widgets.

## How to Capture a Screenshot

Run the game binary with the `--screenshot` CLI argument:

```bash
# Capture the Level Editor view (default)
cargo run --bin laserpotato -- --screenshot /tmp/game_render.png --frames 20

# Capture Playtest mode
cargo run --bin laserpotato -- --screenshot /tmp/game_playtest.png --playtest --frames 20

# Capture Solution Playback mode
cargo run --bin laserpotato -- --screenshot /tmp/game_playback.png --replay solution.json --frames 20
```

### CLI Options
- `-s, --screenshot <path>`: Specifies output PNG path and activates automated capture mode.
- `--frames <N>`: Number of frames to simulate before capturing (default `20` to ensure all assets and lighting clusters initialize).
- `--playtest`: Launches directly into Playtest mode.
- `--editor`: Launches directly into Level Editor mode.
- `-r, --replay <file>`: Loads a JSON move sequence and launches in Playback mode.

## How to Inspect the Rendered Screenshot

Call the `view_file` tool with the absolute path of the generated `.png` image:

```json
{
  "AbsolutePath": "/tmp/game_render.png",
  "toolAction": "Viewing in-engine screenshot",
  "toolSummary": "Inspect self-rendered screenshot"
}
```

The image will be rendered directly into the conversation context for visual verification.
