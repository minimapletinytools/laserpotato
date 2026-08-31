//! Bevy UI layouts, buttons, and visual panels for the Level Editor.

use bevy::prelude::*;
use crate::block_types::BlockKind;
use super::{AppMode, EditorAction, EditorState};

// ---------------------------------------------------------------------------
// UI Component Markers
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct EditorRootUi;

#[derive(Component)]
pub struct PaletteButton(pub BlockKind);

#[derive(Component)]
pub struct PropertyToggleButton(pub bool); // true = fixed, false = moveable

#[derive(Component)]
pub struct ZModeToggleButton(pub crate::editor::ZPlacementMode);

#[derive(Component)]
pub struct ZLayerIncButton;

#[derive(Component)]
pub struct ZLayerDecButton;

#[derive(Component)]
pub struct ZLayerLabelText;

#[derive(Component)]
pub struct InspectorPanel;

#[derive(Component)]
pub struct InspectorText;

#[derive(Component)]
pub struct RotateCwButton;

#[derive(Component)]
pub struct RotateCcwButton;

#[derive(Component)]
pub struct RotateXPosButton;

#[derive(Component)]
pub struct RotateXNegButton;

#[derive(Component)]
pub struct RotateYPosButton;

#[derive(Component)]
pub struct RotateYNegButton;

#[derive(Component)]
pub struct ReflectXButton;

#[derive(Component)]
pub struct ReflectYButton;

#[derive(Component)]
pub struct ToggleFixedButton;

#[derive(Component)]
pub struct DeleteBlockButton;

#[derive(Component)]
pub struct ActionButton(pub EditorAction);

#[derive(Component)]
pub struct SolverStatusBadge;

#[derive(Component)]
pub struct ToastNotificationText;

#[derive(Component)]
pub struct PalettePreviewLabel;

// ---------------------------------------------------------------------------
// Colors & Styling Constants
// ---------------------------------------------------------------------------

pub const PANEL_BG: Color = Color::srgba(0.08, 0.08, 0.12, 0.94);
pub const BTN_NORMAL: Color = Color::srgba(0.18, 0.18, 0.24, 0.90);
pub const BTN_ACTIVE: Color = Color::srgba(0.22, 0.50, 0.85, 1.0);
pub const BTN_DISABLED: Color = Color::srgba(0.12, 0.12, 0.14, 0.50);
pub const BTN_SUCCESS: Color = Color::srgba(0.15, 0.55, 0.35, 1.0);
pub const BTN_DANGER: Color = Color::srgba(0.65, 0.20, 0.20, 1.0);
pub const TEXT_PRIMARY: Color = Color::srgb(0.92, 0.92, 0.96);
pub const TEXT_MUTED: Color = Color::srgb(0.60, 0.60, 0.68);

// ---------------------------------------------------------------------------
// Setup Editor UI
// ---------------------------------------------------------------------------

pub fn setup_editor_ui(mut commands: Commands) {
    commands
        .spawn((
            EditorRootUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
        ))
        .with_children(|root| {
            // ===============================================================
            // TOP ACTION BAR
            // ===============================================================
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(52.0),
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                BackgroundColor(PANEL_BG),
            ))
            .with_children(|top_bar| {
                // Left group: File management
                top_bar
                    .spawn(Node {
                        column_gap: Val::Px(8.0),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|group| {
                        group.spawn((
                            Text::new("LASER POTATO EDITOR"),
                            TextFont::from_font_size(15.0),
                            TextColor(Color::srgb(0.9, 0.8, 0.3)),
                        ));

                        spawn_action_btn(group, EditorAction::NewLevel, "New Level");
                        spawn_action_btn(group, EditorAction::Save, "Save");
                        spawn_action_btn(group, EditorAction::SaveAs, "Save As...");
                        spawn_action_btn(group, EditorAction::ToggleLevelsMenu, "[Levels]");
                    });

                // Center group: Solver badge & Attempt to Solve
                top_bar
                    .spawn(Node {
                        column_gap: Val::Px(10.0),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|group| {
                        group.spawn((
                            SolverStatusBadge,
                            Text::new("Solver: Idle"),
                            TextFont::from_font_size(13.0),
                            TextColor(TEXT_MUTED),
                        ));
                        spawn_action_btn(group, EditorAction::AttemptSolve, "Solve Level");
                    });

                // Middle-Right group: View rotation controls
                top_bar
                    .spawn(Node {
                        column_gap: Val::Px(6.0),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|group| {
                        spawn_action_btn(group, EditorAction::RotateViewCcw, "Rot L [Q]");
                        spawn_action_btn(group, EditorAction::RotateViewCw, "Rot R [E]");
                    });

                // Right group: Playtest & Replay Mode controls
                top_bar
                    .spawn(Node {
                        column_gap: Val::Px(8.0),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|group| {
                        spawn_action_btn(group, EditorAction::TestPlay, "Test Play");
                        spawn_action_btn(group, EditorAction::TestWithSolution, "Test with Solution");
                    });
            });

            // ===============================================================
            // MAIN MIDDLE WORKSPACE (Left Palette + Right Inspector)
            // ===============================================================
            root.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            })
            .with_children(|workspace| {
                // -----------------------------------------------------------
                // LEFT SIDEBAR: BLOCK PALETTE & PROPERTY SELECTOR
                // -----------------------------------------------------------
                workspace
                    .spawn((
                        Node {
                            width: Val::Px(240.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(12.0)),
                            row_gap: Val::Px(8.0),
                            ..default()
                        },
                        BackgroundColor(PANEL_BG),
                    ))
                    .with_children(|sidebar| {
                        sidebar.spawn((
                            Text::new("BLOCK PALETTE"),
                            TextFont::from_font_size(13.0),
                            TextColor(TEXT_PRIMARY),
                        ));

                        // 3D Preview Frame Box
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
                                BorderColor::all(Color::srgba(0.3, 0.5, 0.8, 0.5)),
                                BackgroundColor(Color::srgba(0.04, 0.04, 0.06, 0.90)),
                            ))
                            .with_children(|box_node| {
                                box_node.spawn((
                                    Text::new("3D PREVIEW"),
                                    TextFont::from_font_size(10.0),
                                    TextColor(Color::srgb(0.5, 0.8, 1.0)),
                                ));
                                box_node.spawn((
                                    PalettePreviewLabel,
                                    Text::new("Mirror (Moveable)"),
                                    TextFont::from_font_size(11.0),
                                    TextColor(TEXT_PRIMARY),
                                ));
                            });

                        // Base Block Buttons
                        let blocks = [
                            (BlockKind::Player, "[P] Player"),
                            (BlockKind::Mirror, "[M] Mirror"),
                            (BlockKind::LaserSource, "[L] Laser Source"),
                            (BlockKind::Pushable, "[C] Pushable Crate"),
                            (BlockKind::Wall, "[W] Wall"),
                            (BlockKind::Goal, "[G] Goal Pyramid"),
                        ];

                        for (kind, label) in blocks {
                            sidebar
                                .spawn((
                                    PaletteButton(kind),
                                    Button,
                                    Node {
                                        width: Val::Percent(100.0),
                                        padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                                        justify_content: JustifyContent::Start,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(BTN_NORMAL),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new(label),
                                        TextFont::from_font_size(12.0),
                                        TextColor(TEXT_PRIMARY),
                                    ));
                                });
                        }

                        // Placement Property Selector
                        sidebar.spawn((
                            Text::new("PLACEMENT PROPERTY"),
                            TextFont::from_font_size(12.0),
                            TextColor(TEXT_MUTED),
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
                                        BackgroundColor(BTN_NORMAL),
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn((
                                            Text::new("Moveable"),
                                            TextFont::from_font_size(12.0),
                                            TextColor(TEXT_PRIMARY),
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
                                        BackgroundColor(BTN_NORMAL),
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn((
                                            Text::new("Stationary"),
                                            TextFont::from_font_size(12.0),
                                            TextColor(TEXT_PRIMARY),
                                        ));
                                    });
                            });

                        // Z Placement Mode Selector
                        sidebar.spawn((
                            Text::new("Z PLACEMENT"),
                            TextFont::from_font_size(12.0),
                            TextColor(TEXT_MUTED),
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
                                        ZModeToggleButton(crate::editor::ZPlacementMode::StackOnTop),
                                        Button,
                                        Node {
                                            flex_grow: 1.0,
                                            padding: UiRect::axes(Val::Px(4.0), Val::Px(6.0)),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        BackgroundColor(BTN_NORMAL),
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn((
                                            Text::new("Stack Top"),
                                            TextFont::from_font_size(11.0),
                                            TextColor(TEXT_PRIMARY),
                                        ));
                                    });

                                z_mode_row
                                    .spawn((
                                        ZModeToggleButton(crate::editor::ZPlacementMode::FixedLayer),
                                        Button,
                                        Node {
                                            flex_grow: 1.0,
                                            padding: UiRect::axes(Val::Px(4.0), Val::Px(6.0)),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        BackgroundColor(BTN_NORMAL),
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn((
                                            Text::new("Grid Layer"),
                                            TextFont::from_font_size(11.0),
                                            TextColor(TEXT_PRIMARY),
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
                                        BackgroundColor(BTN_NORMAL),
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn((
                                            Text::new("-"),
                                            TextFont::from_font_size(13.0),
                                            TextColor(TEXT_PRIMARY),
                                        ));
                                    });

                                layer_row
                                    .spawn((
                                        Node {
                                            flex_grow: 1.0,
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                    ))
                                    .with_children(|cell| {
                                        cell.spawn((
                                            ZLayerLabelText,
                                            Text::new("Layer Z: 0"),
                                            TextFont::from_font_size(12.0),
                                            TextColor(TEXT_PRIMARY),
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
                                        BackgroundColor(BTN_NORMAL),
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn((
                                            Text::new("+"),
                                            TextFont::from_font_size(13.0),
                                            TextColor(TEXT_PRIMARY),
                                        ));
                                    });
                            });

                        // Instructions / shortcuts hint
                        sidebar.spawn((
                            Text::new("Controls:\n- Esc: Select Mode / Deselect\n- L-Click: Place / Select\n- Drag: Move Block\n- R-Click: Delete Block\n- Tab: Toggle Z Mode\n- PgUp/PgDn: Change Z\n- Q / E: Rotate View 90 deg\n- WASD: Pan Camera\n- Scroll: Zoom In/Out"),
                            TextFont::from_font_size(11.0),
                            TextColor(TEXT_MUTED),
                        ));
                    });

                // -----------------------------------------------------------
                // RIGHT FLOATING INSPECTOR PANEL (Visible when block is selected)
                // -----------------------------------------------------------
                workspace
                    .spawn((
                        InspectorPanel,
                        Node {
                            width: Val::Px(240.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(12.0)),
                            row_gap: Val::Px(10.0),
                            ..default()
                        },
                        BackgroundColor(PANEL_BG),
                    ))
                    .with_children(|inspector| {
                        inspector.spawn((
                            Text::new("BLOCK INSPECTOR"),
                            TextFont::from_font_size(13.0),
                            TextColor(TEXT_PRIMARY),
                        ));

                        inspector.spawn((
                            InspectorText,
                            Text::new("Click a block in the 3D grid to select and modify its properties."),
                            TextFont::from_font_size(12.0),
                            TextColor(TEXT_MUTED),
                        ));

                        // 3D Pitch (World X-axis tilt)
                        inspector
                            .spawn(Node {
                                width: Val::Percent(100.0),
                                column_gap: Val::Px(6.0),
                                ..default()
                            })
                            .with_children(|row| {
                                row.spawn((
                                    RotateXPosButton,
                                    Button,
                                    Node {
                                        flex_grow: 1.0,
                                        padding: UiRect::axes(Val::Px(6.0), Val::Px(6.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(BTN_NORMAL),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("Pitch +X"),
                                        TextFont::from_font_size(11.0),
                                        TextColor(TEXT_PRIMARY),
                                    ));
                                });

                                row.spawn((
                                    RotateXNegButton,
                                    Button,
                                    Node {
                                        flex_grow: 1.0,
                                        padding: UiRect::axes(Val::Px(6.0), Val::Px(6.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(BTN_NORMAL),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("Pitch -X"),
                                        TextFont::from_font_size(11.0),
                                        TextColor(TEXT_PRIMARY),
                                    ));
                                });
                            });

                        // 3D Roll (World Y-axis tilt)
                        inspector
                            .spawn(Node {
                                width: Val::Percent(100.0),
                                column_gap: Val::Px(6.0),
                                ..default()
                            })
                            .with_children(|row| {
                                row.spawn((
                                    RotateYPosButton,
                                    Button,
                                    Node {
                                        flex_grow: 1.0,
                                        padding: UiRect::axes(Val::Px(6.0), Val::Px(6.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(BTN_NORMAL),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("Roll +Y"),
                                        TextFont::from_font_size(11.0),
                                        TextColor(TEXT_PRIMARY),
                                    ));
                                });

                                row.spawn((
                                    RotateYNegButton,
                                    Button,
                                    Node {
                                        flex_grow: 1.0,
                                        padding: UiRect::axes(Val::Px(6.0), Val::Px(6.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(BTN_NORMAL),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("Roll -Y"),
                                        TextFont::from_font_size(11.0),
                                        TextColor(TEXT_PRIMARY),
                                    ));
                                });
                            });

                        // 2D Rotation Controls (Yaw / Z-axis)
                        inspector
                            .spawn(Node {
                                width: Val::Percent(100.0),
                                column_gap: Val::Px(6.0),
                                ..default()
                            })
                            .with_children(|row| {
                                row.spawn((
                                    RotateCcwButton,
                                    Button,
                                    Node {
                                        flex_grow: 1.0,
                                        padding: UiRect::axes(Val::Px(6.0), Val::Px(6.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(BTN_NORMAL),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("Rot Z CCW"),
                                        TextFont::from_font_size(11.0),
                                        TextColor(TEXT_PRIMARY),
                                    ));
                                });

                                row.spawn((
                                    RotateCwButton,
                                    Button,
                                    Node {
                                        flex_grow: 1.0,
                                        padding: UiRect::axes(Val::Px(6.0), Val::Px(6.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(BTN_NORMAL),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("Rot Z [Key: R]"),
                                        TextFont::from_font_size(11.0),
                                        TextColor(TEXT_PRIMARY),
                                    ));
                                });
                            });

                        // Spatial Reflection Controls (48 Oh symmetry group)
                        inspector
                            .spawn(Node {
                                width: Val::Percent(100.0),
                                column_gap: Val::Px(6.0),
                                ..default()
                            })
                            .with_children(|row| {
                                row.spawn((
                                    ReflectXButton,
                                    Button,
                                    Node {
                                        flex_grow: 1.0,
                                        padding: UiRect::axes(Val::Px(6.0), Val::Px(6.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(BTN_NORMAL),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("Flip X [Key: X]"),
                                        TextFont::from_font_size(11.0),
                                        TextColor(TEXT_PRIMARY),
                                    ));
                                });

                                row.spawn((
                                    ReflectYButton,
                                    Button,
                                    Node {
                                        flex_grow: 1.0,
                                        padding: UiRect::axes(Val::Px(6.0), Val::Px(6.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(BTN_NORMAL),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("Flip Y [Key: Y]"),
                                        TextFont::from_font_size(11.0),
                                        TextColor(TEXT_PRIMARY),
                                    ));
                                });
                            });

                        // Property Toggle Button
                        inspector
                            .spawn((
                                ToggleFixedButton,
                                Button,
                                Node {
                                    width: Val::Percent(100.0),
                                    padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(BTN_NORMAL),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Toggle Fixed / Moveable"),
                                    TextFont::from_font_size(12.0),
                                    TextColor(TEXT_PRIMARY),
                                ));
                            });

                        // Delete Block Button
                        inspector
                            .spawn((
                                DeleteBlockButton,
                                Button,
                                Node {
                                    width: Val::Percent(100.0),
                                    padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(BTN_DANGER),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Delete Block"),
                                    TextFont::from_font_size(12.0),
                                    TextColor(TEXT_PRIMARY),
                                ));
                            });
                    });
            });

            // ===============================================================
            // BOTTOM STATUS & TOAST BAR
            // ===============================================================
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(28.0),
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(4.0)),
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(PANEL_BG),
            ))
            .with_children(|bottom| {
                bottom.spawn((
                    ToastNotificationText,
                    Text::new("Ready. Select block from palette to place on grid."),
                    TextFont::from_font_size(12.0),
                    TextColor(TEXT_MUTED),
                ));
            });
        });
}

fn spawn_action_btn(parent: &mut ChildSpawnerCommands, action: EditorAction, label: &str) {
    parent
        .spawn((
            ActionButton(action),
            Button,
            Node {
                padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(BTN_NORMAL),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont::from_font_size(12.0),
                TextColor(TEXT_PRIMARY),
            ));
        });
}

// ---------------------------------------------------------------------------
// Dynamic UI Update System
// ---------------------------------------------------------------------------

pub fn update_editor_ui_system(
    app_mode: Res<State<AppMode>>,
    editor: Res<EditorState>,
    game: Res<crate::GameState>,
    mut root_query: Query<&mut Visibility, With<EditorRootUi>>,
    mut palette_query: Query<(&PaletteButton, &mut BackgroundColor), (Without<PropertyToggleButton>, Without<ZModeToggleButton>)>,
    mut prop_query: Query<(&PropertyToggleButton, &mut BackgroundColor), (Without<PaletteButton>, Without<ZModeToggleButton>)>,
    mut z_mode_query: Query<(&ZModeToggleButton, &mut BackgroundColor), (Without<PaletteButton>, Without<PropertyToggleButton>, Without<ActionButton>)>,
    mut z_layer_query: Query<&mut Text, (With<ZLayerLabelText>, Without<InspectorText>, Without<SolverStatusBadge>, Without<ToastNotificationText>, Without<PalettePreviewLabel>)>,
    mut inspector_text_query: Query<&mut Text, (With<InspectorText>, Without<SolverStatusBadge>, Without<ToastNotificationText>, Without<PalettePreviewLabel>, Without<ZLayerLabelText>)>,
    mut solver_badge_query: Query<(&mut Text, &mut TextColor), (With<SolverStatusBadge>, Without<InspectorText>, Without<ToastNotificationText>, Without<PalettePreviewLabel>, Without<ZLayerLabelText>)>,
    mut toast_query: Query<&mut Text, (With<ToastNotificationText>, Without<InspectorText>, Without<SolverStatusBadge>, Without<PalettePreviewLabel>, Without<ZLayerLabelText>)>,
    mut preview_label_query: Query<&mut Text, (With<PalettePreviewLabel>, Without<InspectorText>, Without<SolverStatusBadge>, Without<ToastNotificationText>, Without<ZLayerLabelText>)>,
    mut action_btns_query: Query<(&ActionButton, &mut BackgroundColor), (Without<PaletteButton>, Without<PropertyToggleButton>, Without<ZModeToggleButton>)>,
) {
    // Show UI only in Editor mode
    for mut vis in &mut root_query {
        *vis = if *app_mode.get() == AppMode::Editor {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    if *app_mode.get() != AppMode::Editor {
        return;
    }

    // 1. Update Palette 3D Preview Label
    for mut text in &mut preview_label_query {
        if let Some(kind) = editor.selected_kind {
            let (can_moveable, can_fixed) = editor.allowed_fixed_state(kind);
            let is_fixed = if !can_moveable {
                true
            } else if !can_fixed {
                false
            } else {
                editor.is_fixed
            };
            let prop_str = if is_fixed { "Stationary" } else { "Moveable" };
            let icon_name = match kind {
                BlockKind::Player => "Player".into(),
                BlockKind::Mirror => format!("Mirror ({})", prop_str),
                BlockKind::LaserSource => format!("Laser Source ({})", prop_str),
                BlockKind::Pushable => format!("Pushable Crate ({})", prop_str),
                BlockKind::Wall => "Wall (Stationary)".into(),
                BlockKind::Floor => "Floor (Stationary)".into(),
                BlockKind::Goal => format!("Goal Pyramid ({})", prop_str),
                BlockKind::Glass => format!("Glass Block ({})", prop_str),
            };
            text.0 = icon_name;
        } else {
            text.0 = "Select-Only Mode [Esc]".into();
        }
    }

    // 2. Highlight active palette button
    for (palette_btn, mut bg) in &mut palette_query {
        bg.0 = if Some(palette_btn.0) == editor.selected_kind {
            BTN_ACTIVE
        } else {
            BTN_NORMAL
        };
    }

    // 3. Highlight active property button & disable invalid ones
    if let Some(kind) = editor.selected_kind {
        let (can_moveable, can_fixed) = editor.allowed_fixed_state(kind);
        for (prop_btn, mut bg) in &mut prop_query {
            let is_fixed_btn = prop_btn.0;
            if is_fixed_btn {
                if !can_fixed {
                    bg.0 = BTN_DISABLED;
                } else if editor.is_fixed {
                    bg.0 = BTN_ACTIVE;
                } else {
                    bg.0 = BTN_NORMAL;
                }
            } else {
                if !can_moveable {
                    bg.0 = BTN_DISABLED;
                } else if !editor.is_fixed {
                    bg.0 = BTN_ACTIVE;
                } else {
                    bg.0 = BTN_NORMAL;
                }
            }
        }
    } else {
        for (_, mut bg) in &mut prop_query {
            bg.0 = BTN_NORMAL;
        }
    }

    // 4. Highlight active Z placement mode button
    for (z_btn, mut bg) in &mut z_mode_query {
        bg.0 = if z_btn.0 == editor.z_mode {
            BTN_ACTIVE
        } else {
            BTN_NORMAL
        };
    }

    // 5. Update Z layer label text
    for mut text in &mut z_layer_query {
        text.0 = format!("Layer Z: {}", editor.current_z);
    }

    // 6. Update Inspector details
    for mut text in &mut inspector_text_query {
        if let Some(body_id) = editor.selected_body_id {
            if let Some(body) = game.engine.world.body(body_id) {
                let fixed_str = if body.is_fixed() { "Stationary (Fixed)" } else { "Moveable" };
                let sym_str = if body.orientation.is_reflection() { "Reflected (Chiral)" } else { "Proper Rotation" };
                text.0 = format!(
                    "Type: {:?}\nPosition: ({}, {}, {})\nStatus: {}\nSymmetry: {} (det={})",
                    body.kind, body.anchor.x, body.anchor.y, body.anchor.z, fixed_str, sym_str, body.orientation.det()
                );
            } else {
                text.0 = "No block selected.\nClick a block in the grid to inspect.".into();
            }
        } else {
            text.0 = "No block selected.\nClick a block in the grid to inspect.".into();
        }
    }

    // 5. Update Solver Status Badge
    for (mut text, mut color) in &mut solver_badge_query {
        text.0 = format!("Solver: {}", editor.solver_status);
        if editor.solver_status.starts_with('✓') {
            color.0 = Color::srgb(0.3, 1.0, 0.6);
        } else if editor.solver_status.starts_with('✗') {
            color.0 = Color::srgb(1.0, 0.4, 0.4);
        } else if editor.solver_status.starts_with("Solving") {
            color.0 = Color::srgb(1.0, 0.8, 0.2);
        } else {
            color.0 = TEXT_MUTED;
        }
    }

    // 6. Update Toast / Status Bar
    let current_hash = crate::level::compute_level_hash(&game.engine.world);
    let cached_valid = editor
        .cached_solution
        .as_ref()
        .map(|(h, _)| *h == current_hash)
        .unwrap_or(false);

    for mut text in &mut toast_query {
        if let Some((msg, _)) = &editor.status_message {
            text.0 = msg.clone();
        } else {
            text.0 = format!(
                "Level: {} | Hash: 0x{:08x} | Blocks: {} | Solution Ready: {}",
                editor.current_level_path,
                current_hash,
                game.engine.world.bodies().len(),
                if cached_valid { "YES" } else { "NO" }
            );
        }
    }

    // 7. Highlight "Test with Solution" button if solution is ready
    for (action_btn, mut bg) in &mut action_btns_query {
        if action_btn.0 == EditorAction::TestWithSolution {
            bg.0 = if cached_valid {
                BTN_SUCCESS
            } else {
                BTN_DISABLED
            };
        }
    }
}
