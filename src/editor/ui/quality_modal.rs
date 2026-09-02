use bevy::prelude::*;
use crate::editor::{AppMode, EditorState};
use crate::editor::ui::{
    theme, QualityModal, QualityModalCloseBtn, QualityModalContentContainer,
    QualityModalItem, QualitySelectRedundantBtn,
};

/// Spawn the Floating Quality Analysis Modal into the root UI hierarchy.
pub fn spawn_quality_modal(root: &mut ChildSpawnerCommands) {
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
            TextColor(theme::TEXT_GOLD),
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
                    BackgroundColor(theme::BTN_NORMAL),
                ))
                .with_children(|b| {
                    b.spawn((Text::new("Close [Esc]"), TextFont::from_font_size(12.0), TextColor(theme::TEXT_PRIMARY)));
                });
            });
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
                    TextColor(theme::TEXT_PRIMARY),
                ));
            }
        });

        // Card 2: Epiphany & Deception Score
        if profile.is_solvable {
            let (tier_text, tier_color) = if profile.epiphany_score > 5.0 {
                ("★★★★★ High Heuristic Deception (Aha! Moment)", theme::TEXT_GOLD)
            } else if profile.epiphany_score > 1.5 {
                ("★★★☆☆ Moderate Insight & Detour", Color::srgb(0.4, 0.9, 0.6))
            } else {
                ("★☆☆☆☆ Direct / Greedy Linear Path", theme::TEXT_MUTED)
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
                    TextColor(theme::TEXT_GOLD),
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
                    TextColor(if profile.redundant_bodies.is_empty() { theme::TEXT_ACCENT } else { theme::TEXT_WARNING }),
                ));
                if profile.redundant_bodies.is_empty() {
                    card.spawn((
                        Text::new("✓ 100% Essential: Zero redundant / useless blocks detected on grid."),
                        TextFont::from_font_size(12.0),
                        TextColor(theme::TEXT_SUCCESS),
                    ));
                } else {
                    card.spawn((
                        Text::new(format!("⚠️ Redundant Pieces Found: {} block(s) can be removed without breaking the puzzle solution.", profile.redundant_bodies.len())),
                        TextFont::from_font_size(12.0),
                        TextColor(theme::TEXT_WARNING),
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
                        TextColor(theme::TEXT_PRIMARY),
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
