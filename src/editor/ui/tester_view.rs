use bevy::prelude::*;
use crate::editor::{AppMode, EditorState, TesterSortColumn, TesterSortDirection};
use crate::editor::ui::{
    theme, EditorLeftSidebar, EditorModeTopBar, EditorRightSidebar, TesterBulkCountText,
    TesterCommentInputText, TesterCommentModal, TesterDeleteConfirmText, TesterDeleteModal,
    TesterDirText, TesterExpandToggleBtn, TesterHeaderColItem, TesterHeaderRowContainer,
    TesterLeftPanel, TesterListContainer, TesterModeTopBar, TesterPromoteFilenameText,
    TesterPromoteModal, TesterPromoteTitleText, TesterRefreshBtn, TesterRightCard,
    TesterRowCheckBtn, TesterRowItem, TesterRowSelectBtn, TesterScrollBarThumb,
    TesterScrollBarTrack, TesterScrollDownBtn, TesterScrollPageDownBtn, TesterScrollPageUpBtn,
    TesterScrollUpBtn, TesterSelectAllBtn, TesterSortHeaderBtn, TesterStatusText,
    TesterSummaryCommentText, TesterSummaryStatsText, TesterSummaryTitleText,
    TesterTrashSelectedBtn, TesterUpBtn,
};

/// Spawn the Left Browser Panel for Level Tester mode into the root UI hierarchy.
pub fn spawn_tester_left_panel(root: &mut ChildSpawnerCommands) {
    root.spawn((
        TesterLeftPanel,
        Node {
            position_type: PositionType::Absolute,
            left: Val::ZERO,
            top: Val::Px(52.0),
            bottom: Val::Px(28.0),
            width: Val::Px(380.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(8.0)),
            row_gap: Val::Px(6.0),
            border: UiRect::right(Val::Px(1.0)),
            ..default()
        },
        BorderColor::all(Color::srgba(0.25, 0.35, 0.5, 0.6)),
        BackgroundColor(theme::PANEL_BG),
    ))
    .with_children(|panel| {
        // Directory row
        panel
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BorderColor::all(Color::srgba(0.3, 0.45, 0.7, 0.6)),
                BackgroundColor(Color::srgba(0.10, 0.12, 0.16, 1.0)),
            ))
            .with_children(|dir_box| {
                dir_box.spawn((
                    TesterDirText,
                    Text::new("Directory: levels/mined/"),
                    TextFont::from_font_size(11.0),
                    TextColor(theme::TEXT_CYAN),
                ));

                dir_box
                    .spawn(Node {
                        column_gap: Val::Px(4.0),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|btns| {
                        btns.spawn((
                            TesterUpBtn("..".to_string()),
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(theme::BTN_NORMAL),
                        ))
                        .with_children(|b| {
                            b.spawn((Text::new("[UP]"), TextFont::from_font_size(10.0), TextColor(theme::TEXT_PRIMARY)));
                        });

                        btns.spawn((
                            TesterRefreshBtn,
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(theme::BTN_NORMAL),
                        ))
                        .with_children(|b| {
                            b.spawn((Text::new("⟳"), TextFont::from_font_size(10.0), TextColor(theme::TEXT_PRIMARY)));
                        });
                    });
            });

        // Bulk operations toolbar row
        panel
            .spawn(Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
                ..default()
            })
            .with_children(|toolbar| {
                toolbar
                    .spawn(Node {
                        column_gap: Val::Px(6.0),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|left_tb| {
                        left_tb
                            .spawn((
                                TesterSelectAllBtn,
                                Button,
                                Node {
                                    padding: UiRect::axes(Val::Px(5.0), Val::Px(2.0)),
                                    ..default()
                                },
                                BackgroundColor(theme::BTN_NORMAL),
                            ))
                            .with_children(|b| {
                                b.spawn((Text::new("[ ] All"), TextFont::from_font_size(10.0), TextColor(theme::TEXT_PRIMARY)));
                            });

                        left_tb.spawn((
                            TesterBulkCountText,
                            Text::new("0 selected"),
                            TextFont::from_font_size(10.0),
                            TextColor(theme::TEXT_MUTED),
                        ));

                        left_tb
                            .spawn((
                                TesterTrashSelectedBtn,
                                Button,
                                Node {
                                    padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                                    ..default()
                                },
                                BackgroundColor(theme::BTN_DANGER),
                            ))
                            .with_children(|b| {
                                b.spawn((Text::new("🗑 Trash"), TextFont::from_font_size(10.0), TextColor(theme::TEXT_PRIMARY)));
                            });
                    });

                toolbar
                    .spawn((
                        TesterExpandToggleBtn,
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.35, 0.55, 0.9)),
                    ))
                    .with_children(|b| {
                        b.spawn((Text::new("Show More >"), TextFont::from_font_size(10.0), TextColor(theme::TEXT_PRIMARY)));
                    });
            });

        // Column Headers Row Container
        panel.spawn((
            TesterHeaderRowContainer,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(24.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                border: UiRect::bottom(Val::Px(1.0)),
                padding: UiRect::axes(Val::Px(4.0), Val::ZERO),
                ..default()
            },
            BorderColor::all(Color::srgba(0.3, 0.4, 0.6, 0.6)),
            BackgroundColor(Color::srgba(0.12, 0.15, 0.20, 0.95)),
        ));

        // Main Content Row: List Container + Vertical Scroll Bar
        panel
            .spawn(Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                ..default()
            })
            .with_children(|content_row| {
                content_row.spawn((
                    TesterListContainer,
                    Node {
                        flex_grow: 1.0,
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        overflow: Overflow::clip_y(),
                        padding: UiRect::all(Val::Px(2.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor::all(Color::srgba(0.2, 0.25, 0.35, 0.6)),
                    BackgroundColor(Color::srgba(0.04, 0.05, 0.07, 0.8)),
                ));

                // Scrollbar column
                content_row
                    .spawn((
                        Node {
                            width: Val::Px(24.0),
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
                        scrollbar
                            .spawn((
                                TesterScrollUpBtn,
                                Button,
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(22.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(theme::BTN_NORMAL),
                            ))
                            .with_children(|b| {
                                b.spawn((Text::new("▲"), TextFont::from_font_size(10.0), TextColor(theme::TEXT_PRIMARY)));
                            });

                        scrollbar
                            .spawn((
                                TesterScrollBarTrack,
                                Button,
                                Node {
                                    width: Val::Px(12.0),
                                    flex_grow: 1.0,
                                    margin: UiRect::axes(Val::ZERO, Val::Px(2.0)),
                                    position_type: PositionType::Relative,
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BorderColor::all(Color::srgba(0.25, 0.35, 0.55, 0.8)),
                                BackgroundColor(Color::srgba(0.04, 0.05, 0.07, 0.95)),
                            ))
                            .with_children(|track| {
                                track.spawn((
                                    TesterScrollBarThumb,
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

                        scrollbar
                            .spawn((
                                TesterScrollDownBtn,
                                Button,
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(22.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(theme::BTN_NORMAL),
                            ))
                            .with_children(|b| {
                                b.spawn((Text::new("▼"), TextFont::from_font_size(10.0), TextColor(theme::TEXT_PRIMARY)));
                            });
                    });
            });

        // Status & page jump row at bottom of left panel
        panel
            .spawn(Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    TesterStatusText,
                    Text::new("Showing 0 items"),
                    TextFont::from_font_size(10.0),
                    TextColor(Color::srgb(0.6, 0.75, 0.9)),
                ));

                row.spawn(Node {
                    column_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|btns| {
                    btns.spawn((
                        TesterScrollPageUpBtn,
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(5.0), Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(theme::BTN_NORMAL),
                    ))
                    .with_children(|b| {
                        b.spawn((Text::new("<<"), TextFont::from_font_size(10.0), TextColor(theme::TEXT_PRIMARY)));
                    });

                    btns.spawn((
                        TesterScrollPageDownBtn,
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(5.0), Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(theme::BTN_NORMAL),
                    ))
                    .with_children(|b| {
                        b.spawn((Text::new(">>"), TextFont::from_font_size(10.0), TextColor(theme::TEXT_PRIMARY)));
                    });
                });
            });
    });
}

/// Spawn the Right Level Summary Card for Level Tester mode into the root UI hierarchy.
pub fn spawn_tester_right_card(root: &mut ChildSpawnerCommands) {
    root.spawn((
        TesterRightCard,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(16.0),
            bottom: Val::Px(44.0),
            width: Val::Px(340.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(12.0)),
            row_gap: Val::Px(6.0),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BorderColor::all(Color::srgba(0.35, 0.55, 0.85, 0.8)),
        BackgroundColor(Color::srgba(0.06, 0.07, 0.10, 0.95)),
    ))
    .with_children(|card| {
        card.spawn((
            TesterSummaryTitleText,
            Text::new("No level selected"),
            TextFont::from_font_size(13.0),
            TextColor(theme::TEXT_GOLD),
        ));

        card.spawn((
            TesterSummaryStatsText,
            Text::new("Moves: - | Turns: - | Epiphany: -"),
            TextFont::from_font_size(11.0),
            TextColor(theme::TEXT_PRIMARY),
        ));

        card.spawn((
            TesterSummaryCommentText,
            Text::new("Comment: (None)"),
            TextFont::from_font_size(11.0),
            TextColor(theme::TEXT_MUTED),
        ));
    });
}

pub fn sort_tester_entries(
    entries: &mut [crate::level::TesterLevelEntry],
    col: TesterSortColumn,
    dir: TesterSortDirection,
) {
    entries.sort_by(|a, b| {
        match (a.is_directory, b.is_directory) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            (true, true) => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            (false, false) => {
                let ord = match col {
                    TesterSortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    TesterSortColumn::MacroMoves => a.macro_steps.cmp(&b.macro_steps),
                    TesterSortColumn::AtomicTurns => a.atomic_turns.cmp(&b.atomic_turns),
                    TesterSortColumn::Epiphany => {
                        a.epiphany.partial_cmp(&b.epiphany).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    TesterSortColumn::Size => {
                        (a.width * a.height * a.depth).cmp(&(b.width * b.height * b.depth))
                    }
                    TesterSortColumn::Blocks => a.total_blocks.cmp(&b.total_blocks),
                    TesterSortColumn::LoadBearing => {
                        a.load_bearing_pct.partial_cmp(&b.load_bearing_pct).unwrap_or(std::cmp::Ordering::Equal)
                    }
                };
                match dir {
                    TesterSortDirection::Ascending => ord,
                    TesterSortDirection::Descending => ord.reverse(),
                }
            }
        }
    });
}

pub fn update_tester_ui_system(
    app_mode: Res<State<AppMode>>,
    editor: Res<EditorState>,
    mut tester_top_bar_query: Query<&mut Node, (With<TesterModeTopBar>, Without<EditorModeTopBar>, Without<TesterLeftPanel>, Without<TesterRightCard>, Without<EditorLeftSidebar>, Without<EditorRightSidebar>)>,
    mut editor_top_bar_query: Query<&mut Node, (With<EditorModeTopBar>, Without<TesterModeTopBar>, Without<TesterLeftPanel>, Without<TesterRightCard>, Without<EditorLeftSidebar>, Without<EditorRightSidebar>)>,
    mut editor_left_query: Query<&mut Node, (With<EditorLeftSidebar>, Without<TesterModeTopBar>, Without<EditorModeTopBar>, Without<TesterLeftPanel>, Without<TesterRightCard>, Without<EditorRightSidebar>)>,
    mut editor_right_query: Query<&mut Node, (With<EditorRightSidebar>, Without<TesterModeTopBar>, Without<EditorModeTopBar>, Without<TesterLeftPanel>, Without<TesterRightCard>, Without<EditorLeftSidebar>)>,
    mut tester_left_panel_query: Query<&mut Node, (With<TesterLeftPanel>, Without<TesterModeTopBar>, Without<EditorModeTopBar>, Without<TesterRightCard>, Without<EditorLeftSidebar>, Without<EditorRightSidebar>)>,
    mut tester_right_card_query: Query<&mut Node, (With<TesterRightCard>, Without<TesterModeTopBar>, Without<EditorModeTopBar>, Without<TesterLeftPanel>, Without<EditorLeftSidebar>, Without<EditorRightSidebar>)>,
    mut comment_modal_query: Query<&mut Visibility, (With<TesterCommentModal>, Without<TesterPromoteModal>, Without<TesterDeleteModal>)>,
    mut promote_modal_query: Query<&mut Visibility, (With<TesterPromoteModal>, Without<TesterCommentModal>, Without<TesterDeleteModal>)>,
    mut delete_modal_query: Query<&mut Visibility, (With<TesterDeleteModal>, Without<TesterCommentModal>, Without<TesterPromoteModal>)>,
    mut comment_text_query: Query<&mut Text, (With<TesterCommentInputText>, Without<TesterPromoteTitleText>, Without<TesterPromoteFilenameText>, Without<TesterDeleteConfirmText>)>,
    mut promote_title_text_query: Query<&mut Text, (With<TesterPromoteTitleText>, Without<TesterCommentInputText>, Without<TesterPromoteFilenameText>, Without<TesterDeleteConfirmText>)>,
    mut promote_file_text_query: Query<&mut Text, (With<TesterPromoteFilenameText>, Without<TesterCommentInputText>, Without<TesterPromoteTitleText>, Without<TesterDeleteConfirmText>)>,
    mut delete_confirm_text_query: Query<&mut Text, (With<TesterDeleteConfirmText>, Without<TesterCommentInputText>, Without<TesterPromoteTitleText>, Without<TesterPromoteFilenameText>)>,
) {
    let is_tester = *app_mode.get() == AppMode::LevelTester;
    let is_editor = *app_mode.get() == AppMode::Editor;

    for mut node in &mut tester_top_bar_query {
        node.display = if is_tester { Display::Flex } else { Display::None };
    }
    for mut node in &mut editor_top_bar_query {
        node.display = if is_editor { Display::Flex } else { Display::None };
    }
    for mut node in &mut editor_left_query {
        node.display = if is_editor { Display::Flex } else { Display::None };
    }
    for mut node in &mut editor_right_query {
        node.display = if is_editor { Display::Flex } else { Display::None };
    }
    for mut node in &mut tester_left_panel_query {
        node.display = if is_tester { Display::Flex } else { Display::None };
    }
    for mut node in &mut tester_right_card_query {
        node.display = if is_tester { Display::Flex } else { Display::None };
    }

    for mut vis in &mut comment_modal_query {
        *vis = if is_tester && editor.tester_comment_modal_open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for mut vis in &mut promote_modal_query {
        *vis = if is_tester && editor.tester_promote_modal_open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for mut vis in &mut delete_modal_query {
        *vis = if is_tester && editor.tester_delete_modal_open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    if is_tester {
        if editor.tester_comment_modal_open {
            for mut text in &mut comment_text_query {
                text.0 = format!("{}_", editor.tester_comment_buffer);
            }
        }
        if editor.tester_promote_modal_open {
            for mut text in &mut promote_title_text_query {
                text.0 = format!("{}_", editor.tester_promote_title_buffer);
            }
            for mut text in &mut promote_file_text_query {
                text.0 = format!("{}_", editor.tester_promote_filename_buffer);
            }
        }
        if editor.tester_delete_modal_open {
            let count = editor.tester_bulk_selected.len();
            for mut text in &mut delete_confirm_text_query {
                if count > 0 {
                    text.0 = format!("Permanently delete {} selected level file(s) from disk?", count);
                } else if let Some(selected) = &editor.tester_selected_path {
                    let file_name = std::path::Path::new(selected)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(selected);
                    text.0 = format!("Permanently delete '{}' from disk?", file_name);
                } else {
                    text.0 = "No level selected for deletion.".into();
                }
            }
        }
    }
}

pub fn update_tester_table_ui_system(
    mut commands: Commands,
    app_mode: Res<State<AppMode>>,
    mut editor: ResMut<EditorState>,
    text_queries: (
        Query<&mut Text, (With<TesterDirText>, Without<TesterBulkCountText>, Without<TesterStatusText>, Without<TesterSummaryTitleText>, Without<TesterSummaryStatsText>, Without<TesterSummaryCommentText>)>,
        Query<&mut Text, (With<TesterBulkCountText>, Without<TesterDirText>, Without<TesterStatusText>, Without<TesterSummaryTitleText>, Without<TesterSummaryStatsText>, Without<TesterSummaryCommentText>)>,
        Query<&Children, With<TesterSelectAllBtn>>,
        Query<&Children, With<TesterExpandToggleBtn>>,
        Query<&mut Text, (Without<TesterDirText>, Without<TesterBulkCountText>, Without<TesterStatusText>, Without<TesterSummaryTitleText>, Without<TesterSummaryStatsText>, Without<TesterSummaryCommentText>)>,
        Query<&mut Text, (With<TesterStatusText>, Without<TesterDirText>, Without<TesterBulkCountText>, Without<TesterSummaryTitleText>, Without<TesterSummaryStatsText>, Without<TesterSummaryCommentText>)>,
    ),
    layout_queries: (
        Query<&mut Node, With<TesterLeftPanel>>,
        Query<&mut Node, (With<TesterScrollBarThumb>, Without<TesterLeftPanel>)>,
        Query<Entity, With<TesterHeaderRowContainer>>,
        Query<Entity, With<TesterHeaderColItem>>,
        Query<Entity, With<TesterListContainer>>,
        Query<Entity, With<TesterRowItem>>,
    ),
    summary_queries: (
        Query<&mut Text, (With<TesterSummaryTitleText>, Without<TesterSummaryStatsText>, Without<TesterSummaryCommentText>, Without<TesterDirText>, Without<TesterBulkCountText>, Without<TesterStatusText>)>,
        Query<&mut Text, (With<TesterSummaryStatsText>, Without<TesterSummaryTitleText>, Without<TesterSummaryCommentText>, Without<TesterDirText>, Without<TesterBulkCountText>, Without<TesterStatusText>)>,
        Query<&mut Text, (With<TesterSummaryCommentText>, Without<TesterSummaryTitleText>, Without<TesterSummaryStatsText>, Without<TesterDirText>, Without<TesterBulkCountText>, Without<TesterStatusText>)>,
    ),
) {
    if *app_mode.get() != AppMode::LevelTester {
        return;
    }

    if !editor.tester_dirty {
        return;
    }

    editor.tester_dirty = false;

    let (
        mut dir_text_query,
        mut bulk_text_query,
        select_all_btn_query,
        expand_btn_query,
        mut text_child_query,
        mut status_text_query,
    ) = text_queries;
    let (
        mut left_panel_query,
        mut thumb_query,
        header_container_query,
        header_cols_query,
        list_container_query,
        row_item_query,
    ) = layout_queries;
    let (
        mut summary_title_query,
        mut summary_stats_query,
        mut summary_comment_query,
    ) = summary_queries;

    // 1. Scan directory if entries are empty
    if editor.tester_entries.is_empty() {
        let (_parent, items) = crate::level::list_directory_entries(&editor.tester_dir);
        let mut loaded = Vec::new();
        for item in items {
            match item.kind {
                crate::level::FilePickerEntryKind::Directory => {
                    loaded.push(crate::level::extract_tester_dir_entry(&item.path, &item.name));
                }
                crate::level::FilePickerEntryKind::JsonLevelFile => {
                    if let Some(entry_data) = crate::level::extract_tester_level_entry(&item.path, &item.name) {
                        loaded.push(entry_data);
                    }
                }
            }
        }
        editor.tester_entries = loaded;
        if editor.tester_selected_path.is_none() && !editor.tester_entries.is_empty() {
            if let Some(first_file) = editor.tester_entries.iter().find(|e| !e.is_directory) {
                editor.tester_selected_path = Some(first_file.path.clone());
            }
        }
    }

    // 2. Sort entries
    let sort_col = editor.tester_sort_col;
    let sort_dir = editor.tester_sort_dir;
    sort_tester_entries(&mut editor.tester_entries, sort_col, sort_dir);

    // 3. Update Left Panel Width and Expand Button text
    let is_expanded = editor.tester_expanded;
    for mut panel_node in &mut left_panel_query {
        panel_node.width = if is_expanded { Val::Px(780.0) } else { Val::Px(380.0) };
    }

    for children in &expand_btn_query {
        for child in children.iter() {
            if let Ok(mut text) = text_child_query.get_mut(child) {
                text.0 = if is_expanded { "< Show Less".into() } else { "Show More >".into() };
            }
        }
    }

    // 4. Update Directory, Bulk Count, Select All texts
    for mut text in &mut dir_text_query {
        text.0 = format!("Directory: {}/", editor.tester_dir);
    }

    let bulk_count = editor.tester_bulk_selected.len();
    for mut text in &mut bulk_text_query {
        text.0 = format!("{} selected", bulk_count);
    }

    let total_count = editor.tester_entries.len();
    let all_selected = total_count > 0 && bulk_count == total_count;
    for children in &select_all_btn_query {
        for child in children.iter() {
            if let Ok(mut text) = text_child_query.get_mut(child) {
                text.0 = if all_selected { "[✓] All".into() } else { "[ ] All".into() };
            }
        }
    }

    // 5. Update Header Row Container
    for col_entity in &header_cols_query {
        commands.entity(col_entity).despawn();
    }

    if let Some(header_entity) = header_container_query.iter().next() {
        let arrow = match sort_dir {
            TesterSortDirection::Ascending => " ▲",
            TesterSortDirection::Descending => " ▼",
        };

        let col_title = |col: TesterSortColumn, name: &str| -> String {
            if sort_col == col {
                format!("{}{}", name, arrow)
            } else {
                name.to_string()
            }
        };

        commands.entity(header_entity).with_children(|row| {
            // Checkbox column spacer
            row.spawn((
                TesterHeaderColItem,
                Node {
                    width: Val::Px(28.0),
                    ..default()
                },
            ));

            if !is_expanded {
                // Compact headers
                spawn_sort_header_btn(row, TesterSortColumn::Name, &col_title(TesterSortColumn::Name, "Level Name"), Val::Px(250.0));
                spawn_sort_header_btn(row, TesterSortColumn::MacroMoves, &col_title(TesterSortColumn::MacroMoves, "Moves"), Val::Px(60.0));
            } else {
                // Expanded headers
                spawn_sort_header_btn(row, TesterSortColumn::Name, &col_title(TesterSortColumn::Name, "Name"), Val::Px(160.0));
                spawn_sort_header_btn(row, TesterSortColumn::MacroMoves, &col_title(TesterSortColumn::MacroMoves, "Moves"), Val::Px(55.0));
                spawn_sort_header_btn(row, TesterSortColumn::AtomicTurns, &col_title(TesterSortColumn::AtomicTurns, "Turns"), Val::Px(55.0));
                spawn_sort_header_btn(row, TesterSortColumn::Epiphany, &col_title(TesterSortColumn::Epiphany, "Epiphany"), Val::Px(75.0));
                spawn_sort_header_btn(row, TesterSortColumn::Size, &col_title(TesterSortColumn::Size, "Size"), Val::Px(60.0));
                spawn_sort_header_btn(row, TesterSortColumn::Blocks, &col_title(TesterSortColumn::Blocks, "Blocks"), Val::Px(100.0));
                spawn_sort_header_btn(row, TesterSortColumn::LoadBearing, &col_title(TesterSortColumn::LoadBearing, "Load %"), Val::Px(60.0));
                
                row.spawn((
                    TesterHeaderColItem,
                    Node {
                        width: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                )).with_children(|note_col| {
                    note_col.spawn((
                        Text::new("Notes"),
                        TextFont::from_font_size(10.0),
                        TextColor(theme::TEXT_MUTED),
                    ));
                });
            }
        });
    }

    // 6. Update List Rows
    for row_entity in &row_item_query {
        commands.entity(row_entity).despawn();
    }

    let Some(list_container) = list_container_query.iter().next() else {
        return;
    };

    let visible_count = 14;
    let max_offset = total_count.saturating_sub(visible_count);
    editor.tester_scroll_offset = editor.tester_scroll_offset.min(max_offset);
    let start = editor.tester_scroll_offset;
    let end = (start + visible_count).min(total_count);

    let selected_path_clone = editor.tester_selected_path.clone();

    commands.entity(list_container).with_children(|list| {
        for entry in &editor.tester_entries[start..end] {
            let is_row_selected = selected_path_clone.as_deref() == Some(&entry.path);
            let is_checked = editor.tester_bulk_selected.contains(&entry.path);

            list.spawn((
                TesterRowItem,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(24.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(3.0),
                    ..default()
                },
            ))
            .with_children(|row| {
                if entry.is_directory {
                    // Directory row: folder icon badge
                    row.spawn((
                        Node {
                            width: Val::Px(26.0),
                            height: Val::Px(22.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new("📁"),
                            TextFont::from_font_size(11.0),
                            TextColor(theme::TEXT_CYAN),
                        ));
                    });
                } else {
                    // File row: Checkbox button
                    row.spawn((
                        TesterRowCheckBtn(entry.path.clone()),
                        Button,
                        Node {
                            width: Val::Px(26.0),
                            height: Val::Px(22.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(if is_checked { Color::srgba(0.15, 0.45, 0.25, 0.9) } else { theme::BTN_NORMAL }),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new(if is_checked { "✓" } else { " " }),
                            TextFont::from_font_size(10.5),
                            TextColor(if is_checked { Color::srgb(0.4, 0.9, 0.5) } else { theme::TEXT_MUTED }),
                        ));
                    });
                }

                // Row selection button
                row.spawn((
                    TesterRowSelectBtn(entry.path.clone()),
                    Button,
                    Node {
                        flex_grow: 1.0,
                        height: Val::Px(22.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        padding: UiRect::axes(Val::Px(6.0), Val::ZERO),
                        ..default()
                    },
                    BackgroundColor(if is_row_selected {
                        theme::BTN_ACTIVE
                    } else if entry.is_directory {
                        Color::srgba(0.12, 0.16, 0.22, 0.8)
                    } else {
                        theme::BTN_NORMAL
                    }),
                ))
                .with_children(|row_btn| {
                    if entry.is_directory {
                        if !is_expanded {
                            row_btn.spawn((
                                Text::new(&entry.name),
                                TextFont::from_font_size(11.0),
                                TextColor(theme::TEXT_CYAN),
                                Node {
                                    width: Val::Px(250.0),
                                    overflow: Overflow::clip_x(),
                                    ..default()
                                },
                            ));
                            row_btn.spawn((
                                Text::new("[DIR]"),
                                TextFont::from_font_size(10.0),
                                TextColor(theme::TEXT_MUTED),
                                Node {
                                    width: Val::Px(60.0),
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                            ));
                        } else {
                            row_btn.spawn((
                                Text::new(&entry.name),
                                TextFont::from_font_size(11.0),
                                TextColor(theme::TEXT_CYAN),
                                Node {
                                    width: Val::Px(160.0),
                                    overflow: Overflow::clip_x(),
                                    ..default()
                                },
                            ));
                            for _ in 0..3 {
                                row_btn.spawn((
                                    Text::new("--"),
                                    TextFont::from_font_size(10.5),
                                    TextColor(theme::TEXT_MUTED),
                                    Node {
                                        width: Val::Px(55.0),
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                ));
                            }
                            row_btn.spawn((
                                Text::new("--"),
                                TextFont::from_font_size(10.5),
                                TextColor(theme::TEXT_MUTED),
                                Node {
                                    width: Val::Px(75.0),
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                            ));
                            row_btn.spawn((
                                Text::new("Folder"),
                                TextFont::from_font_size(10.5),
                                TextColor(theme::TEXT_MUTED),
                                Node {
                                    width: Val::Px(60.0),
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                            ));
                            row_btn.spawn((
                                Text::new("--"),
                                TextFont::from_font_size(10.5),
                                TextColor(theme::TEXT_MUTED),
                                Node {
                                    width: Val::Px(100.0),
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                            ));
                            row_btn.spawn((
                                Text::new("--"),
                                TextFont::from_font_size(10.5),
                                TextColor(theme::TEXT_MUTED),
                                Node {
                                    width: Val::Px(60.0),
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                            ));
                            row_btn.spawn((
                                Text::new("-"),
                                TextFont::from_font_size(10.0),
                                TextColor(theme::TEXT_MUTED),
                                Node {
                                    width: Val::Px(50.0),
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                            ));
                        }
                    } else if !is_expanded {
                        // Compact file row: Name + Moves
                        row_btn.spawn((
                            Text::new(&entry.name),
                            TextFont::from_font_size(11.0),
                            TextColor(theme::TEXT_PRIMARY),
                            Node {
                                width: Val::Px(250.0),
                                overflow: Overflow::clip_x(),
                                ..default()
                            },
                        ));
                        row_btn.spawn((
                            Text::new(format!("{}m", entry.macro_steps)),
                            TextFont::from_font_size(11.0),
                            TextColor(Color::srgb(0.5, 0.85, 1.0)),
                            Node {
                                width: Val::Px(60.0),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                        ));
                    } else {
                        // Expanded file row: Name, Moves, Turns, Epiphany, Size, Blocks, Load %, Notes
                        row_btn.spawn((
                            Text::new(&entry.name),
                            TextFont::from_font_size(11.0),
                            TextColor(theme::TEXT_PRIMARY),
                            Node {
                                width: Val::Px(160.0),
                                overflow: Overflow::clip_x(),
                                ..default()
                            },
                        ));
                        row_btn.spawn((
                            Text::new(format!("{}", entry.macro_steps)),
                            TextFont::from_font_size(10.5),
                            TextColor(Color::srgb(0.5, 0.85, 1.0)),
                            Node {
                                width: Val::Px(55.0),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                        ));
                        row_btn.spawn((
                            Text::new(format!("{}", entry.atomic_turns)),
                            TextFont::from_font_size(10.5),
                            TextColor(Color::srgb(0.7, 0.8, 0.9)),
                            Node {
                                width: Val::Px(55.0),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                        ));
                        row_btn.spawn((
                            Text::new(format!("{:.1}", entry.epiphany)),
                            TextFont::from_font_size(10.5),
                            TextColor(theme::TEXT_GOLD),
                            Node {
                                width: Val::Px(75.0),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                        ));
                        row_btn.spawn((
                            Text::new(format!("{}x{}", entry.width, entry.height)),
                            TextFont::from_font_size(10.5),
                            TextColor(theme::TEXT_MUTED),
                            Node {
                                width: Val::Px(60.0),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                        ));
                        row_btn.spawn((
                            Text::new(format!("M:{} C:{}", entry.mirrors, entry.crates)),
                            TextFont::from_font_size(10.5),
                            TextColor(Color::srgb(0.6, 0.8, 0.7)),
                            Node {
                                width: Val::Px(100.0),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                        ));
                        row_btn.spawn((
                            Text::new(format!("{:.0}%", entry.load_bearing_pct)),
                            TextFont::from_font_size(10.5),
                            TextColor(if entry.load_bearing_pct >= 99.0 { Color::srgb(0.4, 0.9, 0.5) } else { Color::srgb(1.0, 0.6, 0.3) }),
                            Node {
                                width: Val::Px(60.0),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                        ));
                        row_btn.spawn((
                            Text::new(if entry.has_comment { "[Note]" } else { "-" }),
                            TextFont::from_font_size(10.0),
                            TextColor(if entry.has_comment { theme::TEXT_GOLD } else { theme::TEXT_MUTED }),
                            Node {
                                width: Val::Px(50.0),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                        ));
                    }
                });
            });
        }
    });

    // 7. Update Status Text
    for mut status_text in &mut status_text_query {
        if total_count == 0 {
            status_text.0 = "Directory is empty".into();
        } else {
            status_text.0 = format!("Showing {}–{} of {} items  |  {} selected", start + 1, end, total_count, bulk_count);
        }
    }

    // 8. Update ScrollBar Thumb
    let thumb_height_pct = if total_count <= visible_count {
        100.0
    } else {
        ((visible_count as f32 / total_count as f32) * 100.0).clamp(10.0, 100.0)
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

    // 9. Update Right Summary Info Card
    if let Some(selected_path) = &editor.tester_selected_path {
        if let Some(entry) = editor.tester_entries.iter().find(|e| &e.path == selected_path) {
            for mut title_text in &mut summary_title_query {
                if entry.is_directory {
                    title_text.0 = format!("{} (Folder)", entry.filename);
                } else {
                    title_text.0 = format!("{} ({})", entry.name, entry.filename);
                }
            }
            for mut stats_text in &mut summary_stats_query {
                if entry.is_directory {
                    stats_text.0 = "Folder containing levels and subdirectories.\nClick row to browse folder contents.".into();
                } else {
                    stats_text.0 = format!(
                        "Moves: {} | Turns: {} | Epiphany: {:.1}\nSize: {}x{}x{} | Total Blocks: {} | Essential: {:.0}%",
                        entry.macro_steps, entry.atomic_turns, entry.epiphany, entry.width, entry.height, entry.depth, entry.total_blocks, entry.load_bearing_pct
                    );
                }
            }
            for mut comment_text in &mut summary_comment_query {
                if entry.is_directory {
                    comment_text.0 = "Directory: (Click to open)".into();
                } else if entry.has_comment {
                    comment_text.0 = format!("Notes: {}", entry.description);
                } else {
                    comment_text.0 = "Notes: (No comment added yet)".into();
                }
            }
        }
    }
}

fn spawn_sort_header_btn(
    parent: &mut ChildSpawnerCommands,
    col: TesterSortColumn,
    label: &str,
    width: Val,
) {
    parent
        .spawn((
            TesterHeaderColItem,
            TesterSortHeaderBtn(col),
            Button,
            Node {
                width,
                padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(label),
                TextFont::from_font_size(10.5),
                TextColor(Color::srgb(0.7, 0.85, 1.0)),
            ));
        });
}
