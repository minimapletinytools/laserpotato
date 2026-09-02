//! Bevy UI layouts, buttons, and visual panels for the Level Editor.

use bevy::prelude::*;
use crate::block_types::BlockKind;
use super::{AppMode, EditorAction, EditorState};

pub mod file_picker;
pub mod inspector;
pub mod modals;
pub mod palette;
pub mod quality_modal;
pub mod solution_picker;
pub mod tester_view;
pub mod theme;
pub mod top_bar;
pub mod widgets;

pub use file_picker::*;
pub use inspector::*;
pub use modals::*;
pub use palette::*;
pub use quality_modal::*;
pub use solution_picker::*;
pub use tester_view::*;
pub use theme::*;
pub use top_bar::*;
pub use widgets::*;

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
pub struct FilePickerScrollBarTrack;

#[derive(Component)]
pub struct FilePickerScrollBarThumb;

#[derive(Component)]
pub struct FilePickerScrollUpBtn;

#[derive(Component)]
pub struct FilePickerScrollDownBtn;

#[derive(Component)]
pub struct FilePickerScrollStatusText;

#[derive(Component)]
pub struct FilePickerScrollPageUpBtn;

#[derive(Component)]
pub struct FilePickerScrollPageDownBtn;

#[derive(Component)]
pub struct SolutionPickerModal;

#[derive(Component)]
pub struct SolutionPickerListContainer;

#[derive(Component)]
pub struct SolutionPickerCancelBtn;

#[derive(Component)]
pub struct SolutionItem;

#[derive(Component)]
pub struct SolutionPlayBtn(pub usize);

#[derive(Component)]
pub struct SolutionDeleteBtn(pub usize);

#[derive(Component)]
pub struct SolutionPickerItem;

#[derive(Component)]
pub struct SolutionSpeedDecBtn;

#[derive(Component)]
pub struct SolutionSpeedIncBtn;

#[derive(Component)]
pub struct SolutionSpeedLabel;

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

#[derive(Component)]
pub struct EditorModeTopBar;

#[derive(Component)]
pub struct EditorLeftSidebar;

#[derive(Component)]
pub struct EditorRightSidebar;

#[derive(Component)]
pub struct TesterModeTopBar;

#[derive(Component)]
pub struct TesterLeftPanel;

#[derive(Component)]
pub struct TesterDirText;

#[derive(Component)]
pub struct TesterUpBtn(pub String);

#[derive(Component)]
pub struct TesterRefreshBtn;

#[derive(Component)]
pub struct TesterSelectAllBtn;

#[derive(Component)]
pub struct TesterBulkCountText;

#[derive(Component)]
pub struct TesterTrashSelectedBtn;

#[derive(Component)]
pub struct TesterExpandToggleBtn;

#[derive(Component)]
pub struct TesterHeaderRowContainer;

#[derive(Component)]
pub struct TesterHeaderColItem;

#[derive(Component)]
pub struct TesterSortHeaderBtn(pub crate::editor::TesterSortColumn);

#[derive(Component)]
pub struct TesterListContainer;

#[derive(Component)]
pub struct TesterRowItem;

#[derive(Component)]
pub struct TesterRowCheckBtn(pub String);

#[derive(Component)]
pub struct TesterRowSelectBtn(pub String);

#[derive(Component)]
pub struct TesterStatusText;

#[derive(Component)]
pub struct TesterScrollUpBtn;

#[derive(Component)]
pub struct TesterScrollDownBtn;

#[derive(Component)]
pub struct TesterScrollBarTrack;

#[derive(Component)]
pub struct TesterScrollBarThumb;

#[derive(Component)]
pub struct TesterScrollPageUpBtn;

#[derive(Component)]
pub struct TesterScrollPageDownBtn;

#[derive(Component)]
pub struct TesterRightCard;

#[derive(Component)]
pub struct TesterSummaryTitleText;

#[derive(Component)]
pub struct TesterSummaryStatsText;

#[derive(Component)]
pub struct TesterSummaryCommentText;

#[derive(Component)]
pub struct TesterCommentModal;

#[derive(Component)]
pub struct TesterCommentInputText;

#[derive(Component)]
pub struct TesterCommentSaveBtn;

#[derive(Component)]
pub struct TesterCommentCancelBtn;

#[derive(Component)]
pub struct TesterPromoteModal;

#[derive(Component)]
pub struct TesterPromoteTitleText;

#[derive(Component)]
pub struct TesterPromoteFilenameText;

#[derive(Component)]
pub struct TesterPromoteCopyBtn;

#[derive(Component)]
pub struct TesterPromoteMoveBtn;

#[derive(Component)]
pub struct TesterPromoteCancelBtn;

#[derive(Component)]
pub struct TesterDeleteModal;

#[derive(Component)]
pub struct TesterDeleteConfirmText;

#[derive(Component)]
pub struct TesterDeleteConfirmBtn;

#[derive(Component)]
pub struct TesterDeleteCancelBtn;

// ---------------------------------------------------------------------------
// Setup Editor UI
// ---------------------------------------------------------------------------

pub fn setup_editor_ui(mut commands: Commands) {
    commands
        .spawn((
            EditorRootUi,
            theme::root_layout(),
        ))
        .with_children(|root| {
            // Top action bars
            top_bar::spawn_editor_top_bar(root);
            top_bar::spawn_tester_top_bar(root);

            // Validation warning banner
            modals::spawn_validation_error_banner(root);

            // Floating Modals & Dialogs (Editor Mode)
            modals::spawn_floorplan_modal(root);
            modals::spawn_save_as_modal(root);
            modals::spawn_unsaved_confirm_modal(root);
            file_picker::spawn_file_picker_modal(root);
            solution_picker::spawn_solution_picker_modal(root);
            quality_modal::spawn_quality_modal(root);

            // Main middle workspace
            root.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            })
            .with_children(|workspace| {
                palette::spawn_palette_panel(workspace);
                inspector::spawn_inspector_panel(workspace);
            });

            // Level Tester Mode Panels & Modals
            tester_view::spawn_tester_left_panel(root);
            tester_view::spawn_tester_right_card(root);
            modals::spawn_tester_comment_modal(root);
            modals::spawn_tester_promote_modal(root);
            modals::spawn_tester_delete_modal(root);
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
    // Show UI in Editor or LevelTester mode
    for mut vis in &mut root_query {
        *vis = if *app_mode.get() == AppMode::Editor || *app_mode.get() == AppMode::LevelTester {
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
                    col.0 = theme::TEXT_PRIMARY;
                }
            } else {
                text.0 = "BLOCK INSPECTOR".into();
                if let Some(mut col) = text_col_opt {
                    col.0 = theme::TEXT_PRIMARY;
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
                theme::BTN_ACTIVE
            } else {
                theme::BTN_NORMAL
            };
        } else if let Some(prop_btn) = prop_opt {
            if editor.selected_kind.is_some() {
                let is_fixed_btn = prop_btn.0;
                if is_fixed_btn {
                    if !can_fixed {
                        bg.0 = theme::BTN_DISABLED;
                    } else if editor.is_fixed {
                        bg.0 = theme::BTN_ACTIVE;
                    } else {
                        bg.0 = theme::BTN_NORMAL;
                    }
                } else {
                    if !can_moveable {
                        bg.0 = theme::BTN_DISABLED;
                    } else if !editor.is_fixed {
                        bg.0 = theme::BTN_ACTIVE;
                    } else {
                        bg.0 = theme::BTN_NORMAL;
                    }
                }
            } else {
                bg.0 = theme::BTN_NORMAL;
            }
        } else if let Some(z_btn) = z_mode_opt {
            bg.0 = if z_btn.0 == editor.z_mode {
                theme::BTN_ACTIVE
            } else {
                theme::BTN_NORMAL
            };
        } else if combine_opt.is_some() {
            bg.0 = if all_selected_moveable {
                theme::BTN_SUCCESS
            } else {
                theme::BTN_DISABLED
            };
        } else if uncombine_opt.is_some() {
            bg.0 = if has_any_combined {
                theme::BTN_NORMAL
            } else {
                theme::BTN_DISABLED
            };
        } else if toggle_fixed_opt.is_some() {
            bg.0 = if can_toggle_fixed {
                theme::BTN_NORMAL
            } else {
                theme::BTN_DISABLED
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
                    color.0 = theme::TEXT_MUTED;
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
                bg.0 = if has_val_err { theme::BTN_DISABLED } else { theme::BTN_NORMAL };
            }
            EditorAction::TestWithSolution => {
                bg.0 = if has_val_err || (editor.solutions.is_empty() && !cached_valid) {
                    theme::BTN_DISABLED
                } else {
                    theme::BTN_SUCCESS
                };
            }
            EditorAction::Undo => {
                bg.0 = if editor.can_undo() { theme::BTN_NORMAL } else { theme::BTN_DISABLED };
            }
            EditorAction::Redo => {
                bg.0 = if editor.can_redo() { theme::BTN_NORMAL } else { theme::BTN_DISABLED };
            }
            EditorAction::ToggleFloorplanModal => {
                bg.0 = if editor.floorplan_open { theme::BTN_ACTIVE } else { theme::BTN_NORMAL };
            }
            EditorAction::ToggleFramePreview => {
                bg.0 = if editor.show_frame1_preview { theme::BTN_ACTIVE } else { theme::BTN_NORMAL };
            }
            _ => {}
        }
    }
}
