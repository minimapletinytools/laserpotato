//! Interaction systems, folder navigation, and dynamic list rendering for Shipped Level Browser.

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use crate::editor::ui::theme;
use crate::editor::ui::widgets::scrollbar::calculate_thumb_layout;
use crate::play_ui::catalog::{CatalogSortDirection, LevelCatalog};
use crate::play_ui::hud::{PlayHudLevelNameText, PlayHudMovesText, PlayHudRoot};
use crate::play_ui::level_select::*;
use crate::play_ui::overlay::{
    GameOverOverlayRoot, GameOverOverlayText, NextLevelButton, ReplayButton, ResetButton,
    ReturnToMenuButton, UndoButton, VictoryOverlayRoot, VictoryOverlayText,
};
use crate::turn::{PlayerAction, TurnEngine};
use crate::GameState;

#[derive(States, Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum PlayMode {
    #[default]
    LevelSelect,
    Playing,
}

/// Button interaction system for the Shipped Level Browser dialog.
pub fn browser_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            Option<&BrowserUpBtn>,
            Option<&BrowserFolderBtn>,
            Option<&BrowserLevelRowBtn>,
            Option<&BrowserPlayBtn>,
            Option<&BrowserSortHeaderBtn>,
            Option<&BrowserScrollUpBtn>,
            Option<&BrowserScrollDownBtn>,
            Option<&BrowserScrollPageUpBtn>,
            Option<&BrowserScrollPageDownBtn>,
            Option<&BrowserStartGameBtn>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut catalog: ResMut<LevelCatalog>,
    mut game: ResMut<GameState>,
    mut next_mode: ResMut<NextState<PlayMode>>,
) {
    for (
        interaction,
        up_btn,
        folder_btn,
        level_btn,
        play_btn,
        sort_btn,
        scroll_up,
        scroll_down,
        page_up,
        page_down,
        start_game_btn,
    ) in &mut interaction_query
    {
        if *interaction == Interaction::Pressed {
            if up_btn.is_some() {
                catalog.navigate_up();
                if let Some(lvl) = catalog.current_level() {
                    let world = lvl.level_data.to_world();
                    game.engine = TurnEngine::new(world);
                }
            } else if let Some(folder) = folder_btn {
                let name = folder.0.clone();
                catalog.navigate_into(&name);
                if let Some(lvl) = catalog.current_level() {
                    let world = lvl.level_data.to_world();
                    game.engine = TurnEngine::new(world);
                }
            } else if let Some(lvl_row) = level_btn {
                let idx = lvl_row.0;
                if idx < catalog.levels.len() {
                    catalog.current_level_index = idx;
                    catalog.is_dirty = true;
                    let world = catalog.levels[idx].level_data.to_world();
                    game.engine = TurnEngine::new(world);
                }
            } else if let Some(play) = play_btn {
                let idx = play.0;
                if idx < catalog.levels.len() {
                    catalog.current_level_index = idx;
                    let world = catalog.levels[idx].level_data.to_world();
                    game.engine = TurnEngine::new(world);
                    next_mode.set(PlayMode::Playing);
                }
            } else if start_game_btn.is_some() {
                if let Some(lvl) = catalog.current_level() {
                    let world = lvl.level_data.to_world();
                    game.engine = TurnEngine::new(world);
                    next_mode.set(PlayMode::Playing);
                }
            } else if let Some(sort) = sort_btn {
                if catalog.sort_column == sort.0 {
                    catalog.sort_direction = match catalog.sort_direction {
                        CatalogSortDirection::Ascending => CatalogSortDirection::Descending,
                        CatalogSortDirection::Descending => CatalogSortDirection::Ascending,
                    };
                } else {
                    catalog.sort_column = sort.0;
                    catalog.sort_direction = CatalogSortDirection::Ascending;
                }
                catalog.scroll_offset = 0;
                catalog.is_dirty = true;
            } else if scroll_up.is_some() {
                catalog.scroll_offset = catalog.scroll_offset.saturating_sub(1);
                catalog.is_dirty = true;
            } else if scroll_down.is_some() {
                let total = catalog.total_items_count();
                if catalog.scroll_offset + catalog.max_visible_rows < total {
                    catalog.scroll_offset += 1;
                    catalog.is_dirty = true;
                }
            } else if page_up.is_some() {
                catalog.scroll_offset = catalog
                    .scroll_offset
                    .saturating_sub(catalog.max_visible_rows);
                catalog.is_dirty = true;
            } else if page_down.is_some() {
                let total = catalog.total_items_count();
                if catalog.scroll_offset + catalog.max_visible_rows < total {
                    catalog.scroll_offset = (catalog.scroll_offset + catalog.max_visible_rows)
                        .min(total.saturating_sub(catalog.max_visible_rows));
                    catalog.is_dirty = true;
                }
            }
        }
    }
}

/// Button interaction system for HUD and Overlay modals.
pub fn hud_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            Option<&NextLevelButton>,
            Option<&ReplayButton>,
            Option<&ReturnToMenuButton>,
            Option<&UndoButton>,
            Option<&ResetButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut catalog: ResMut<LevelCatalog>,
    mut game: ResMut<GameState>,
    mut next_mode: ResMut<NextState<PlayMode>>,
) {
    for (interaction, next_btn, replay_btn, menu_btn, undo_btn, reset_btn) in
        &mut interaction_query
    {
        if *interaction == Interaction::Pressed {
            if next_btn.is_some() {
                if let Some(next_lvl) = catalog.select_next_in_folder() {
                    let world = next_lvl.level_data.to_world();
                    game.engine = TurnEngine::new(world);
                    next_mode.set(PlayMode::Playing);
                }
            } else if replay_btn.is_some() || reset_btn.is_some() {
                if let Some(lvl) = catalog.current_level() {
                    let world = lvl.level_data.to_world();
                    game.engine = TurnEngine::new(world);
                } else {
                    game.engine.apply(PlayerAction::Reset);
                }
            } else if undo_btn.is_some() {
                game.engine.apply(PlayerAction::Undo);
            } else if menu_btn.is_some() {
                next_mode.set(PlayMode::LevelSelect);
            }
        }
    }
}

pub fn mouse_wheel_scroll_system(
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut catalog: ResMut<LevelCatalog>,
    mode: Res<State<PlayMode>>,
) {
    if *mode.get() != PlayMode::LevelSelect {
        return;
    }

    for event in mouse_wheel.read() {
        let total = catalog.total_items_count();
        if event.y > 0.0 {
            catalog.scroll_offset = catalog.scroll_offset.saturating_sub(1);
            catalog.is_dirty = true;
        } else if event.y < 0.0 {
            if catalog.scroll_offset + catalog.max_visible_rows < total {
                catalog.scroll_offset += 1;
                catalog.is_dirty = true;
            }
        }
    }
}

pub fn update_browser_dynamic_list_system(
    mut commands: Commands,
    mut catalog: ResMut<LevelCatalog>,
    mode: Res<State<PlayMode>>,
    container_query: Query<Entity, With<BrowserListContainer>>,
    item_query: Query<Entity, With<BrowserItem>>,
    mut dir_text_query: Query<&mut Text, (With<BrowserDirText>, Without<BrowserScrollStatusText>)>,
    mut status_text_query: Query<&mut Text, (With<BrowserScrollStatusText>, Without<BrowserDirText>)>,
    mut thumb_query: Query<&mut Node, With<BrowserScrollBarThumb>>,
) {
    if *mode.get() != PlayMode::LevelSelect {
        return;
    }

    if !catalog.is_dirty {
        return;
    }

    catalog.is_dirty = false;

    // 1. Update Directory Breadcrumb Text
    for mut text in &mut dir_text_query {
        let path = if catalog.current_folder.is_empty() {
            "Directory: shipped/".to_string()
        } else {
            format!("Directory: shipped/{}/", catalog.current_folder)
        };
        text.0 = path;
    }

    let folders = catalog.folders_in_current_folder();
    let level_indices = catalog.level_indices_in_current_folder();
    let total_items = folders.len() + level_indices.len();

    // 2. Update Scrollbar Thumb Geometry
    let (thumb_height_pct, thumb_top_pct) = calculate_thumb_layout(
        catalog.max_visible_rows,
        total_items,
        catalog.scroll_offset,
        100.0,
    );

    for mut node in &mut thumb_query {
        node.top = Val::Percent(thumb_top_pct);
        node.height = Val::Percent(thumb_height_pct);
    }

    // 3. Update Status Text
    for mut text in &mut status_text_query {
        if total_items == 0 {
            text.0 = "No levels or folders found".to_string();
        } else {
            let start = catalog.scroll_offset + 1;
            let end = (catalog.scroll_offset + catalog.max_visible_rows).min(total_items);
            text.0 = format!("Showing {}-{} of {} items", start, end, total_items);
        }
    }

    // 4. Rebuild the sliced visible list inside BrowserListContainer
    let Some(container_entity) = container_query.iter().next() else {
        return;
    };

    for item_ent in &item_query {
        commands.entity(item_ent).despawn();
    }

    let start_idx = catalog.scroll_offset;
    let end_idx = (start_idx + catalog.max_visible_rows).min(total_items);

    commands.entity(container_entity).with_children(|parent| {
        for i in start_idx..end_idx {
            if i < folders.len() {
                // Folder Item Row
                let folder_name = &folders[i];
                parent
                    .spawn((
                        BrowserItem,
                        BrowserFolderBtn(folder_name.clone()),
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(28.0),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(theme::BG_CARD),
                        BorderColor::all(theme::BORDER_SUBTLE),
                    ))
                    .with_children(|row| {
                        row.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(6.0),
                            ..default()
                        })
                        .with_children(|left| {
                            left.spawn((
                                Text::new("[DIR] "),
                                TextFont::from_font_size(11.0),
                                TextColor(theme::TEXT_GOLD),
                            ));
                            left.spawn((
                                Text::new(format!("{}/", folder_name)),
                                TextFont::from_font_size(12.0),
                                TextColor(theme::TEXT_CYAN),
                            ));
                        });

                        row.spawn((
                            Text::new("[Open Folder >]"),
                            TextFont::from_font_size(10.0),
                            TextColor(theme::TEXT_MUTED),
                        ));
                    });
            } else {
                // Level Item Row
                let level_pos = i - folders.len();
                let actual_idx = level_indices[level_pos];
                let lvl = &catalog.levels[actual_idx];
                let is_selected = actual_idx == catalog.current_level_index;

                let (bg_col, border_col) = if is_selected {
                    (theme::BG_ROW_SELECTED, theme::BORDER_SELECTED)
                } else {
                    (theme::BG_CARD, theme::BORDER_SUBTLE)
                };

                let diff_color = match lvl.difficulty.as_str() {
                    "Introductory" => theme::TEXT_SUCCESS,
                    "Medium" => theme::TEXT_WARNING,
                    "Advanced" => theme::TEXT_DANGER,
                    _ => theme::TEXT_ACCENT,
                };

                parent
                    .spawn((
                        BrowserItem,
                        BrowserLevelRowBtn(actual_idx),
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(28.0),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(bg_col),
                        BorderColor::all(border_col),
                    ))
                    .with_children(|row| {
                        // Col 1: Name (46%)
                        row.spawn(Node {
                            width: Val::Percent(46.0),
                            overflow: Overflow::clip_x(),
                            ..default()
                        })
                        .with_children(|c| {
                            c.spawn((
                                Text::new(&lvl.title),
                                TextFont::from_font_size(12.0),
                                TextColor(theme::TEXT_PRIMARY),
                            ));
                        });

                        // Col 2: Moves (18%)
                        row.spawn(Node {
                            width: Val::Percent(18.0),
                            ..default()
                        })
                        .with_children(|c| {
                            let moves_str = lvl
                                .macro_moves
                                .map(|m| format!("{}m", m))
                                .unwrap_or_else(|| "-".into());
                            c.spawn((
                                Text::new(moves_str),
                                TextFont::from_font_size(11.0),
                                TextColor(theme::TEXT_ACCENT),
                            ));
                        });

                        // Col 3: Diff (20%)
                        row.spawn(Node {
                            width: Val::Percent(20.0),
                            ..default()
                        })
                        .with_children(|c| {
                            c.spawn((
                                Text::new(&lvl.difficulty),
                                TextFont::from_font_size(11.0),
                                TextColor(diff_color),
                            ));
                        });

                        // Col 4: Blocks (16%)
                        row.spawn(Node {
                            width: Val::Percent(16.0),
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|c| {
                            c.spawn((
                                Text::new(format!("{}", lvl.body_count)),
                                TextFont::from_font_size(11.0),
                                TextColor(theme::TEXT_MUTED),
                            ));

                            // Mini Play button
                            c.spawn((
                                BrowserPlayBtn(actual_idx),
                                Button,
                                Node {
                                    padding: UiRect::axes(Val::Px(6.0), Val::Px(1.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(theme::BTN_PRIMARY),
                                BorderColor::all(theme::BORDER_FOCUS),
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new(">"),
                                    TextFont::from_font_size(11.0),
                                    TextColor(theme::TEXT_PRIMARY),
                                ));
                            });
                        });
                    });
            }
        }
    });
}

pub fn update_browser_detail_card_system(
    catalog: Res<LevelCatalog>,
    mode: Res<State<PlayMode>>,
    mut title_query: Query<&mut Text, (With<BrowserDetailsTitleText>, Without<BrowserDetailsPathText>, Without<BrowserDetailsStatsText>)>,
    mut path_query: Query<&mut Text, (With<BrowserDetailsPathText>, Without<BrowserDetailsTitleText>, Without<BrowserDetailsStatsText>)>,
    mut stats_query: Query<&mut Text, (With<BrowserDetailsStatsText>, Without<BrowserDetailsTitleText>, Without<BrowserDetailsPathText>)>,
) {
    if *mode.get() != PlayMode::LevelSelect {
        return;
    }

    let lvl_opt = catalog.current_level();

    for mut text in &mut title_query {
        text.0 = lvl_opt
            .map(|l| l.title.clone())
            .unwrap_or_else(|| "Select a Level".into());
    }

    for mut text in &mut path_query {
        text.0 = lvl_opt
            .map(|l| format!("File: shipped/{}", l.rel_path))
            .unwrap_or_else(|| "No level selected".into());
    }

    for mut text in &mut stats_query {
        if let Some(lvl) = lvl_opt {
            let moves_str = lvl
                .macro_moves
                .map(|m| format!("{} moves", m))
                .unwrap_or_else(|| "unsolved".into());
            let epi_str = lvl
                .epiphany_score
                .map(|e| format!(" | Epiphany: {:.1}", e))
                .unwrap_or_default();
            text.0 = format!(
                "Moves: {} | Size: {}x{}x{} | Blocks: {}{}",
                moves_str, lvl.width, lvl.height, lvl.depth, lvl.body_count, epi_str
            );
        } else {
            text.0 = String::new();
        }
    }
}

pub fn gameplay_shortcuts_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut catalog: ResMut<LevelCatalog>,
    mut game: ResMut<GameState>,
    mut next_mode: ResMut<NextState<PlayMode>>,
) {
    // Esc -> Return to Level Select
    if keys.just_pressed(KeyCode::Escape) {
        catalog.is_dirty = true;
        next_mode.set(PlayMode::LevelSelect);
    }

    // When level is won: Space / Enter / N advances to next level in folder
    if game.engine.is_won() {
        if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::KeyN) {
            if let Some(next_lvl) = catalog.select_next_in_folder() {
                let world = next_lvl.level_data.to_world();
                game.engine = TurnEngine::new(world);
            }
        }
    }
}

pub fn level_select_shortcuts_system(
    keys: Res<ButtonInput<KeyCode>>,
    catalog: Res<LevelCatalog>,
    mut game: ResMut<GameState>,
    mut next_mode: ResMut<NextState<PlayMode>>,
) {
    // Space or Enter -> Play selected level
    if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) {
        if let Some(lvl) = catalog.current_level() {
            let world = lvl.level_data.to_world();
            game.engine = TurnEngine::new(world);
            next_mode.set(PlayMode::Playing);
        }
    }
}

pub fn update_hud_system(
    mode: Res<State<PlayMode>>,
    catalog: Res<LevelCatalog>,
    game: Res<GameState>,
    mut level_select_query: Query<
        &mut Visibility,
        (
            With<LevelSelectRoot>,
            Without<PlayHudRoot>,
            Without<VictoryOverlayRoot>,
            Without<GameOverOverlayRoot>,
        ),
    >,
    mut hud_query: Query<
        &mut Visibility,
        (
            With<PlayHudRoot>,
            Without<LevelSelectRoot>,
            Without<VictoryOverlayRoot>,
            Without<GameOverOverlayRoot>,
        ),
    >,
    mut victory_query: Query<
        &mut Visibility,
        (
            With<VictoryOverlayRoot>,
            Without<LevelSelectRoot>,
            Without<PlayHudRoot>,
            Without<GameOverOverlayRoot>,
        ),
    >,
    mut game_over_query: Query<
        &mut Visibility,
        (
            With<GameOverOverlayRoot>,
            Without<LevelSelectRoot>,
            Without<PlayHudRoot>,
            Without<VictoryOverlayRoot>,
        ),
    >,
    mut level_name_text: Query<
        &mut Text,
        (
            With<PlayHudLevelNameText>,
            Without<PlayHudMovesText>,
            Without<VictoryOverlayText>,
            Without<GameOverOverlayText>,
        ),
    >,
    mut moves_text: Query<
        &mut Text,
        (
            With<PlayHudMovesText>,
            Without<PlayHudLevelNameText>,
            Without<VictoryOverlayText>,
            Without<GameOverOverlayText>,
        ),
    >,
    mut victory_text: Query<
        &mut Text,
        (
            With<VictoryOverlayText>,
            Without<PlayHudLevelNameText>,
            Without<PlayHudMovesText>,
            Without<GameOverOverlayText>,
        ),
    >,
) {
    let is_select = *mode.get() == PlayMode::LevelSelect;
    let is_playing = *mode.get() == PlayMode::Playing;

    for mut vis in &mut level_select_query {
        *vis = if is_select {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for mut vis in &mut hud_query {
        *vis = if is_playing {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    let is_won = is_playing && game.engine.is_won();
    let is_lost = is_playing && game.engine.is_lost();

    for mut vis in &mut victory_query {
        *vis = if is_won {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for mut vis in &mut game_over_query {
        *vis = if is_lost {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    if is_playing {
        let current_lvl = catalog.current_level();
        let title = current_lvl.map(|l| l.title.as_str()).unwrap_or("Puzzle Level");
        let steps = game.engine.action_history.len();

        for mut text in &mut level_name_text {
            text.0 = title.to_string();
        }

        for mut text in &mut moves_text {
            text.0 = format!("Moves: {}", steps);
        }

        for mut text in &mut victory_text {
            text.0 = format!("Goal pyramid energized in {} move(s)!", steps);
        }
    }
}
