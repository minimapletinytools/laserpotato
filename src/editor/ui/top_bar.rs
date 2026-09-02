use bevy::prelude::*;
use crate::editor::EditorAction;
use crate::editor::ui::{
    theme, ActionButton, ActionButtonText, EditorModeTopBar, SolverStatusBadge,
    TesterModeTopBar,
};

/// Helper to spawn an action button connected to an `EditorAction`.
pub fn spawn_action_btn(parent: &mut ChildSpawnerCommands, action: EditorAction, label: &str) {
    parent
        .spawn((
            ActionButton(action),
            Button,
            Node {
                padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BorderColor::all(theme::BORDER_SUBTLE),
            BackgroundColor(theme::BTN_NORMAL),
        ))
        .with_children(|b| {
            b.spawn((
                ActionButtonText(action),
                Text::new(label),
                TextFont::from_font_size(12.0),
                TextColor(theme::TEXT_PRIMARY),
            ));
        });
}

/// Spawn the Top Action Bar for Editor Mode.
pub fn spawn_editor_top_bar(root: &mut ChildSpawnerCommands) {
    root.spawn((
        EditorModeTopBar,
        theme::top_bar_node(52.0),
        BackgroundColor(theme::PANEL_BG),
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
                    TextColor(theme::TEXT_GOLD),
                ));

                spawn_action_btn(group, EditorAction::NewLevel, "New");
                spawn_action_btn(group, EditorAction::SaveLevel, "Save");
                spawn_action_btn(group, EditorAction::SaveAsLevel, "Save As");
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
                    TextColor(theme::TEXT_MUTED),
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

        // Right group: Playtest & Replay Mode controls + Tester Switch
        top_bar
            .spawn(Node {
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|group| {
                spawn_action_btn(group, EditorAction::TestPlay, "Test Play");
                spawn_action_btn(group, EditorAction::TestWithSolution, "Test with Solution");
                spawn_action_btn(group, EditorAction::EnterLevelTester, "Tester [F2]");
            });
    });
}

/// Spawn the Top Action Bar for Level Tester Mode.
pub fn spawn_tester_top_bar(root: &mut ChildSpawnerCommands) {
    root.spawn((
        TesterModeTopBar,
        theme::top_bar_node(52.0),
        BackgroundColor(theme::PANEL_BG),
    ))
    .with_children(|top_bar| {
        // Left group: Title & Play actions
        top_bar
            .spawn(Node {
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|group| {
                group.spawn((
                    Text::new("LASER POTATO [TESTER]"),
                    TextFont::from_font_size(15.0),
                    TextColor(theme::TEXT_CYAN),
                ));

                spawn_action_btn(group, EditorAction::TesterOpenInEditor, "Open in Editor");
                spawn_action_btn(group, EditorAction::TesterPlay, "Play [Space]");
                spawn_action_btn(group, EditorAction::TesterPlaySolution, "Play Solution [P]");
            });

        // Center group: Curator actions (Comment, Promote, Delete)
        top_bar
            .spawn(Node {
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|group| {
                spawn_action_btn(group, EditorAction::TesterComment, "Add Comment");
                spawn_action_btn(group, EditorAction::TesterPromote, "Rename + Promote");
                spawn_action_btn(group, EditorAction::TesterDelete, "Delete");
            });

        // Right group: View rotation & Exit
        top_bar
            .spawn(Node {
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|group| {
                spawn_action_btn(group, EditorAction::RotateViewCcw, "Rot L [Q]");
                spawn_action_btn(group, EditorAction::RotateViewCw, "Rot R [E]");
                spawn_action_btn(group, EditorAction::TesterExit, "Exit Tester");
            });
    });
}
