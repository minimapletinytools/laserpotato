use bevy::prelude::*;
use crate::editor::ui::{
    theme, CombineButton, CopyAndPlaceButton, DeleteBlockButton, EditorRightSidebar,
    InspectorHeaderTitle, InspectorPanel, InspectorText, PlacementOnlyControl, ReflectXButton,
    ReflectYButton, ResetPlacementOrientationButton, RotateCcwButton, RotateCwButton,
    RotateXNegButton, RotateXPosButton, RotateYNegButton, RotateYPosButton, SelectionOnlyControl,
    ToggleFixedButton, TransformControlsRow, UncombineButton,
};

/// Spawn the Right Inspector Panel into the editor workspace.
pub fn spawn_inspector_panel(workspace: &mut ChildSpawnerCommands) {
    workspace
        .spawn((
            EditorRightSidebar,
            InspectorPanel,
            Node {
                width: Val::Px(240.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
        ))
        .with_children(|inspector| {
            inspector.spawn((
                InspectorHeaderTitle,
                Text::new("BLOCK INSPECTOR"),
                TextFont::from_font_size(13.0),
                TextColor(theme::TEXT_PRIMARY),
            ));

            inspector.spawn((
                InspectorText,
                Text::new("Click a block in the 3D grid to select and modify its properties."),
                TextFont::from_font_size(11.0),
                TextColor(theme::TEXT_MUTED),
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
                    BackgroundColor(theme::BTN_NORMAL),
                    BorderColor::all(Color::srgba(0.35, 0.45, 0.60, 0.6)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Reset Orientation"),
                        TextFont::from_font_size(11.0),
                        TextColor(theme::TEXT_PRIMARY),
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
                        BackgroundColor(theme::BTN_NORMAL),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Pitch +X [T]"),
                            TextFont::from_font_size(11.0),
                            TextColor(theme::TEXT_PRIMARY),
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
                        BackgroundColor(theme::BTN_NORMAL),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Pitch -X"),
                            TextFont::from_font_size(11.0),
                            TextColor(theme::TEXT_PRIMARY),
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
                        BackgroundColor(theme::BTN_NORMAL),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Roll +Y [G]"),
                            TextFont::from_font_size(11.0),
                            TextColor(theme::TEXT_PRIMARY),
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
                        BackgroundColor(theme::BTN_NORMAL),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Roll -Y"),
                            TextFont::from_font_size(11.0),
                            TextColor(theme::TEXT_PRIMARY),
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
                        BackgroundColor(theme::BTN_NORMAL),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Rot CCW"),
                            TextFont::from_font_size(11.0),
                            TextColor(theme::TEXT_PRIMARY),
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
                        BackgroundColor(theme::BTN_NORMAL),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Rot CW [R]"),
                            TextFont::from_font_size(11.0),
                            TextColor(theme::TEXT_PRIMARY),
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
                        BackgroundColor(theme::BTN_NORMAL),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Flip X [X]"),
                            TextFont::from_font_size(11.0),
                            TextColor(theme::TEXT_PRIMARY),
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
                        BackgroundColor(theme::BTN_NORMAL),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Flip Y [Y]"),
                            TextFont::from_font_size(11.0),
                            TextColor(theme::TEXT_PRIMARY),
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
                    BackgroundColor(theme::BTN_NORMAL),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Toggle Fixed / Moveable"),
                        TextFont::from_font_size(11.0),
                        TextColor(theme::TEXT_PRIMARY),
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
                    BackgroundColor(theme::BTN_NORMAL),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Combine [Mega Block]"),
                        TextFont::from_font_size(11.0),
                        TextColor(theme::TEXT_PRIMARY),
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
                    BackgroundColor(theme::BTN_NORMAL),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Uncombine Group"),
                        TextFont::from_font_size(11.0),
                        TextColor(theme::TEXT_PRIMARY),
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
                    BackgroundColor(theme::BTN_DANGER),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Delete Block(s) [Del]"),
                        TextFont::from_font_size(11.0),
                        TextColor(theme::TEXT_PRIMARY),
                    ));
                });
        });
}
