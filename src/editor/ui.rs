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
pub struct CombineButton;

#[derive(Component)]
pub struct UncombineButton;

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

#[derive(Component)]
pub struct ValidationErrorBanner;

#[derive(Component)]
pub struct ValidationErrorText;

#[derive(Component)]
pub struct FloorplanModal;

#[derive(Component)]
pub struct FloorplanWidthDecBtn;

#[derive(Component)]
pub struct FloorplanWidthIncBtn;

#[derive(Component)]
pub struct FloorplanWidthLabel;

#[derive(Component)]
pub struct FloorplanHeightDecBtn;

#[derive(Component)]
pub struct FloorplanHeightIncBtn;

#[derive(Component)]
pub struct FloorplanHeightLabel;

#[derive(Component)]
pub struct FloorplanZDecBtn;

#[derive(Component)]
pub struct FloorplanZIncBtn;

#[derive(Component)]
pub struct FloorplanZLabel;

#[derive(Component)]
pub struct FloorplanFillBtn;

#[derive(Component)]
pub struct FloorplanLockToggleBtn;

#[derive(Component)]
pub struct FloorplanLockToggleText;

#[derive(Component)]
pub struct FloorplanCloseBtn;

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
                // Left group: File management & Floorplan tool
                top_bar
                    .spawn(Node {
                        column_gap: Val::Px(8.0),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|group| {
                        group.spawn((
                            Text::new("LASER POTATO"),
                            TextFont::from_font_size(15.0),
                            TextColor(Color::srgb(0.9, 0.8, 0.3)),
                        ));

                        spawn_action_btn(group, EditorAction::NewLevel, "New");
                        spawn_action_btn(group, EditorAction::Save, "Save");
                        spawn_action_btn(group, EditorAction::SaveAs, "Save As");
                        spawn_action_btn(group, EditorAction::ToggleLevelsMenu, "[Levels]");
                        spawn_action_btn(group, EditorAction::ToggleFloorplanModal, "Floorplan");
                        spawn_action_btn(group, EditorAction::ToggleFramePreview, "Preview: 1");
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
            // VALIDATION ERROR BANNER (Visible when Frame 1 resolution fails)
            // ===============================================================
            root.spawn((
                ValidationErrorBanner,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(28.0),
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(4.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.85, 0.15, 0.15, 0.95)),
            ))
            .with_children(|banner| {
                banner.spawn((
                    ValidationErrorText,
                    Text::new("⚠ LEVEL INVALID: spontaneous movement detected at Frame 1. Frame 1 preview disabled."),
                    TextFont::from_font_size(12.0),
                    TextColor(Color::srgb(1.0, 1.0, 1.0)),
                ));
            });

            // ===============================================================
            // FLOATING FLOORPLAN & LEVEL SIZE MODAL
            // ===============================================================
            root.spawn((
                FloorplanModal,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(260.0),
                    top: Val::Px(60.0),
                    width: Val::Px(280.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(12.0)),
                    row_gap: Val::Px(8.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BorderColor::all(Color::srgba(0.4, 0.6, 0.9, 0.8)),
                BackgroundColor(Color::srgba(0.06, 0.07, 0.10, 0.98)),
            ))
            .with_children(|modal| {
                modal.spawn((
                    Text::new("FLOORPLAN & LEVEL SIZE"),
                    TextFont::from_font_size(13.0),
                    TextColor(Color::srgb(0.9, 0.8, 0.3)),
                ));

                // Width Row
                modal
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            FloorplanWidthDecBtn,
                            Button,
                            Node {
                                width: Val::Px(28.0),
                                padding: UiRect::all(Val::Px(4.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(BTN_NORMAL),
                        ))
                        .with_children(|b| {
                            b.spawn((Text::new("-"), TextFont::from_font_size(12.0), TextColor(TEXT_PRIMARY)));
                        });

                        row.spawn((
                            FloorplanWidthLabel,
                            Text::new("Width: 10"),
                            TextFont::from_font_size(12.0),
                            TextColor(TEXT_PRIMARY),
                        ));

                        row.spawn((
                            FloorplanWidthIncBtn,
                            Button,
                            Node {
                                width: Val::Px(28.0),
                                padding: UiRect::all(Val::Px(4.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(BTN_NORMAL),
                        ))
                        .with_children(|b| {
                            b.spawn((Text::new("+"), TextFont::from_font_size(12.0), TextColor(TEXT_PRIMARY)));
                        });
                    });

                // Height Row
                modal
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            FloorplanHeightDecBtn,
                            Button,
                            Node {
                                width: Val::Px(28.0),
                                padding: UiRect::all(Val::Px(4.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(BTN_NORMAL),
                        ))
                        .with_children(|b| {
                            b.spawn((Text::new("-"), TextFont::from_font_size(12.0), TextColor(TEXT_PRIMARY)));
                        });

                        row.spawn((
                            FloorplanHeightLabel,
                            Text::new("Height: 10"),
                            TextFont::from_font_size(12.0),
                            TextColor(TEXT_PRIMARY),
                        ));

                        row.spawn((
                            FloorplanHeightIncBtn,
                            Button,
                            Node {
                                width: Val::Px(28.0),
                                padding: UiRect::all(Val::Px(4.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(BTN_NORMAL),
                        ))
                        .with_children(|b| {
                            b.spawn((Text::new("+"), TextFont::from_font_size(12.0), TextColor(TEXT_PRIMARY)));
                        });
                    });

                // Floor Z Row
                modal
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            FloorplanZDecBtn,
                            Button,
                            Node {
                                width: Val::Px(28.0),
                                padding: UiRect::all(Val::Px(4.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(BTN_NORMAL),
                        ))
                        .with_children(|b| {
                            b.spawn((Text::new("-"), TextFont::from_font_size(12.0), TextColor(TEXT_PRIMARY)));
                        });

                        row.spawn((
                            FloorplanZLabel,
                            Text::new("Floor Z: -1"),
                            TextFont::from_font_size(12.0),
                            TextColor(TEXT_PRIMARY),
                        ));

                        row.spawn((
                            FloorplanZIncBtn,
                            Button,
                            Node {
                                width: Val::Px(28.0),
                                padding: UiRect::all(Val::Px(4.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(BTN_NORMAL),
                        ))
                        .with_children(|b| {
                            b.spawn((Text::new("+"), TextFont::from_font_size(12.0), TextColor(TEXT_PRIMARY)));
                        });
                    });

                // Fill Floor Action Button
                modal
                    .spawn((
                        FloorplanFillBtn,
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(BTN_ACTIVE),
                    ))
                    .with_children(|b| {
                        b.spawn((Text::new("Fill Floor Area"), TextFont::from_font_size(12.0), TextColor(TEXT_PRIMARY)));
                    });

                // Lock Floor Toggle Button
                modal
                    .spawn((
                        FloorplanLockToggleBtn,
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
                    .with_children(|b| {
                        b.spawn((FloorplanLockToggleText, Text::new("Lock Floor Layer (Z=-1)"), TextFont::from_font_size(12.0), TextColor(TEXT_PRIMARY)));
                    });

                // Close Button
                modal
                    .spawn((
                        FloorplanCloseBtn,
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(BTN_NORMAL),
                    ))
                    .with_children(|b| {
                        b.spawn((Text::new("Close"), TextFont::from_font_size(11.0), TextColor(TEXT_MUTED)));
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

                        // Base Block Buttons (including Floor and Glass)
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
                                    PaletteButton(kind),
                                    Button,
                                    Node {
                                        width: Val::Percent(100.0),
                                        padding: UiRect::axes(Val::Px(8.0), Val::Px(5.0)),
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
                            Text::new("Controls:\n- Shift+Click: Multi-Select\n- Box Drag (Layer): Multi-Select\n- Esc: Select Mode / Clear\n- L-Click: Place / Select\n- Drag (Stack): Move Block\n- R-Click: Delete Block\n- Tab: Toggle Z Mode\n- PgUp/PgDn: Change Z\n- Q / E: Rotate View 90°\n- WASD: Pan Camera"),
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
                            row_gap: Val::Px(8.0),
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
                            TextFont::from_font_size(11.0),
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
                                        Text::new("Pitch +X [T]"),
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
                                        Text::new("Roll +Y [G]"),
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
                                        Text::new("Rot CCW"),
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
                                        Text::new("Rot CW [R]"),
                                        TextFont::from_font_size(11.0),
                                        TextColor(TEXT_PRIMARY),
                                    ));
                                });
                            });

                        // Spatial Reflection Controls
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
                                        Text::new("Flip X [X]"),
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
                                        Text::new("Flip Y [Y]"),
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
                                    TextFont::from_font_size(11.0),
                                    TextColor(TEXT_PRIMARY),
                                ));
                            });

                        // Combine Mega Block Button
                        inspector
                            .spawn((
                                CombineButton,
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
                                    Text::new("Combine [Mega Block]"),
                                    TextFont::from_font_size(11.0),
                                    TextColor(TEXT_PRIMARY),
                                ));
                            });

                        // Uncombine Button
                        inspector
                            .spawn((
                                UncombineButton,
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
                                    Text::new("Uncombine Group"),
                                    TextFont::from_font_size(11.0),
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
                                    Text::new("Delete Block(s) [Del]"),
                                    TextFont::from_font_size(11.0),
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
    mut text_query: Query<(
        &mut Text,
        Option<&PalettePreviewLabel>,
        Option<&ZLayerLabelText>,
        Option<&InspectorText>,
    )>,
    mut button_query: Query<(
        &mut BackgroundColor,
        Option<&PaletteButton>,
        Option<&PropertyToggleButton>,
        Option<&ZModeToggleButton>,
        Option<&CombineButton>,
        Option<&UncombineButton>,
    )>,
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

    // 1. Update text elements (Preview, Z layer, Inspector)
    let selected_count = editor.selected_body_ids.len();
    for (mut text, preview_opt, z_layer_opt, inspector_opt) in &mut text_query {
        if preview_opt.is_some() {
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
        } else if z_layer_opt.is_some() {
            let locked_tag = if editor.is_layer_locked(editor.current_z) { " [LOCKED]" } else { "" };
            text.0 = format!("Layer Z: {}{}", editor.current_z, locked_tag);
        } else if inspector_opt.is_some() {
            if selected_count == 0 {
                text.0 = "No block selected.\nClick or drag to select blocks in the grid.".into();
            } else if selected_count == 1 {
                let body_id = editor.selected_body_ids[0];
                if let Some(body) = game.engine.world.body(body_id) {
                    let fixed_str = if body.is_fixed() { "Stationary" } else { "Moveable" };
                    let sym_str = if body.orientation.is_reflection() { "Reflected" } else { "Rotation" };
                    let grp_str = if let Some(gid) = body.combined_group {
                        format!(" | Group #{}", gid)
                    } else {
                        "".into()
                    };
                    text.0 = format!(
                        "Type: {:?}{}\nPosition: ({}, {}, {})\nProperty: {}\nSymmetry: {}",
                        body.kind, grp_str, body.anchor.x, body.anchor.y, body.anchor.z, fixed_str, sym_str
                    );
                } else {
                    text.0 = "Selected block not found.".into();
                }
            } else {
                let mut moveable_count = 0;
                let mut stationary_count = 0;
                let mut groups = std::collections::HashSet::new();
                for &id in &editor.selected_body_ids {
                    if let Some(body) = game.engine.world.body(id) {
                        if body.is_pushable() {
                            moveable_count += 1;
                        } else {
                            stationary_count += 1;
                        }
                        if let Some(gid) = body.combined_group {
                            groups.insert(gid);
                        }
                    }
                }
                let can_combine = stationary_count == 0;
                text.0 = format!(
                    "Selected: {} blocks\nMoveable: {} | Stationary: {}\nCombined Groups: {}\nCan Combine: {}",
                    selected_count,
                    moveable_count,
                    stationary_count,
                    groups.len(),
                    if can_combine { "YES" } else { "NO (contains stationary)" }
                );
            }
        }
    }

    // 2. Update button backgrounds
    let (can_moveable, can_fixed) = editor.selected_kind.map(|k| editor.allowed_fixed_state(k)).unwrap_or((false, false));
    let all_selected_moveable = selected_count >= 2
        && editor.selected_body_ids.iter().all(|&id| {
            game.engine.world.body(id).map(|b| b.is_pushable()).unwrap_or(false)
        });
    let has_any_combined = editor.selected_body_ids.iter().any(|&id| {
        game.engine.world.body(id).and_then(|b| b.combined_group).is_some()
    });

    for (mut bg, palette_opt, prop_opt, z_mode_opt, combine_opt, uncombine_opt) in &mut button_query {
        if let Some(palette_btn) = palette_opt {
            bg.0 = if Some(palette_btn.0) == editor.selected_kind {
                BTN_ACTIVE
            } else {
                BTN_NORMAL
            };
        } else if let Some(prop_btn) = prop_opt {
            if editor.selected_kind.is_some() {
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
            } else {
                bg.0 = BTN_NORMAL;
            }
        } else if let Some(z_btn) = z_mode_opt {
            bg.0 = if z_btn.0 == editor.z_mode {
                BTN_ACTIVE
            } else {
                BTN_NORMAL
            };
        } else if combine_opt.is_some() {
            bg.0 = if all_selected_moveable {
                BTN_SUCCESS
            } else {
                BTN_DISABLED
            };
        } else if uncombine_opt.is_some() {
            bg.0 = if has_any_combined {
                BTN_NORMAL
            } else {
                BTN_DISABLED
            };
        }
    }
}

pub fn update_editor_status_and_modal_ui_system(
    app_mode: Res<State<AppMode>>,
    editor: Res<EditorState>,
    game: Res<crate::GameState>,
    mut text_query: Query<(
        &mut Text,
        Option<&mut TextColor>,
        Option<&SolverStatusBadge>,
        Option<&ToastNotificationText>,
        Option<&ValidationErrorText>,
        Option<&FloorplanWidthLabel>,
        Option<&FloorplanHeightLabel>,
        Option<&FloorplanZLabel>,
        Option<&FloorplanLockToggleText>,
    )>,
    mut action_btns_query: Query<(&ActionButton, &mut BackgroundColor)>,
    mut modal_query: Query<&mut Visibility, (With<FloorplanModal>, Without<EditorRootUi>, Without<ValidationErrorBanner>)>,
    mut banner_query: Query<&mut Visibility, (With<ValidationErrorBanner>, Without<EditorRootUi>, Without<FloorplanModal>)>,
) {
    if *app_mode.get() != AppMode::Editor {
        return;
    }

    let has_val_err = game.engine.validation_error.is_some();
    let current_hash = crate::level::compute_level_hash(&game.engine.world);
    let cached_valid = editor
        .cached_solution
        .as_ref()
        .map(|(h, _)| *h == current_hash)
        .unwrap_or(false);

    // 1. Update Floorplan Modal visibility
    for mut vis in &mut modal_query {
        *vis = if editor.floorplan_open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // 2. Update Validation Error Banner visibility
    for mut vis in &mut banner_query {
        *vis = if has_val_err {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // 3. Update Text elements
    for (mut text, mut color_opt, solver_opt, toast_opt, banner_opt, fp_w_opt, fp_h_opt, fp_z_opt, fp_lock_opt) in &mut text_query {
        if solver_opt.is_some() {
            text.0 = format!("Solver: {}", editor.solver_status);
            if let Some(color) = &mut color_opt {
                if editor.solver_status.starts_with('✓') {
                    color.0 = Color::srgb(0.3, 1.0, 0.6);
                } else if editor.solver_status.starts_with('✗') || editor.solver_status.starts_with("Invalid") {
                    color.0 = Color::srgb(1.0, 0.4, 0.4);
                } else if editor.solver_status.starts_with("Solving") {
                    color.0 = Color::srgb(1.0, 0.8, 0.2);
                } else {
                    color.0 = TEXT_MUTED;
                }
            }
        } else if toast_opt.is_some() {
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
        } else if banner_opt.is_some() {
            if let Some(err_msg) = &game.engine.validation_error {
                text.0 = format!("⚠ {}", err_msg);
            }
        } else if fp_w_opt.is_some() {
            text.0 = format!("Width: {}", editor.floorplan_width);
        } else if fp_h_opt.is_some() {
            text.0 = format!("Height: {}", editor.floorplan_height);
        } else if fp_z_opt.is_some() {
            text.0 = format!("Height: {}", editor.floorplan_height);
            text.0 = format!("Floor Z: {}", editor.floorplan_z);
        } else if fp_h_opt.is_some() {
            text.0 = format!("Height: {}", editor.floorplan_height);
        } else if fp_lock_opt.is_some() {
            let is_locked = editor.is_layer_locked(editor.floorplan_z);
            text.0 = if is_locked {
                format!("Unlock Floor Layer (Z={})", editor.floorplan_z)
            } else {
                format!("Lock Floor Layer (Z={})", editor.floorplan_z)
            };
        }
    }

    // 4. Highlight "Test with Solution" / disable playtest buttons if level is invalid
    for (action_btn, mut bg) in &mut action_btns_query {
        match action_btn.0 {
            EditorAction::TestPlay => {
                bg.0 = if has_val_err { BTN_DISABLED } else { BTN_NORMAL };
            }
            EditorAction::TestWithSolution => {
                bg.0 = if has_val_err || !cached_valid {
                    BTN_DISABLED
                } else {
                    BTN_SUCCESS
                };
            }
            EditorAction::ToggleFloorplanModal => {
                bg.0 = if editor.floorplan_open { BTN_ACTIVE } else { BTN_NORMAL };
            }
            _ => {}
        }
    }
}
