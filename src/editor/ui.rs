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
pub struct PaletteButton(pub Option<BlockKind>);

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
pub struct InspectorHeaderTitle;

#[derive(Component)]
pub struct InspectorText;

#[derive(Component)]
pub struct CopyAndPlaceButton;

#[derive(Component)]
pub struct ResetPlacementOrientationButton;

#[derive(Component)]
pub struct SelectionOnlyControl;

#[derive(Component)]
pub struct PlacementOnlyControl;

#[derive(Component)]
pub struct TransformControlsRow;

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
pub struct ActionButtonText(pub EditorAction);

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

#[derive(Component)]
pub struct SaveAsModal;

#[derive(Component)]
pub struct SaveAsFilenameText;

#[derive(Component)]
pub struct SaveAsConfirmBtn;

#[derive(Component)]
pub struct SaveAsCancelBtn;

#[derive(Component)]
pub struct UnsavedConfirmModal;

#[derive(Component)]
pub struct UnsavedConfirmDescText;

#[derive(Component)]
pub struct DiscardConfirmBtn;

#[derive(Component)]
pub struct DiscardConfirmBtnText;

#[derive(Component)]
pub struct DiscardCancelBtn;

#[derive(Component)]
pub struct FilePickerModal;

#[derive(Component)]
pub struct FilePickerCurrentDirText;

#[derive(Component)]
pub struct FilePickerListContainer;

#[derive(Component)]
pub struct FilePickerCancelBtn;

#[derive(Component)]
pub struct FilePickerUpBtn(pub String);

#[derive(Component)]
pub struct FilePickerDirBtn(pub String);

#[derive(Component)]
pub struct FilePickerFileBtn(pub String);

#[derive(Component)]
pub struct FilePickerItem;

#[derive(Component)]
pub struct FilePickerScrollUpBtn;

#[derive(Component)]
pub struct FilePickerScrollDownBtn;

#[derive(Component)]
pub struct FilePickerScrollPageUpBtn;

#[derive(Component)]
pub struct FilePickerScrollPageDownBtn;

#[derive(Component)]
pub struct FilePickerScrollBarTrack;

#[derive(Component)]
pub struct FilePickerScrollBarThumb;

#[derive(Component)]
pub struct FilePickerScrollStatusText;

#[derive(Component)]
pub struct SolutionPickerModal;

#[derive(Component)]
pub struct SolutionPickerListContainer;

#[derive(Component)]
pub struct SolutionPickerItem;

#[derive(Component)]
pub struct SolutionPlayBtn(pub usize);

#[derive(Component)]
pub struct SolutionDeleteBtn(pub usize);

#[derive(Component)]
pub struct SolutionPickerCancelBtn;

#[derive(Component)]
pub struct SolutionSpeedLabel;

#[derive(Component)]
pub struct SolutionSpeedDecBtn;

#[derive(Component)]
pub struct SolutionSpeedIncBtn;

#[derive(Component)]
pub struct SolutionSpeedPresetBtn(pub f32);

#[derive(Component)]
pub struct QualityModal;

#[derive(Component)]
pub struct QualityModalContentContainer;

#[derive(Component)]
pub struct QualityModalItem;

#[derive(Component)]
pub struct QualityModalCloseBtn;

#[derive(Component)]
pub struct QualitySelectRedundantBtn;

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
                        spawn_action_btn(group, EditorAction::OpenLevel, "Open");
                        spawn_action_btn(group, EditorAction::Undo, "Undo");
                        spawn_action_btn(group, EditorAction::Redo, "Redo");
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
                        spawn_action_btn(group, EditorAction::AnalyzeQuality, "Analyze");
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
                    Text::new("[!] LEVEL INVALID: spontaneous movement detected at Frame 1. Frame 1 preview disabled."),
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
            // FLOATING SAVE AS MODAL
            // ===============================================================
            root.spawn((
                SaveAsModal,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(35.0),
                    top: Val::Percent(28.0),
                    width: Val::Px(340.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(14.0)),
                    row_gap: Val::Px(10.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BorderColor::all(Color::srgba(0.4, 0.6, 0.9, 0.8)),
                BackgroundColor(Color::srgba(0.06, 0.07, 0.10, 0.98)),
            ))
            .with_children(|modal| {
                modal.spawn((
                    Text::new("SAVE LEVEL AS"),
                    TextFont::from_font_size(14.0),
                    TextColor(Color::srgb(0.9, 0.8, 0.3)),
                ));

                modal.spawn((
                    Text::new("Enter filename (.json):"),
                    TextFont::from_font_size(11.0),
                    TextColor(TEXT_MUTED),
                ));

                // Filename input display box
                modal
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BorderColor::all(Color::srgba(0.3, 0.5, 0.8, 0.6)),
                        BackgroundColor(Color::srgba(0.12, 0.14, 0.18, 1.0)),
                    ))
                    .with_children(|box_node| {
                        box_node.spawn((
                            SaveAsFilenameText,
                            Text::new("puzzle_custom.json"),
                            TextFont::from_font_size(13.0),
                            TextColor(Color::srgb(0.3, 0.85, 1.0)),
                        ));
                    });

                // Buttons row: [Cancel] [Save]
                modal
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::FlexEnd,
                        column_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            SaveAsCancelBtn,
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(BTN_NORMAL),
                        ))
                        .with_children(|b| {
                            b.spawn((Text::new("Cancel [Esc]"), TextFont::from_font_size(12.0), TextColor(TEXT_MUTED)));
                        });

                        row.spawn((
                            SaveAsConfirmBtn,
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(BTN_SUCCESS),
                        ))
                        .with_children(|b| {
                            b.spawn((Text::new("Save [Enter]"), TextFont::from_font_size(12.0), TextColor(Color::WHITE)));
                        });
                    });
            });

            // ===============================================================
            // FLOATING UNSAVED CHANGES CONFIRM MODAL
            // ===============================================================
            root.spawn((
                UnsavedConfirmModal,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(35.0),
                    top: Val::Percent(28.0),
                    width: Val::Px(350.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(14.0)),
                    row_gap: Val::Px(10.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BorderColor::all(Color::srgba(0.9, 0.3, 0.3, 0.8)),
                BackgroundColor(Color::srgba(0.08, 0.07, 0.09, 0.98)),
            ))
            .with_children(|modal| {
                modal.spawn((
                    Text::new("DISCARD UNSAVED CHANGES?"),
                    TextFont::from_font_size(14.0),
                    TextColor(Color::srgb(1.0, 0.4, 0.3)),
                ));

                modal.spawn((
                    UnsavedConfirmDescText,
                    Text::new("You have unsaved changes in the current level.\nAre you sure you want to discard changes and create a new level?"),
                    TextFont::from_font_size(12.0),
                    TextColor(TEXT_PRIMARY),
                ));

                // Buttons row: [Cancel] [Discard & Confirm]
                modal
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::FlexEnd,
                        column_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            DiscardCancelBtn,
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(BTN_NORMAL),
                        ))
                        .with_children(|b| {
                            b.spawn((Text::new("Cancel [Esc]"), TextFont::from_font_size(12.0), TextColor(TEXT_MUTED)));
                        });

                        row.spawn((
                            DiscardConfirmBtn,
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.85, 0.2, 0.2, 0.9)),
                        ))
                        .with_children(|b| {
                            b.spawn((DiscardConfirmBtnText, Text::new("Discard & New"), TextFont::from_font_size(12.0), TextColor(Color::WHITE)));
                        });
                    });
            });

            // ===============================================================
            // FLOATING FILE PICKER MODAL
            // ===============================================================
            root.spawn((
                FilePickerModal,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(28.0),
                    top: Val::Percent(14.0),
                    width: Val::Px(540.0),
                    max_height: Val::Px(580.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(14.0)),
                    row_gap: Val::Px(8.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BorderColor::all(Color::srgba(0.35, 0.55, 0.85, 0.8)),
                BackgroundColor(Color::srgba(0.06, 0.07, 0.10, 0.98)),
            ))
            .with_children(|modal| {
                modal.spawn((
                    Text::new("OPEN LEVEL FILE"),
                    TextFont::from_font_size(14.0),
                    TextColor(Color::srgb(0.9, 0.8, 0.3)),
                ));

                // Current Directory row
                modal
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(5.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BorderColor::all(Color::srgba(0.3, 0.45, 0.7, 0.6)),
                        BackgroundColor(Color::srgba(0.10, 0.12, 0.16, 1.0)),
                    ))
                    .with_children(|box_node| {
                        box_node.spawn((
                            FilePickerCurrentDirText,
                            Text::new("Directory: levels/"),
                            TextFont::from_font_size(12.0),
                            TextColor(Color::srgb(0.4, 0.8, 1.0)),
                        ));
                    });

                // Main content: Horizontal row containing the List Container + Vertical Scroll Bar
                modal
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(340.0),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|content_row| {
                        // Left: Sliced Item List Container
                        content_row.spawn((
                            FilePickerListContainer,
                            Node {
                                flex_grow: 1.0,
                                height: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(3.0),
                                overflow: Overflow::clip_y(),
                                padding: UiRect::all(Val::Px(4.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BorderColor::all(Color::srgba(0.2, 0.25, 0.35, 0.6)),
                            BackgroundColor(Color::srgba(0.04, 0.05, 0.07, 0.8)),
                        ));

                        // Right: Vertical Scroll Bar Column
                        content_row
                            .spawn((
                                Node {
                                    width: Val::Px(28.0),
                                    height: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::SpaceBetween,
                                    padding: UiRect::axes(Val::Px(2.0), Val::Px(2.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BorderColor::all(Color::srgba(0.25, 0.35, 0.5, 0.6)),
                                BackgroundColor(Color::srgba(0.08, 0.10, 0.14, 0.95)),
                            ))
                            .with_children(|scrollbar| {
                                // Scroll Up [▲] Button
                                scrollbar
                                    .spawn((
                                        FilePickerScrollUpBtn,
                                        Button,
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Px(24.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        BackgroundColor(BTN_NORMAL),
                                    ))
                                    .with_children(|b| {
                                        b.spawn((Text::new("▲"), TextFont::from_font_size(11.0), TextColor(TEXT_PRIMARY)));
                                    });

                                // Scroll Track + Thumb
                                scrollbar
                                    .spawn((
                                        FilePickerScrollBarTrack,
                                        Node {
                                            width: Val::Px(10.0),
                                            flex_grow: 1.0,
                                            margin: UiRect::axes(Val::ZERO, Val::Px(3.0)),
                                            position_type: PositionType::Relative,
                                            border: UiRect::all(Val::Px(1.0)),
                                            ..default()
                                        },
                                        BorderColor::all(Color::srgba(0.15, 0.2, 0.3, 0.8)),
                                        BackgroundColor(Color::srgba(0.04, 0.05, 0.07, 0.9)),
                                    ))
                                    .with_children(|track| {
                                        // Scroll Thumb
                                        track.spawn((
                                            FilePickerScrollBarThumb,
                                            Node {
                                                position_type: PositionType::Absolute,
                                                left: Val::ZERO,
                                                top: Val::Percent(0.0),
                                                width: Val::Percent(100.0),
                                                height: Val::Percent(25.0),
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgba(0.35, 0.65, 0.95, 0.85)),
                                        ));
                                    });

                                // Scroll Down [▼] Button
                                scrollbar
                                    .spawn((
                                        FilePickerScrollDownBtn,
                                        Button,
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Px(24.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        BackgroundColor(BTN_NORMAL),
                                    ))
                                    .with_children(|b| {
                                        b.spawn((Text::new("▼"), TextFont::from_font_size(11.0), TextColor(TEXT_PRIMARY)));
                                    });
                            });
                    });

                // Status & Quick Page Jump Row
                modal
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            FilePickerScrollStatusText,
                            Text::new("Showing 0 items"),
                            TextFont::from_font_size(11.0),
                            TextColor(Color::srgb(0.6, 0.75, 0.9)),
                        ));

                        row.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(6.0),
                            ..default()
                        })
                        .with_children(|btns| {
                            // [<< Page]
                            btns.spawn((
                                FilePickerScrollPageUpBtn,
                                Button,
                                Node {
                                    padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(BTN_NORMAL),
                            ))
                            .with_children(|b| {
                                b.spawn((Text::new("<< Page"), TextFont::from_font_size(11.0), TextColor(TEXT_MUTED)));
                            });

                            // [Page >>]
                            btns.spawn((
                                FilePickerScrollPageDownBtn,
                                Button,
                                Node {
                                    padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(BTN_NORMAL),
                            ))
                            .with_children(|b| {
                                b.spawn((Text::new("Page >>"), TextFont::from_font_size(11.0), TextColor(TEXT_MUTED)));
                            });
                        });
                    });

                // Bottom Buttons row: [Hint label] and [Cancel [Esc]]
                modal
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(4.0)),
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            Text::new("Scroll wheel / Arrow keys to scroll"),
                            TextFont::from_font_size(10.0),
                            TextColor(TEXT_MUTED),
                        ));

                        row.spawn((
                            FilePickerCancelBtn,
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(BTN_NORMAL),
                        ))
                        .with_children(|b| {
                            b.spawn((Text::new("Cancel [Esc]"), TextFont::from_font_size(12.0), TextColor(TEXT_MUTED)));
                        });
                    });
            });

            // ===============================================================
            // FLOATING SOLUTION PICKER MODAL
            // ===============================================================
            root.spawn((
                SolutionPickerModal,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(30.0),
                    top: Val::Percent(18.0),
                    width: Val::Px(480.0),
                    max_height: Val::Px(500.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(14.0)),
                    row_gap: Val::Px(10.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BorderColor::all(Color::srgba(0.3, 0.7, 0.45, 0.8)),
                BackgroundColor(Color::srgba(0.06, 0.08, 0.07, 0.98)),
            ))
            .with_children(|modal| {
                modal.spawn((
                    Text::new("CHOOSE SOLUTION TO TEST"),
                    TextFont::from_font_size(14.0),
                    TextColor(Color::srgb(0.3, 1.0, 0.6)),
                ));

                modal.spawn((
                    Text::new("Select a recorded solution for the current level:"),
                    TextFont::from_font_size(12.0),
                    TextColor(TEXT_PRIMARY),
                ));

                // Speed Slider / Stepper Control Row
                modal.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor::all(Color::srgba(0.2, 0.45, 0.3, 0.7)),
                    BackgroundColor(Color::srgba(0.04, 0.07, 0.05, 0.9)),
                ))
                .with_children(|speed_row| {
                    speed_row.spawn((
                        Text::new("Play Speed:"),
                        TextFont::from_font_size(12.0),
                        TextColor(Color::srgb(0.5, 0.9, 0.7)),
                    ));

                    speed_row.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(5.0),
                        ..default()
                    })
                    .with_children(|btns| {
                        // [-] Step slower button
                        btns.spawn((
                            SolutionSpeedDecBtn,
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(BTN_NORMAL),
                        ))
                        .with_children(|b| {
                            b.spawn((Text::new("[-]"), TextFont::from_font_size(12.0), TextColor(TEXT_PRIMARY)));
                        });

                        // Speed display text
                        btns.spawn((
                            SolutionSpeedLabel,
                            Text::new("1.0x (400ms)"),
                            TextFont::from_font_size(12.0),
                            TextColor(Color::srgb(0.95, 0.95, 1.0)),
                        ));

                        // [+] Step faster button
                        btns.spawn((
                            SolutionSpeedIncBtn,
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(BTN_NORMAL),
                        ))
                        .with_children(|b| {
                            b.spawn((Text::new("[+]"), TextFont::from_font_size(12.0), TextColor(TEXT_PRIMARY)));
                        });

                        // Preset buttons: 0.5x, 1x, 2x, 4x
                        for &preset in &[0.5f32, 1.0, 2.0, 4.0] {
                            btns.spawn((
                                SolutionSpeedPresetBtn(preset),
                                Button,
                                Node {
                                    padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(if (preset - 1.0).abs() < 0.01 { BTN_ACTIVE } else { Color::srgba(0.12, 0.18, 0.22, 0.8) }),
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new(format!("{}x", preset)),
                                    TextFont::from_font_size(11.0),
                                    TextColor(if (preset - 1.0).abs() < 0.01 { TEXT_PRIMARY } else { TEXT_MUTED }),
                                ));
                            });
                        }
                    });
                });

                // Scrollable Container for solution items
                modal.spawn((
                    SolutionPickerListContainer,
                    Node {
                        width: Val::Percent(100.0),
                        max_height: Val::Px(300.0),
                        min_height: Val::Px(100.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        overflow: Overflow::clip_y(),
                        padding: UiRect::all(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.03, 0.05, 0.04, 0.8)),
                ));

                // Bottom Buttons row: [Cancel [Esc]]
                modal
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::FlexEnd,
                        column_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            SolutionPickerCancelBtn,
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(BTN_NORMAL),
                        ))
                        .with_children(|b| {
                            b.spawn((Text::new("Cancel [Esc]"), TextFont::from_font_size(12.0), TextColor(TEXT_MUTED)));
                        });
                    });
            });

            // ===============================================================
            // FLOATING PUZZLE QUALITY ANALYSIS MODAL
            // ===============================================================
            root.spawn((
                QualityModal,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(25.0),
                    top: Val::Percent(12.0),
                    width: Val::Px(580.0),
                    max_height: Val::Px(640.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(16.0)),
                    row_gap: Val::Px(12.0),
                    border: UiRect::all(Val::Px(1.5)),
                    ..default()
                },
                BorderColor::all(Color::srgba(0.85, 0.65, 0.2, 0.9)),
                BackgroundColor(Color::srgba(0.06, 0.07, 0.09, 0.98)),
            ))
            .with_children(|modal| {
                modal.spawn((
                    Text::new("PUZZLE QUALITY & INSIGHT PROFILER"),
                    TextFont::from_font_size(15.0),
                    TextColor(Color::srgb(1.0, 0.85, 0.3)),
                ));

                // Scrollable Container for quality analysis content
                modal.spawn((
                    QualityModalContentContainer,
                    Node {
                        width: Val::Percent(100.0),
                        max_height: Val::Px(480.0),
                        min_height: Val::Px(160.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        overflow: Overflow::clip_y(),
                        padding: UiRect::all(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.04, 0.05, 0.07, 0.8)),
                ));

                // Bottom Buttons row: [Select Redundant Blocks] [Close [Esc]]
                modal
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            QualitySelectRedundantBtn,
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.6, 0.3, 0.1, 0.9)),
                        ))
                        .with_children(|b| {
                            b.spawn((Text::new("Select Redundant Blocks"), TextFont::from_font_size(12.0), TextColor(Color::srgb(1.0, 0.9, 0.7))));
                        });

                        row.spawn((
                            QualityModalCloseBtn,
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(BTN_NORMAL),
                        ))
                        .with_children(|b| {
                            b.spawn((Text::new("Close [Esc]"), TextFont::from_font_size(12.0), TextColor(TEXT_PRIMARY)));
                        });
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
                                BackgroundColor(BTN_NORMAL),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("[S] Select"),
                                    TextFont::from_font_size(12.0),
                                    TextColor(TEXT_PRIMARY),
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
                            Text::new("Controls:\n- ⌘Z / ⇧⌘Z: Undo / Redo\n- ⌘S: Save Level\n- Shift+Click: Multi-Select\n- Box Drag (Layer): Multi-Select\n- Esc: Select Mode / Clear\n- L-Click: Place / Select\n- Drag (Stack): Move Block\n- R-Click: Delete Block\n- Tab: Toggle Z Mode\n- PgUp/PgDn: Change Z\n- Q / E: Rotate View 90 deg\n- WASD: Pan Camera"),
                            TextFont::from_font_size(11.0),
                            TextColor(TEXT_MUTED),
                        ));
                    });

                // -----------------------------------------------------------
                // RIGHT FLOATING INSPECTOR PANEL
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
                            InspectorHeaderTitle,
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

                        // Copy and Place Button (Visible when 1+ blocks selected)
                        inspector
                            .spawn((
                                CopyAndPlaceButton,
                                SelectionOnlyControl,
                                Button,
                                Node {
                                    width: Val::Percent(100.0),
                                    padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(1.5)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.12, 0.40, 0.65, 0.95)),
                                BorderColor::all(Color::srgb(0.35, 0.85, 1.0)),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Copy & Place"),
                                    TextFont::from_font_size(12.0),
                                    TextColor(Color::srgb(1.0, 1.0, 1.0)),
                                ));
                            });

                        // Reset Placement Orientation Button (Visible when in placement mode)
                        inspector
                            .spawn((
                                ResetPlacementOrientationButton,
                                PlacementOnlyControl,
                                Button,
                                Node {
                                    width: Val::Percent(100.0),
                                    padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(BTN_NORMAL),
                                BorderColor::all(Color::srgba(0.35, 0.45, 0.60, 0.6)),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Reset Orientation"),
                                    TextFont::from_font_size(11.0),
                                    TextColor(TEXT_PRIMARY),
                                ));
                            });

                        // 3D Pitch (World X-axis tilt)
                        inspector
                            .spawn((
                                TransformControlsRow,
                                Node {
                                    width: Val::Percent(100.0),
                                    column_gap: Val::Px(6.0),
                                    ..default()
                                },
                            ))
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
                            .spawn((
                                TransformControlsRow,
                                Node {
                                    width: Val::Percent(100.0),
                                    column_gap: Val::Px(6.0),
                                    ..default()
                                },
                            ))
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
                            .spawn((
                                TransformControlsRow,
                                Node {
                                    width: Val::Percent(100.0),
                                    column_gap: Val::Px(6.0),
                                    ..default()
                                },
                            ))
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
                            .spawn((
                                TransformControlsRow,
                                Node {
                                    width: Val::Percent(100.0),
                                    column_gap: Val::Px(6.0),
                                    ..default()
                                },
                            ))
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
                                TransformControlsRow,
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
                                SelectionOnlyControl,
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
                                SelectionOnlyControl,
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
                                SelectionOnlyControl,
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
                ActionButtonText(action),
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
        Option<&mut TextColor>,
        Option<&PalettePreviewLabel>,
        Option<&ZLayerLabelText>,
        Option<&InspectorHeaderTitle>,
        Option<&InspectorText>,
    )>,
    mut button_query: Query<(
        &mut BackgroundColor,
        Option<&PaletteButton>,
        Option<&PropertyToggleButton>,
        Option<&ZModeToggleButton>,
        Option<&CombineButton>,
        Option<&UncombineButton>,
        Option<&ToggleFixedButton>,
    )>,
    mut z_btn_query: Query<&mut Node, Or<(With<ZLayerDecButton>, With<ZLayerIncButton>)>>,
    mut selection_ctrl_query: Query<
        &mut Node,
        (
            With<SelectionOnlyControl>,
            Without<PlacementOnlyControl>,
            Without<TransformControlsRow>,
            Without<ZLayerDecButton>,
            Without<ZLayerIncButton>,
        ),
    >,
    mut placement_ctrl_query: Query<
        &mut Node,
        (
            With<PlacementOnlyControl>,
            Without<SelectionOnlyControl>,
            Without<TransformControlsRow>,
            Without<ZLayerDecButton>,
            Without<ZLayerIncButton>,
        ),
    >,
    mut transform_ctrl_query: Query<
        &mut Node,
        (
            With<TransformControlsRow>,
            Without<SelectionOnlyControl>,
            Without<PlacementOnlyControl>,
            Without<ZLayerDecButton>,
            Without<ZLayerIncButton>,
        ),
    >,
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

    let is_stack_mode = editor.z_mode == crate::editor::ZPlacementMode::StackOnTop;
    for mut node in &mut z_btn_query {
        node.display = if is_stack_mode {
            Display::None
        } else {
            Display::Flex
        };
    }

    let is_placement_mode = editor.selected_kind.is_some();
    let selected_count = editor.selected_body_ids.len();
    let has_selection = selected_count > 0;

    // Toggle visibility of inspector sub-sections
    for mut node in &mut selection_ctrl_query {
        node.display = if has_selection {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut node in &mut placement_ctrl_query {
        node.display = if is_placement_mode {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut node in &mut transform_ctrl_query {
        node.display = if is_placement_mode || has_selection {
            Display::Flex
        } else {
            Display::None
        };
    }

    // 1. Update text elements (Preview, Z layer, Inspector Header & Details)
    for (mut text, text_col_opt, preview_opt, z_layer_opt, inspector_header_opt, inspector_opt) in &mut text_query {
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
            if is_stack_mode {
                text.0 = format!("floor z = {}", editor.floorplan_z);
            } else {
                let locked_tag = if editor.is_layer_locked(editor.current_z) { " [LOCKED]" } else { "" };
                text.0 = format!("Layer Z: {}{}", editor.current_z, locked_tag);
            }
        } else if inspector_header_opt.is_some() {
            if is_placement_mode {
                text.0 = "PLACEMENT PROPERTIES".into();
                if let Some(mut col) = text_col_opt {
                    col.0 = Color::srgb(0.35, 0.85, 1.0);
                }
            } else if has_selection {
                if selected_count == 1 {
                    text.0 = "SELECTED BLOCK".into();
                } else {
                    text.0 = format!("SELECTED BLOCKS ({})", selected_count);
                }
                if let Some(mut col) = text_col_opt {
                    col.0 = TEXT_PRIMARY;
                }
            } else {
                text.0 = "BLOCK INSPECTOR".into();
                if let Some(mut col) = text_col_opt {
                    col.0 = TEXT_PRIMARY;
                }
            }
        } else if inspector_opt.is_some() {
            if is_placement_mode {
                let kind = editor.selected_kind.unwrap();
                let (can_moveable, can_fixed) = editor.allowed_fixed_state(kind);
                let is_fixed = if !can_moveable {
                    true
                } else if !can_fixed {
                    false
                } else {
                    editor.is_fixed
                };
                let prop_str = if is_fixed { "Stationary" } else { "Moveable" };
                let facing = editor.placement_orientation.apply(IVec3::Y);
                let sym_str = if editor.placement_orientation.is_reflection() { "Reflected" } else { "Rotated" };
                let z_str = if is_stack_mode {
                    format!("Stack on Top (floor z={})", editor.floorplan_z)
                } else {
                    format!("Fixed Layer Z={}", editor.current_z)
                };
                text.0 = format!(
                    "Tool: {:?}\nProperty: {}\nFacing: ({}, {}, {})\nSymmetry: {}\nZ Mode: {}\n\nClick grid to place.\n[T/G/R/X/Y] Adjust orientation.",
                    kind, prop_str, facing.x, facing.y, facing.z, sym_str, z_str
                );
            } else if !has_selection {
                text.0 = "Select Mode Active [Esc]\nClick or drag to select blocks in the grid.\n\nPick a block from the palette on the left to enter Placement Mode.".into();
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
                    let facing = body.orientation.apply(IVec3::Y);
                    text.0 = format!(
                        "Type: {:?}{}\nPosition: ({}, {}, {})\nProperty: {}\nFacing: ({}, {}, {})\nSymmetry: {}\n\nClick [Copy & Place] to place matching blocks.",
                        body.kind, grp_str, body.anchor.x, body.anchor.y, body.anchor.z, fixed_str, facing.x, facing.y, facing.z, sym_str
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
    let can_toggle_fixed = if is_placement_mode {
        can_moveable && can_fixed
    } else {
        selected_count > 0
            && editor.selected_body_ids.iter().any(|&id| {
                if let Some(body) = game.engine.world.body(id) {
                    let (can_m, can_f) = editor.allowed_fixed_state(body.kind);
                    can_m && can_f
                } else {
                    false
                }
            })
    };

    for (mut bg, palette_opt, prop_opt, z_mode_opt, combine_opt, uncombine_opt, toggle_fixed_opt) in &mut button_query {
        if let Some(palette_btn) = palette_opt {
            bg.0 = if palette_btn.0 == editor.selected_kind {
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
        } else if toggle_fixed_opt.is_some() {
            bg.0 = if can_toggle_fixed {
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
        Option<&ActionButtonText>,
        Option<&SaveAsFilenameText>,
        Option<&UnsavedConfirmDescText>,
        Option<&DiscardConfirmBtnText>,
    )>,
    mut action_btns_query: Query<(&ActionButton, &mut BackgroundColor)>,
    mut modal_query: Query<&mut Visibility, (With<FloorplanModal>, Without<SaveAsModal>, Without<UnsavedConfirmModal>, Without<FilePickerModal>, Without<SolutionPickerModal>, Without<QualityModal>, Without<EditorRootUi>, Without<ValidationErrorBanner>)>,
    mut save_as_modal_query: Query<&mut Visibility, (With<SaveAsModal>, Without<FloorplanModal>, Without<UnsavedConfirmModal>, Without<FilePickerModal>, Without<SolutionPickerModal>, Without<QualityModal>, Without<EditorRootUi>, Without<ValidationErrorBanner>)>,
    mut unsaved_modal_query: Query<&mut Visibility, (With<UnsavedConfirmModal>, Without<FloorplanModal>, Without<SaveAsModal>, Without<FilePickerModal>, Without<SolutionPickerModal>, Without<QualityModal>, Without<EditorRootUi>, Without<ValidationErrorBanner>)>,
    mut banner_query: Query<&mut Visibility, (With<ValidationErrorBanner>, Without<EditorRootUi>, Without<FloorplanModal>, Without<SaveAsModal>, Without<UnsavedConfirmModal>, Without<FilePickerModal>, Without<SolutionPickerModal>, Without<QualityModal>)>,
    mut solution_modal_query: Query<&mut Visibility, (With<SolutionPickerModal>, Without<FloorplanModal>, Without<SaveAsModal>, Without<UnsavedConfirmModal>, Without<FilePickerModal>, Without<QualityModal>, Without<EditorRootUi>, Without<ValidationErrorBanner>)>,
    mut quality_modal_query: Query<&mut Visibility, (With<QualityModal>, Without<FloorplanModal>, Without<SaveAsModal>, Without<UnsavedConfirmModal>, Without<FilePickerModal>, Without<SolutionPickerModal>, Without<EditorRootUi>, Without<ValidationErrorBanner>)>,
) {
    if *app_mode.get() != AppMode::Editor {
        for mut vis in &mut modal_query { *vis = Visibility::Hidden; }
        for mut vis in &mut save_as_modal_query { *vis = Visibility::Hidden; }
        for mut vis in &mut unsaved_modal_query { *vis = Visibility::Hidden; }
        for mut vis in &mut solution_modal_query { *vis = Visibility::Hidden; }
        for mut vis in &mut quality_modal_query { *vis = Visibility::Hidden; }
        for mut vis in &mut banner_query { *vis = Visibility::Hidden; }
        return;
    }

    let has_val_err = game.engine.validation_error.is_some();
    let current_hash = crate::level::compute_level_hash(&game.engine.world);
    let cached_valid = editor
        .cached_solution
        .as_ref()
        .map(|(h, _)| *h == current_hash)
        .unwrap_or(false);

    // 1. Update Modal visibilities
    for mut vis in &mut modal_query {
        *vis = if editor.floorplan_open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for mut vis in &mut save_as_modal_query {
        *vis = if editor.save_as_open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for mut vis in &mut unsaved_modal_query {
        *vis = if editor.unsaved_confirm_open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for mut vis in &mut solution_modal_query {
        *vis = if editor.solution_picker_open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for mut vis in &mut quality_modal_query {
        *vis = if editor.quality_modal_open {
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
    for (mut text, mut color_opt, solver_opt, toast_opt, banner_opt, fp_w_opt, fp_h_opt, fp_z_opt, fp_lock_opt, action_btn_text_opt, save_as_opt, unsaved_desc_opt, discard_btn_text_opt) in &mut text_query {
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
                    "Level: {} | Hash: 0x{:08x} | Blocks: {} | Solutions: {}",
                    editor.current_level_path,
                    current_hash,
                    game.engine.world.bodies().len(),
                    editor.solutions.len()
                );
            }
        } else if banner_opt.is_some() {
            if let Some(err_msg) = &game.engine.validation_error {
                text.0 = format!("[!] {}", err_msg);
            }
        } else if fp_w_opt.is_some() {
            text.0 = format!("Width: {}", editor.floorplan_width);
        } else if fp_h_opt.is_some() {
            text.0 = format!("Height: {}", editor.floorplan_height);
        } else if fp_z_opt.is_some() {
            text.0 = format!("Floor Z: {}", editor.floorplan_z);
        } else if fp_lock_opt.is_some() {
            let is_locked = editor.is_layer_locked(editor.floorplan_z);
            text.0 = if is_locked {
                format!("Unlock Floor Layer (Z={})", editor.floorplan_z)
            } else {
                format!("Lock Floor Layer (Z={})", editor.floorplan_z)
            };
        } else if save_as_opt.is_some() {
            text.0 = format!("{}_", editor.save_as_filename);
        } else if unsaved_desc_opt.is_some() {
            text.0 = match editor.unsaved_action {
                crate::editor::UnsavedAction::NewLevel => "You have unsaved changes in the current level.\nAre you sure you want to discard changes and create a new level?".into(),
                crate::editor::UnsavedAction::OpenLevel => "You have unsaved changes in the current level.\nAre you sure you want to discard changes and open another level?".into(),
            };
        } else if discard_btn_text_opt.is_some() {
            text.0 = match editor.unsaved_action {
                crate::editor::UnsavedAction::NewLevel => "Discard & New".into(),
                crate::editor::UnsavedAction::OpenLevel => "Discard & Open".into(),
            };
        } else if let Some(btn_action) = action_btn_text_opt {
            if btn_action.0 == EditorAction::ToggleFramePreview {
                text.0 = if editor.show_frame1_preview {
                    "Preview: ON".into()
                } else {
                    "Preview: OFF".into()
                };
            }
        }
    }

    // 4. Highlight action button backgrounds
    for (action_btn, mut bg) in &mut action_btns_query {
        match action_btn.0 {
            EditorAction::TestPlay => {
                bg.0 = if has_val_err { BTN_DISABLED } else { BTN_NORMAL };
            }
            EditorAction::TestWithSolution => {
                bg.0 = if has_val_err || (editor.solutions.is_empty() && !cached_valid) {
                    BTN_DISABLED
                } else {
                    BTN_SUCCESS
                };
            }
            EditorAction::Undo => {
                bg.0 = if editor.can_undo() { BTN_NORMAL } else { BTN_DISABLED };
            }
            EditorAction::Redo => {
                bg.0 = if editor.can_redo() { BTN_NORMAL } else { BTN_DISABLED };
            }
            EditorAction::ToggleFloorplanModal => {
                bg.0 = if editor.floorplan_open { BTN_ACTIVE } else { BTN_NORMAL };
            }
            EditorAction::ToggleFramePreview => {
                bg.0 = if editor.show_frame1_preview { BTN_ACTIVE } else { BTN_NORMAL };
            }
            _ => {}
        }
    }
}

enum PickerEntryItem {
    Up(String),
    Directory(String, String),
    File(String, String),
}

/// Dynamically updates the File Picker UI: visibility, current directory label, scrollbar thumb, and directory contents.
pub fn update_file_picker_ui_system(
    mut commands: Commands,
    app_mode: Res<State<AppMode>>,
    mut editor: ResMut<EditorState>,
    mut picker_modal_query: Query<&mut Visibility, With<FilePickerModal>>,
    mut dir_text_query: Query<&mut Text, (With<FilePickerCurrentDirText>, Without<SolverStatusBadge>, Without<ToastNotificationText>, Without<FilePickerScrollStatusText>)>,
    mut status_text_query: Query<&mut Text, (With<FilePickerScrollStatusText>, Without<SolverStatusBadge>, Without<ToastNotificationText>, Without<FilePickerCurrentDirText>)>,
    mut thumb_query: Query<&mut Node, With<FilePickerScrollBarThumb>>,
    container_query: Query<Entity, With<FilePickerListContainer>>,
    item_query: Query<Entity, With<FilePickerItem>>,
) {
    if *app_mode.get() != AppMode::Editor {
        for mut vis in &mut picker_modal_query {
            *vis = Visibility::Hidden;
        }
        return;
    }

    for mut vis in &mut picker_modal_query {
        *vis = if editor.file_picker_open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    if !editor.file_picker_open || !editor.file_picker_dirty {
        return;
    }

    editor.file_picker_dirty = false;

    for mut text in &mut dir_text_query {
        text.0 = format!("Directory: {}/", editor.file_picker_dir);
    }

    let Some(container_entity) = container_query.iter().next() else {
        return;
    };

    for item_entity in &item_query {
        commands.entity(item_entity).despawn();
    }

    let (parent_opt, entries) = crate::level::list_directory_entries(&editor.file_picker_dir);

    // 1. Collect all entries into a unified list
    let mut all_items = Vec::new();
    if let Some(parent) = parent_opt {
        all_items.push(PickerEntryItem::Up(parent));
    }
    for entry in entries {
        match entry.kind {
            crate::level::FilePickerEntryKind::Directory => {
                all_items.push(PickerEntryItem::Directory(entry.name, entry.path));
            }
            crate::level::FilePickerEntryKind::JsonLevelFile => {
                all_items.push(PickerEntryItem::File(entry.name, entry.path));
            }
        }
    }

    let total_count = all_items.len();
    let visible_count = 10;
    let max_offset = total_count.saturating_sub(visible_count);
    editor.file_picker_scroll_offset = editor.file_picker_scroll_offset.min(max_offset);
    let start = editor.file_picker_scroll_offset;
    let end = (start + visible_count).min(total_count);

    // Update status text
    for mut status_text in &mut status_text_query {
        if total_count == 0 {
            status_text.0 = "Directory is empty".into();
        } else {
            status_text.0 = format!("Showing {}–{} of {} items", start + 1, end, total_count);
        }
    }

    // Update ScrollBar Thumb
    let thumb_height_pct = if total_count <= visible_count {
        100.0
    } else {
        ((visible_count as f32 / total_count as f32) * 100.0).clamp(12.0, 100.0)
    };
    let scroll_pct = if max_offset == 0 {
        0.0
    } else {
        (start as f32 / max_offset as f32) * (100.0 - thumb_height_pct)
    };
    for mut thumb_node in &mut thumb_query {
        thumb_node.height = Val::Percent(thumb_height_pct);
        thumb_node.top = Val::Percent(scroll_pct);
    }

    // Spawn visible window
    commands.entity(container_entity).with_children(|list| {
        if total_count == 0 {
            list.spawn((
                FilePickerItem,
                Text::new("(No subfolders or .json level files in this directory)"),
                TextFont::from_font_size(11.0),
                TextColor(TEXT_MUTED),
            ));
            return;
        }

        for item in &all_items[start..end] {
            match item {
                PickerEntryItem::Up(parent) => {
                    list.spawn((
                        FilePickerItem,
                        FilePickerUpBtn(parent.clone()),
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.12, 0.18, 0.26, 0.9)),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new("[UP] .. [Parent Directory]"),
                            TextFont::from_font_size(12.0),
                            TextColor(Color::srgb(0.5, 0.8, 1.0)),
                        ));
                    });
                }
                PickerEntryItem::Directory(name, path) => {
                    list.spawn((
                        FilePickerItem,
                        FilePickerDirBtn(path.clone()),
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.13, 0.16, 0.22, 0.9)),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new(format!("[DIR] {}/", name)),
                            TextFont::from_font_size(12.0),
                            TextColor(Color::srgb(0.4, 0.85, 1.0)),
                        ));
                    });
                }
                PickerEntryItem::File(name, path) => {
                    list.spawn((
                        FilePickerItem,
                        FilePickerFileBtn(path.clone()),
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(BTN_NORMAL),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new(format!("[LVL] {}", name)),
                            TextFont::from_font_size(12.0),
                            TextColor(TEXT_PRIMARY),
                        ));
                    });
                }
            }
        }
    });
}

/// Dynamically updates the Solution Picker UI: visibility and list of available solutions.
pub fn update_solution_picker_ui_system(
    mut commands: Commands,
    app_mode: Res<State<AppMode>>,
    mut editor: ResMut<EditorState>,
    mut picker_modal_query: Query<&mut Visibility, With<SolutionPickerModal>>,
    mut speed_label_query: Query<&mut Text, (With<SolutionSpeedLabel>, Without<SolverStatusBadge>, Without<ToastNotificationText>)>,
    mut preset_btns_query: Query<(&SolutionSpeedPresetBtn, &mut BackgroundColor, &Children)>,
    mut text_color_query: Query<&mut TextColor>,
    container_query: Query<Entity, With<SolutionPickerListContainer>>,
    item_query: Query<Entity, With<SolutionPickerItem>>,
) {
    if *app_mode.get() != AppMode::Editor {
        for mut vis in &mut picker_modal_query {
            *vis = Visibility::Hidden;
        }
        return;
    }

    for mut vis in &mut picker_modal_query {
        *vis = if editor.solution_picker_open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    if !editor.solution_picker_open {
        return;
    }

    // Update speed label and preset buttons styling
    for mut text in &mut speed_label_query {
        let ms = (400.0 / editor.playback_speed.max(0.1)).round() as u32;
        text.0 = format!("{:.1}x ({}ms)", editor.playback_speed, ms);
    }

    for (preset_btn, mut bg, children) in &mut preset_btns_query {
        let is_active = (preset_btn.0 - editor.playback_speed).abs() < 0.05;
        bg.0 = if is_active { BTN_ACTIVE } else { Color::srgba(0.12, 0.18, 0.22, 0.8) };
        for &child in children {
            if let Ok(mut tc) = text_color_query.get_mut(child) {
                tc.0 = if is_active { TEXT_PRIMARY } else { TEXT_MUTED };
            }
        }
    }

    if !editor.solution_picker_dirty {
        return;
    }

    editor.solution_picker_dirty = false;

    let Some(container_entity) = container_query.iter().next() else {
        return;
    };

    for item_entity in &item_query {
        commands.entity(item_entity).despawn();
    }

    commands.entity(container_entity).with_children(|list| {
        if editor.solutions.is_empty() {
            list.spawn((
                SolutionPickerItem,
                Text::new("(No valid recorded solutions for this level)"),
                TextFont::from_font_size(12.0),
                TextColor(TEXT_MUTED),
            ));
        } else {
            for (idx, sol) in editor.solutions.iter().enumerate() {
                list.spawn((
                    SolutionPickerItem,
                    Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(6.0),
                        ..default()
                    },
                ))
                .with_children(|row| {
                    // Play solution button
                    row.spawn((
                        SolutionPlayBtn(idx),
                        Button,
                        Node {
                            flex_grow: 1.0,
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.12, 0.22, 0.32, 0.9)),
                    ))
                    .with_children(|b| {
                        let profile_suffix = if let Some(p) = &sol.profile {
                            format!(" | Epiphany {:.1}, {:.0}% Load-Bearing", p.epiphany_score, p.load_bearing_factor * 100.0)
                        } else {
                            String::new()
                        };
                        b.spawn((
                            Text::new(format!("[PLAY] #{}: {} ({} steps){}", idx + 1, sol.name, sol.actions.len(), profile_suffix)),
                            TextFont::from_font_size(12.0),
                            TextColor(Color::srgb(0.4, 0.9, 0.7)),
                        ));
                    });

                    // Delete solution button
                    row.spawn((
                        SolutionDeleteBtn(idx),
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.4, 0.15, 0.15, 0.9)),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new("[X]"),
                            TextFont::from_font_size(12.0),
                            TextColor(Color::srgb(1.0, 0.4, 0.4)),
                        ));
                    });
                });
            }
        }
    });
}

/// Dynamically updates the Quality Analysis Modal UI with metrics, epiphany score, and milestones.
pub fn update_quality_modal_ui_system(
    mut commands: Commands,
    app_mode: Res<State<AppMode>>,
    mut editor: ResMut<EditorState>,
    mut modal_query: Query<&mut Visibility, With<QualityModal>>,
    mut redundant_btn_query: Query<&mut Visibility, (With<QualitySelectRedundantBtn>, Without<QualityModal>)>,
    container_query: Query<Entity, With<QualityModalContentContainer>>,
    item_query: Query<Entity, With<QualityModalItem>>,
) {
    if *app_mode.get() != AppMode::Editor {
        for mut vis in &mut modal_query {
            *vis = Visibility::Hidden;
        }
        return;
    }

    for mut vis in &mut modal_query {
        *vis = if editor.quality_modal_open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    if !editor.quality_modal_open {
        return;
    }

    if !editor.quality_modal_dirty {
        return;
    }
    editor.quality_modal_dirty = false;

    let Some(container_entity) = container_query.iter().next() else {
        return;
    };

    for item_entity in &item_query {
        commands.entity(item_entity).despawn();
    }

    let Some(profile) = &editor.puzzle_profile else {
        return;
    };

    let has_redundant = !profile.redundant_bodies.is_empty();
    for mut vis in &mut redundant_btn_query {
        *vis = if has_redundant { Visibility::Visible } else { Visibility::Hidden };
    }

    commands.entity(container_entity).with_children(|content| {
        // Card 1: Overview & Solvability
        content.spawn((
            QualityModalItem,
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(4.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BorderColor::all(if profile.is_solvable { Color::srgba(0.2, 0.7, 0.4, 0.6) } else { Color::srgba(0.8, 0.2, 0.2, 0.6) }),
            BackgroundColor(Color::srgba(0.08, 0.12, 0.10, 0.9)),
        )).with_children(|card| {
            card.spawn((
                Text::new(if profile.is_solvable { "SOLVABILITY: ✓ Solvable" } else { "SOLVABILITY: ✗ Unsolvable" }),
                TextFont::from_font_size(13.0),
                TextColor(if profile.is_solvable { Color::srgb(0.3, 1.0, 0.6) } else { Color::srgb(1.0, 0.4, 0.4) }),
            ));
            if profile.is_solvable {
                card.spawn((
                    Text::new(format!("Optimal Solution: {} macro moves ({} atomic turns)", profile.macro_steps, profile.atomic_turns)),
                    TextFont::from_font_size(12.0),
                    TextColor(TEXT_PRIMARY),
                ));
            }
        });

        // Card 2: Epiphany & Deception Score
        if profile.is_solvable {
            let (tier_text, tier_color) = if profile.epiphany_score > 5.0 {
                ("★★★★★ High Heuristic Deception (Aha! Moment)", Color::srgb(1.0, 0.85, 0.2))
            } else if profile.epiphany_score > 1.5 {
                ("★★★☆☆ Moderate Insight & Detour", Color::srgb(0.4, 0.9, 0.6))
            } else {
                ("★☆☆☆☆ Direct / Greedy Linear Path", Color::srgb(0.6, 0.6, 0.7))
            };

            content.spawn((
                QualityModalItem,
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(8.0)),
                    row_gap: Val::Px(4.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BorderColor::all(Color::srgba(0.8, 0.7, 0.2, 0.5)),
                BackgroundColor(Color::srgba(0.12, 0.10, 0.05, 0.9)),
            )).with_children(|card| {
                card.spawn((
                    Text::new(format!("EPIPHANY SCORE: {:.1}", profile.epiphany_score)),
                    TextFont::from_font_size(13.0),
                    TextColor(Color::srgb(1.0, 0.9, 0.4)),
                ));
                card.spawn((
                    Text::new(tier_text),
                    TextFont::from_font_size(12.0),
                    TextColor(tier_color),
                ));
            });

            // Card 3: Load-Bearing Minimality Check
            content.spawn((
                QualityModalItem,
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(8.0)),
                    row_gap: Val::Px(4.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BorderColor::all(if profile.redundant_bodies.is_empty() { Color::srgba(0.2, 0.6, 0.8, 0.5) } else { Color::srgba(0.9, 0.4, 0.1, 0.7) }),
                BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 0.9)),
            )).with_children(|card| {
                card.spawn((
                    Text::new(format!("LOAD-BEARING FACTOR: {:.0}%", profile.load_bearing_factor * 100.0)),
                    TextFont::from_font_size(13.0),
                    TextColor(if profile.redundant_bodies.is_empty() { Color::srgb(0.4, 0.8, 1.0) } else { Color::srgb(1.0, 0.6, 0.2) }),
                ));
                if profile.redundant_bodies.is_empty() {
                    card.spawn((
                        Text::new("✓ 100% Essential: Zero redundant / useless blocks detected on grid."),
                        TextFont::from_font_size(12.0),
                        TextColor(Color::srgb(0.5, 0.9, 0.7)),
                    ));
                } else {
                    card.spawn((
                        Text::new(format!("⚠️ Redundant Pieces Found: {} block(s) can be removed without breaking the puzzle solution.", profile.redundant_bodies.len())),
                        TextFont::from_font_size(12.0),
                        TextColor(Color::srgb(1.0, 0.7, 0.4)),
                    ));
                }
            });

            // Card 4: Milestones / Bottlenecks Sequence
            content.spawn((
                QualityModalItem,
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(8.0)),
                    row_gap: Val::Px(4.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BorderColor::all(Color::srgba(0.4, 0.4, 0.6, 0.4)),
                BackgroundColor(Color::srgba(0.06, 0.06, 0.09, 0.9)),
            )).with_children(|card| {
                card.spawn((
                    Text::new("CONCEPTUAL MILESTONES (BOTTLENECKS):"),
                    TextFont::from_font_size(13.0),
                    TextColor(Color::srgb(0.7, 0.7, 0.9)),
                ));
                for (i, milestone) in profile.milestones.iter().enumerate() {
                    let desc = match milestone {
                        crate::solver::MacroArchetype::OpticalSwitch { source_id, .. } => format!("Optical Topology Switch (Body #{:?})", source_id),
                        crate::solver::MacroArchetype::SpatialPush { kind, from, to, .. } => format!("Spatial Push ({:?} from {:?} to {:?})", kind, from, to),
                        crate::solver::MacroArchetype::SpatialExchange { parking_spot, .. } => format!("Spatial Nook Exchange (Park at {:?})", parking_spot),
                        crate::solver::MacroArchetype::PhaseShift { description } => format!("Phase Shift ({})", description),
                        other => format!("{:?}", other),
                    };
                    card.spawn((
                        Text::new(format!("  {}. {}", i + 1, desc)),
                        TextFont::from_font_size(11.5),
                        TextColor(TEXT_PRIMARY),
                    ));
                }
            });
        }
    });
}

/// Handles interactions with the Quality Analysis Modal buttons.
pub fn quality_modal_interaction_system(
    mut editor: ResMut<EditorState>,
    close_btn_query: Query<&Interaction, (Changed<Interaction>, With<QualityModalCloseBtn>)>,
    redundant_btn_query: Query<&Interaction, (Changed<Interaction>, With<QualitySelectRedundantBtn>)>,
) {
    for interaction in &close_btn_query {
        if *interaction == Interaction::Pressed {
            editor.quality_modal_open = false;
            editor.toast("Closed quality analysis modal.");
        }
    }

    for interaction in &redundant_btn_query {
        if *interaction == Interaction::Pressed {
            if let Some(profile) = &editor.puzzle_profile {
                if !profile.redundant_bodies.is_empty() {
                    let count = profile.redundant_bodies.len();
                    editor.selected_body_ids = profile.redundant_bodies.clone();
                    editor.quality_modal_open = false;
                    editor.toast(format!("Selected {} redundant body(s) on grid for inspection.", count));
                }
            }
        }
    }
}
