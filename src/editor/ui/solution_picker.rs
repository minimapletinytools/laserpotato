use bevy::prelude::*;
use crate::editor::{AppMode, EditorState};
use crate::editor::ui::{
    theme, SolutionDeleteBtn, SolutionPickerCancelBtn,
    SolutionPickerItem, SolutionPickerListContainer, SolutionPickerModal,
    SolutionPlayBtn, SolutionSpeedDecBtn, SolutionSpeedIncBtn,
    SolutionSpeedLabel, SolutionSpeedPresetBtn, SolverStatusBadge,
    ToastNotificationText,
};

/// Spawn the Floating Solution Picker Modal into the root UI hierarchy.
pub fn spawn_solution_picker_modal(root: &mut ChildSpawnerCommands) {
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
            TextColor(theme::TEXT_PRIMARY),
        ));

        // Speed Slider / Stepper Control Row
        modal
            .spawn((
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

                speed_row
                    .spawn(Node {
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
                            BackgroundColor(theme::BTN_NORMAL),
                        ))
                        .with_children(|b| {
                            b.spawn((Text::new("[-]"), TextFont::from_font_size(12.0), TextColor(theme::TEXT_PRIMARY)));
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
                            BackgroundColor(theme::BTN_NORMAL),
                        ))
                        .with_children(|b| {
                            b.spawn((Text::new("[+]"), TextFont::from_font_size(12.0), TextColor(theme::TEXT_PRIMARY)));
                        });

                        // Preset shortcuts: [0.5x] [1x] [2x] [4x]
                        for &(speed, label) in &[(0.5, "0.5x"), (1.0, "1x"), (2.0, "2x"), (4.0, "4x")] {
                            btns.spawn((
                                SolutionSpeedPresetBtn(speed),
                                Button,
                                Node {
                                    padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    margin: UiRect::left(Val::Px(3.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.12, 0.18, 0.22, 0.8)),
                            ))
                            .with_children(|b| {
                                b.spawn((Text::new(label), TextFont::from_font_size(11.0), TextColor(theme::TEXT_MUTED)));
                            });
                        }
                    });
            });

        // Solution Item List container
        modal.spawn((
            SolutionPickerListContainer,
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                padding: UiRect::all(Val::Px(4.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BorderColor::all(Color::srgba(0.2, 0.35, 0.25, 0.6)),
            BackgroundColor(Color::srgba(0.04, 0.05, 0.04, 0.9)),
        ));

        // Bottom action buttons: Cancel [Esc]
        modal
            .spawn(Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::FlexEnd,
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
                    BackgroundColor(theme::BTN_NORMAL),
                ))
                .with_children(|b| {
                    b.spawn((Text::new("Cancel [Esc]"), TextFont::from_font_size(12.0), TextColor(theme::TEXT_MUTED)));
                });
            });
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
        bg.0 = if is_active { theme::BTN_ACTIVE } else { Color::srgba(0.12, 0.18, 0.22, 0.8) };
        for &child in children {
            if let Ok(mut tc) = text_color_query.get_mut(child) {
                tc.0 = if is_active { theme::TEXT_PRIMARY } else { theme::TEXT_MUTED };
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
                TextColor(theme::TEXT_MUTED),
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
