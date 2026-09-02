use bevy::prelude::*;
use crate::block_types::BlockKind;
use crate::editor::ZPlacementMode;
use crate::editor::ui::{
    theme, EditorLeftSidebar, PaletteButton, PalettePreviewLabel, PropertyToggleButton,
    ZLayerDecButton, ZLayerIncButton, ZLayerLabelText, ZModeToggleButton,
};

/// Spawn the Left Palette Panel into the editor workspace.
pub fn spawn_palette_panel(workspace: &mut ChildSpawnerCommands) {
    workspace
        .spawn((
            EditorLeftSidebar,
            Node {
                width: Val::Px(240.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
        ))
        .with_children(|sidebar| {
            sidebar.spawn((
                Text::new("BLOCK PALETTE"),
                TextFont::from_font_size(13.0),
                TextColor(theme::TEXT_PRIMARY),
            ));

            // 3D Preview Box Container
            sidebar
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(80.0),
                        padding: UiRect::all(Val::Px(6.0)),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Start,
                        flex_direction: FlexDirection::Column,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor::all(theme::BORDER_CARD),
                    BackgroundColor(theme::OVERLAY_BACKDROP),
                ))
                .with_children(|box_node| {
                    box_node.spawn((
                        Text::new("3D PREVIEW"),
                        TextFont::from_font_size(10.0),
                        TextColor(theme::TEXT_ACCENT),
                    ));
                    box_node.spawn((
                        PalettePreviewLabel,
                        Text::new("Mirror (Moveable)"),
                        TextFont::from_font_size(11.0),
                        TextColor(theme::TEXT_PRIMARY),
                    ));
                });

            // 1. Select Tool Button
            sidebar
                .spawn((
                    PaletteButton(None),
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        padding: UiRect::axes(Val::Px(8.0), Val::Px(5.0)),
                        justify_content: JustifyContent::Start,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(theme::BTN_NORMAL),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("[S] Select"),
                        TextFont::from_font_size(12.0),
                        TextColor(theme::TEXT_PRIMARY),
                    ));
                });

            // 2. Base Block Buttons (including Floor and Glass)
            let blocks = [
                (BlockKind::Player, "[P] Player"),
                (BlockKind::Mirror, "[M] Mirror"),
                (BlockKind::LaserSource, "[L] Laser Source"),
                (BlockKind::Pushable, "[C] Pushable Crate"),
                (BlockKind::Wall, "[W] Wall"),
                (BlockKind::Floor, "[F] Floor"),
                (BlockKind::Glass, "[K] Glass Block"),
                (BlockKind::Goal, "[G] Goal Pyramid"),
            ];

            for (kind, label) in blocks {
                sidebar
                    .spawn((
                        PaletteButton(Some(kind)),
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(5.0)),
                            justify_content: JustifyContent::Start,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(theme::BTN_NORMAL),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(label),
                            TextFont::from_font_size(12.0),
                            TextColor(theme::TEXT_PRIMARY),
                        ));
                    });
            }

            // Placement Property Selector
            sidebar.spawn((
                Text::new("PLACEMENT PROPERTY"),
                TextFont::from_font_size(12.0),
                TextColor(theme::TEXT_MUTED),
            ));

            sidebar
                .spawn(Node {
                    width: Val::Percent(100.0),
                    column_gap: Val::Px(6.0),
                    ..default()
                })
                .with_children(|prop_row| {
                    prop_row
                        .spawn((
                            PropertyToggleButton(false), // Moveable
                            Button,
                            Node {
                                flex_grow: 1.0,
                                padding: UiRect::axes(Val::Px(6.0), Val::Px(6.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(theme::BTN_NORMAL),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("Moveable"),
                                TextFont::from_font_size(12.0),
                                TextColor(theme::TEXT_PRIMARY),
                            ));
                        });

                    prop_row
                        .spawn((
                            PropertyToggleButton(true), // Stationary
                            Button,
                            Node {
                                flex_grow: 1.0,
                                padding: UiRect::axes(Val::Px(6.0), Val::Px(6.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(theme::BTN_NORMAL),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("Stationary"),
                                TextFont::from_font_size(12.0),
                                TextColor(theme::TEXT_PRIMARY),
                            ));
                        });
                });

            // Z Placement Mode Selector
            sidebar.spawn((
                Text::new("Z PLACEMENT"),
                TextFont::from_font_size(12.0),
                TextColor(theme::TEXT_MUTED),
            ));

            sidebar
                .spawn(Node {
                    width: Val::Percent(100.0),
                    column_gap: Val::Px(6.0),
                    ..default()
                })
                .with_children(|z_mode_row| {
                    z_mode_row
                        .spawn((
                            ZModeToggleButton(ZPlacementMode::StackOnTop),
                            Button,
                            Node {
                                flex_grow: 1.0,
                                padding: UiRect::axes(Val::Px(4.0), Val::Px(6.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(theme::BTN_NORMAL),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("Stack Top"),
                                TextFont::from_font_size(11.0),
                                TextColor(theme::TEXT_PRIMARY),
                            ));
                        });

                    z_mode_row
                        .spawn((
                            ZModeToggleButton(ZPlacementMode::FixedLayer),
                            Button,
                            Node {
                                flex_grow: 1.0,
                                padding: UiRect::axes(Val::Px(4.0), Val::Px(6.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(theme::BTN_NORMAL),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("Grid Layer"),
                                TextFont::from_font_size(11.0),
                                TextColor(theme::TEXT_PRIMARY),
                            ));
                        });
                });

            // Layer Level Stepper ([-] Layer Z: 0 [+])
            sidebar
                .spawn(Node {
                    width: Val::Percent(100.0),
                    column_gap: Val::Px(6.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|layer_row| {
                    layer_row
                        .spawn((
                            ZLayerDecButton,
                            Button,
                            Node {
                                width: Val::Px(30.0),
                                padding: UiRect::axes(Val::Px(4.0), Val::Px(4.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(theme::BTN_NORMAL),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("-"),
                                TextFont::from_font_size(13.0),
                                TextColor(theme::TEXT_PRIMARY),
                            ));
                        });

                    layer_row
                        .spawn((Node {
                            flex_grow: 1.0,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },))
                        .with_children(|cell| {
                            cell.spawn((
                                ZLayerLabelText,
                                Text::new("Layer Z: 0"),
                                TextFont::from_font_size(12.0),
                                TextColor(theme::TEXT_PRIMARY),
                            ));
                        });

                    layer_row
                        .spawn((
                            ZLayerIncButton,
                            Button,
                            Node {
                                width: Val::Px(30.0),
                                padding: UiRect::axes(Val::Px(4.0), Val::Px(4.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(theme::BTN_NORMAL),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("+"),
                                TextFont::from_font_size(13.0),
                                TextColor(theme::TEXT_PRIMARY),
                            ));
                        });
                });

            // Instructions / shortcuts hint
            sidebar.spawn((
                Text::new("Controls:\n- ⌘Z / ⇧⌘Z: Undo / Redo\n- ⌘S: Save Level\n- Shift+Click: Multi-Select\n- Box Drag (Layer): Multi-Select\n- Esc: Select Mode / Clear\n- L-Click: Place / Select\n- Drag (Stack): Move Block\n- R-Click: Delete Block\n- Tab: Toggle Z Mode\n- PgUp/PgDn: Change Z\n- Q / E: Rotate View 90 deg\n- WASD: Pan Camera"),
                TextFont::from_font_size(11.0),
                TextColor(theme::TEXT_MUTED),
            ));
        });
}
