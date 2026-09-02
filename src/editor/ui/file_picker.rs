use bevy::prelude::*;
use crate::editor::{AppMode, EditorState};
use crate::editor::ui::{
    theme, FilePickerCancelBtn, FilePickerCurrentDirText, FilePickerDirBtn,
    FilePickerFileBtn, FilePickerItem, FilePickerListContainer, FilePickerModal,
    FilePickerScrollBarThumb, FilePickerScrollBarTrack, FilePickerScrollDownBtn,
    FilePickerScrollPageDownBtn, FilePickerScrollPageUpBtn, FilePickerScrollStatusText,
    FilePickerScrollUpBtn, FilePickerUpBtn, SolverStatusBadge, ToastNotificationText,
};

#[derive(Clone, Debug)]
pub enum PickerEntryItem {
    Up(String),
    Directory(String, String),
    File(String, String),
}

/// Spawn the Floating File Picker Modal into the root UI hierarchy.
pub fn spawn_file_picker_modal(root: &mut ChildSpawnerCommands) {
    root.spawn((
        FilePickerModal,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(26.0),
            top: Val::Percent(10.0),
            width: Val::Px(580.0),
            height: Val::Px(600.0),
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
            TextColor(theme::TEXT_GOLD),
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
                    TextColor(theme::TEXT_CYAN),
                ));
            });

        // Main content: Horizontal row containing List Container + Vertical Scroll Bar
        modal
            .spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Px(410.0),
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
                        row_gap: Val::Px(2.0),
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
                            width: Val::Px(32.0),
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
                                BackgroundColor(theme::BTN_NORMAL),
                            ))
                            .with_children(|b| {
                                b.spawn((Text::new("▲"), TextFont::from_font_size(11.0), TextColor(theme::TEXT_PRIMARY)));
                            });

                        // Scroll Track + Thumb (Interactive Track Button)
                        scrollbar
                            .spawn((
                                FilePickerScrollBarTrack,
                                Button,
                                Node {
                                    width: Val::Px(16.0),
                                    flex_grow: 1.0,
                                    margin: UiRect::axes(Val::ZERO, Val::Px(3.0)),
                                    position_type: PositionType::Relative,
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BorderColor::all(Color::srgba(0.25, 0.35, 0.55, 0.8)),
                                BackgroundColor(Color::srgba(0.04, 0.05, 0.07, 0.95)),
                            ))
                            .with_children(|track| {
                                track.spawn((
                                    FilePickerScrollBarThumb,
                                    Button,
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: Val::ZERO,
                                        top: Val::Percent(0.0),
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(25.0),
                                        border: UiRect::all(Val::Px(1.0)),
                                        ..default()
                                    },
                                    BorderColor::all(Color::srgba(0.5, 0.8, 1.0, 0.9)),
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
                                BackgroundColor(theme::BTN_NORMAL),
                            ))
                            .with_children(|b| {
                                b.spawn((Text::new("▼"), TextFont::from_font_size(11.0), TextColor(theme::TEXT_PRIMARY)));
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
                    column_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|btns| {
                    btns.spawn((
                        FilePickerScrollPageUpBtn,
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(theme::BTN_NORMAL),
                    ))
                    .with_children(|b| {
                        b.spawn((Text::new("<< Page Up"), TextFont::from_font_size(10.0), TextColor(theme::TEXT_PRIMARY)));
                    });

                    btns.spawn((
                        FilePickerScrollPageDownBtn,
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(theme::BTN_NORMAL),
                    ))
                    .with_children(|b| {
                        b.spawn((Text::new("Page Down >>"), TextFont::from_font_size(10.0), TextColor(theme::TEXT_PRIMARY)));
                    });
                });
            });

        // Bottom action buttons: Cancel [Esc]
        modal
            .spawn(Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::FlexEnd,
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    FilePickerCancelBtn,
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(16.0), Val::Px(6.0)),
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

/// Dynamically updates the File Picker UI: scanning, scrolling, and list population.
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
    let visible_count = 14;
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
                TextColor(theme::TEXT_MUTED),
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
                            height: Val::Px(24.0),
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
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
                            height: Val::Px(24.0),
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
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
                            height: Val::Px(24.0),
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(theme::BTN_NORMAL),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new(format!("[LVL] {}", name)),
                            TextFont::from_font_size(12.0),
                            TextColor(theme::TEXT_PRIMARY),
                        ));
                    });
                }
            }
        }
    });
}
