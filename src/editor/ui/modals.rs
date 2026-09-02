use bevy::prelude::*;
use crate::editor::ui::{
    theme, DiscardCancelBtn, DiscardConfirmBtn, DiscardConfirmBtnText,
    FloorplanCloseBtn, FloorplanFillBtn, FloorplanHeightDecBtn, FloorplanHeightIncBtn,
    FloorplanHeightLabel, FloorplanLockToggleBtn, FloorplanLockToggleText, FloorplanModal,
    FloorplanWidthDecBtn, FloorplanWidthIncBtn, FloorplanWidthLabel, FloorplanZDecBtn,
    FloorplanZIncBtn, FloorplanZLabel, SaveAsCancelBtn, SaveAsConfirmBtn, SaveAsFilenameText,
    SaveAsModal, TesterCommentCancelBtn, TesterCommentInputText, TesterCommentModal,
    TesterCommentSaveBtn, TesterDeleteCancelBtn, TesterDeleteConfirmBtn, TesterDeleteConfirmText,
    TesterDeleteModal, TesterPromoteCancelBtn, TesterPromoteCopyBtn, TesterPromoteFilenameText,
    TesterPromoteModal, TesterPromoteMoveBtn, TesterPromoteTitleText, UnsavedConfirmDescText,
    UnsavedConfirmModal, ValidationErrorBanner, ValidationErrorText,
};

/// Spawn the Top Validation Error Banner (Visible when Frame 1 resolution fails).
pub fn spawn_validation_error_banner(root: &mut ChildSpawnerCommands) {
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
}

/// Spawn the Floating Floorplan & Level Size Modal.
pub fn spawn_floorplan_modal(root: &mut ChildSpawnerCommands) {
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
            TextColor(theme::TEXT_GOLD),
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
                    BackgroundColor(theme::BTN_NORMAL),
                ))
                .with_children(|b| {
                    b.spawn((Text::new("-"), TextFont::from_font_size(12.0), TextColor(theme::TEXT_PRIMARY)));
                });

                row.spawn((
                    FloorplanWidthLabel,
                    Text::new("Width: 10"),
                    TextFont::from_font_size(12.0),
                    TextColor(theme::TEXT_PRIMARY),
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
                    BackgroundColor(theme::BTN_NORMAL),
                ))
                .with_children(|b| {
                    b.spawn((Text::new("+"), TextFont::from_font_size(12.0), TextColor(theme::TEXT_PRIMARY)));
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
                    BackgroundColor(theme::BTN_NORMAL),
                ))
                .with_children(|b| {
                    b.spawn((Text::new("-"), TextFont::from_font_size(12.0), TextColor(theme::TEXT_PRIMARY)));
                });

                row.spawn((
                    FloorplanHeightLabel,
                    Text::new("Height: 10"),
                    TextFont::from_font_size(12.0),
                    TextColor(theme::TEXT_PRIMARY),
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
                    BackgroundColor(theme::BTN_NORMAL),
                ))
                .with_children(|b| {
                    b.spawn((Text::new("+"), TextFont::from_font_size(12.0), TextColor(theme::TEXT_PRIMARY)));
                });
            });

        // Target Z Row
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
                    BackgroundColor(theme::BTN_NORMAL),
                ))
                .with_children(|b| {
                    b.spawn((Text::new("-"), TextFont::from_font_size(12.0), TextColor(theme::TEXT_PRIMARY)));
                });

                row.spawn((
                    FloorplanZLabel,
                    Text::new("Target Z: -1"),
                    TextFont::from_font_size(12.0),
                    TextColor(theme::TEXT_PRIMARY),
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
                    BackgroundColor(theme::BTN_NORMAL),
                ))
                .with_children(|b| {
                    b.spawn((Text::new("+"), TextFont::from_font_size(12.0), TextColor(theme::TEXT_PRIMARY)));
                });
            });

        // Fill Action Button
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
                BackgroundColor(theme::BTN_ACTIVE),
            ))
            .with_children(|b| {
                b.spawn((Text::new("Fill Floor Layer"), TextFont::from_font_size(12.0), TextColor(theme::TEXT_PRIMARY)));
            });

        // Lock Layer Toggle Button
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
                BackgroundColor(theme::BTN_NORMAL),
            ))
            .with_children(|b| {
                b.spawn((FloorplanLockToggleText, Text::new("Lock Floor Layer (Z=-1)"), TextFont::from_font_size(12.0), TextColor(theme::TEXT_PRIMARY)));
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
                BackgroundColor(theme::BTN_NORMAL),
            ))
            .with_children(|b| {
                b.spawn((Text::new("Close"), TextFont::from_font_size(11.0), TextColor(theme::TEXT_MUTED)));
            });
    });
}

/// Spawn the Floating Save As Modal.
pub fn spawn_save_as_modal(root: &mut ChildSpawnerCommands) {
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
            TextColor(theme::TEXT_GOLD),
        ));

        modal.spawn((
            Text::new("Enter filename (.json):"),
            TextFont::from_font_size(11.0),
            TextColor(theme::TEXT_MUTED),
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
                    TextColor(theme::TEXT_CYAN),
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
                    BackgroundColor(theme::BTN_NORMAL),
                ))
                .with_children(|b| {
                    b.spawn((Text::new("Cancel [Esc]"), TextFont::from_font_size(12.0), TextColor(theme::TEXT_MUTED)));
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
                    BackgroundColor(theme::BTN_SUCCESS),
                ))
                .with_children(|b| {
                    b.spawn((Text::new("Save [Enter]"), TextFont::from_font_size(12.0), TextColor(Color::WHITE)));
                });
            });
    });
}

/// Spawn the Floating Unsaved Changes Confirmation Modal.
pub fn spawn_unsaved_confirm_modal(root: &mut ChildSpawnerCommands) {
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
            TextColor(theme::TEXT_DANGER),
        ));

        modal.spawn((
            UnsavedConfirmDescText,
            Text::new("You have unsaved changes in the current level.\nAre you sure you want to discard changes and create a new level?"),
            TextFont::from_font_size(12.0),
            TextColor(theme::TEXT_PRIMARY),
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
                    BackgroundColor(theme::BTN_NORMAL),
                ))
                .with_children(|b| {
                    b.spawn((Text::new("Cancel [Esc]"), TextFont::from_font_size(12.0), TextColor(theme::TEXT_MUTED)));
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
}

/// Spawn the Level Tester Comment Modal.
pub fn spawn_tester_comment_modal(root: &mut ChildSpawnerCommands) {
    root.spawn((
        TesterCommentModal,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(30.0),
            top: Val::Percent(25.0),
            width: Val::Px(480.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(16.0)),
            row_gap: Val::Px(10.0),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BorderColor::all(Color::srgba(0.4, 0.7, 1.0, 0.9)),
        BackgroundColor(Color::srgba(0.06, 0.07, 0.10, 0.98)),
    ))
    .with_children(|modal| {
        modal.spawn((
            Text::new("EDIT LEVEL COMMENT / NOTES"),
            TextFont::from_font_size(14.0),
            TextColor(theme::TEXT_GOLD),
        ));

        modal
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(60.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BorderColor::all(Color::srgba(0.3, 0.45, 0.7, 0.6)),
                BackgroundColor(Color::srgba(0.10, 0.12, 0.16, 1.0)),
            ))
            .with_children(|box_node| {
                box_node.spawn((
                    TesterCommentInputText,
                    Text::new(""),
                    TextFont::from_font_size(12.0),
                    TextColor(theme::TEXT_PRIMARY),
                ));
            });

        modal
            .spawn(Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::FlexEnd,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    TesterCommentCancelBtn,
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(theme::BTN_NORMAL),
                ))
                .with_children(|b| {
                    b.spawn((Text::new("Cancel [Esc]"), TextFont::from_font_size(12.0), TextColor(theme::TEXT_PRIMARY)));
                });

                row.spawn((
                    TesterCommentSaveBtn,
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(theme::BTN_SUCCESS),
                ))
                .with_children(|b| {
                    b.spawn((Text::new("Save Comment [Enter]"), TextFont::from_font_size(12.0), TextColor(theme::TEXT_PRIMARY)));
                });
            });
    });
}

/// Spawn the Level Tester Rename + Promote Modal.
pub fn spawn_tester_promote_modal(root: &mut ChildSpawnerCommands) {
    root.spawn((
        TesterPromoteModal,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(30.0),
            top: Val::Percent(20.0),
            width: Val::Px(520.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(16.0)),
            row_gap: Val::Px(10.0),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BorderColor::all(Color::srgba(0.9, 0.75, 0.3, 0.9)),
        BackgroundColor(Color::srgba(0.06, 0.07, 0.10, 0.98)),
    ))
    .with_children(|modal| {
        modal.spawn((
            Text::new("RENAME & PROMOTE PUZZLE TO CURATED FOLDER"),
            TextFont::from_font_size(14.0),
            TextColor(theme::TEXT_GOLD),
        ));

        modal.spawn((
            Text::new("Level Title:"),
            TextFont::from_font_size(11.0),
            TextColor(theme::TEXT_MUTED),
        ));

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
                    TesterPromoteTitleText,
                    Text::new(""),
                    TextFont::from_font_size(12.0),
                    TextColor(theme::TEXT_PRIMARY),
                ));
            });

        modal.spawn((
            Text::new("Filename (in levels/):"),
            TextFont::from_font_size(11.0),
            TextColor(theme::TEXT_MUTED),
        ));

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
                    TesterPromoteFilenameText,
                    Text::new(""),
                    TextFont::from_font_size(12.0),
                    TextColor(theme::TEXT_PRIMARY),
                ));
            });

        modal
            .spawn(Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::FlexEnd,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    TesterPromoteCancelBtn,
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(theme::BTN_NORMAL),
                ))
                .with_children(|b| {
                    b.spawn((Text::new("Cancel [Esc]"), TextFont::from_font_size(11.0), TextColor(theme::TEXT_PRIMARY)));
                });

                row.spawn((
                    TesterPromoteCopyBtn,
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(theme::BTN_ACTIVE),
                ))
                .with_children(|b| {
                    b.spawn((Text::new("Promote & Copy"), TextFont::from_font_size(11.0), TextColor(theme::TEXT_PRIMARY)));
                });

                row.spawn((
                    TesterPromoteMoveBtn,
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(theme::BTN_SUCCESS),
                ))
                .with_children(|b| {
                    b.spawn((Text::new("Promote & Move"), TextFont::from_font_size(11.0), TextColor(theme::TEXT_PRIMARY)));
                });
            });
    });
}

/// Spawn the Level Tester Confirm Delete Modal.
pub fn spawn_tester_delete_modal(root: &mut ChildSpawnerCommands) {
    root.spawn((
        TesterDeleteModal,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(35.0),
            top: Val::Percent(30.0),
            width: Val::Px(420.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(16.0)),
            row_gap: Val::Px(12.0),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BorderColor::all(Color::srgba(0.85, 0.25, 0.25, 0.9)),
        BackgroundColor(Color::srgba(0.06, 0.07, 0.10, 0.98)),
    ))
    .with_children(|modal| {
        modal.spawn((
            Text::new("CONFIRM DELETE"),
            TextFont::from_font_size(14.0),
            TextColor(theme::TEXT_DANGER),
        ));

        modal.spawn((
            TesterDeleteConfirmText,
            Text::new("Are you sure you want to permanently delete this level?"),
            TextFont::from_font_size(12.0),
            TextColor(theme::TEXT_PRIMARY),
        ));

        modal
            .spawn(Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::FlexEnd,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    TesterDeleteCancelBtn,
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(theme::BTN_NORMAL),
                ))
                .with_children(|b| {
                    b.spawn((Text::new("Cancel [Esc]"), TextFont::from_font_size(11.0), TextColor(theme::TEXT_PRIMARY)));
                });

                row.spawn((
                    TesterDeleteConfirmBtn,
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(theme::BTN_DANGER),
                ))
                .with_children(|b| {
                    b.spawn((Text::new("Permanently Delete [Del]"), TextFont::from_font_size(11.0), TextColor(Color::WHITE)));
                });
            });
    });
}
