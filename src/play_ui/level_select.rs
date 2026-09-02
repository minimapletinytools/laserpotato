//! Navigable Shipped Level Browser dialog with folder navigation, sorting, and scrollbar.

use bevy::prelude::*;
use crate::editor::ui::theme;
use crate::play_ui::catalog::{CatalogSortColumn, LevelCatalog};

#[derive(Component)]
pub struct LevelSelectRoot;

#[derive(Component)]
pub struct BrowserDirText;

#[derive(Component)]
pub struct BrowserUpBtn;

#[derive(Component)]
pub struct BrowserFolderBtn(pub String);

#[derive(Component)]
pub struct BrowserLevelRowBtn(pub usize);

#[derive(Component)]
pub struct BrowserPlayBtn(pub usize);

#[derive(Component)]
pub struct BrowserSortHeaderBtn(pub CatalogSortColumn);

#[derive(Component)]
pub struct BrowserListContainer;

#[derive(Component)]
pub struct BrowserItem;

#[derive(Component)]
pub struct BrowserScrollBarTrack;

#[derive(Component)]
pub struct BrowserScrollBarThumb;

#[derive(Component)]
pub struct BrowserScrollUpBtn;

#[derive(Component)]
pub struct BrowserScrollDownBtn;

#[derive(Component)]
pub struct BrowserScrollPageUpBtn;

#[derive(Component)]
pub struct BrowserScrollPageDownBtn;

#[derive(Component)]
pub struct BrowserScrollStatusText;

#[derive(Component)]
pub struct BrowserDetailsCard;

#[derive(Component)]
pub struct BrowserDetailsTitleText;

#[derive(Component)]
pub struct BrowserDetailsStatsText;

#[derive(Component)]
pub struct BrowserDetailsPathText;

#[derive(Component)]
pub struct BrowserStartGameBtn;

/// Spawns the complete Shipped Level Browser screen into the UI hierarchy.
pub fn spawn_level_select_screen(commands: &mut Commands, _catalog: &LevelCatalog) {
    commands
        .spawn((
            LevelSelectRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
        ))
        .with_children(|root| {
            // Top Bar
            root.spawn(Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                border: UiRect::bottom(Val::Px(1.0)),
                ..default()
            })
            .insert(BackgroundColor(theme::PANEL_BG))
            .insert(BorderColor::all(theme::BORDER_SUBTLE))
            .with_children(|top| {
                top.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(12.0),
                    ..default()
                })
                .with_children(|left| {
                    left.spawn((
                        Text::new("LASER POTATO"),
                        TextFont::from_font_size(22.0),
                        TextColor(theme::TEXT_GOLD),
                    ));
                    left.spawn((
                        Text::new("[SHIPPED LEVEL BROWSER]"),
                        TextFont::from_font_size(14.0),
                        TextColor(theme::TEXT_CYAN),
                    ));
                });

                top.spawn((
                    Text::new("Select a puzzle level or browse folders below"),
                    TextFont::from_font_size(13.0),
                    TextColor(theme::TEXT_MUTED),
                ));
            });

            // Middle Content: Left Browser Dialog + Right Detail Inspector
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                padding: UiRect::axes(Val::ZERO, Val::Px(10.0)),
                column_gap: Val::Px(16.0),
                ..default()
            })
            .with_children(|middle| {
                // Left Panel: Table Browser
                middle
                    .spawn((
                        Node {
                            width: Val::Px(560.0),
                            height: Val::Px(560.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(10.0)),
                            row_gap: Val::Px(6.0),
                            border: UiRect::all(Val::Px(1.5)),
                            ..default()
                        },
                        BackgroundColor(theme::PANEL_BG),
                        BorderColor::all(Color::srgba(0.25, 0.35, 0.5, 0.8)),
                    ))
                    .with_children(|panel| {
                        // Directory Navigation Bar
                        panel
                            .spawn(Node {
                                width: Val::Percent(100.0),
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                padding: UiRect::axes(Val::Px(8.0), Val::Px(5.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            })
                            .insert(BackgroundColor(Color::srgba(0.10, 0.12, 0.16, 1.0)))
                            .insert(BorderColor::all(Color::srgba(0.3, 0.45, 0.7, 0.6)))
                            .with_children(|dir_bar| {
                                dir_bar.spawn((
                                    BrowserDirText,
                                    Text::new("Directory: shipped/"),
                                    TextFont::from_font_size(12.0),
                                    TextColor(theme::TEXT_CYAN),
                                ));

                                dir_bar
                                    .spawn((
                                        BrowserUpBtn,
                                        Button,
                                        Node {
                                            padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                                            border: UiRect::all(Val::Px(1.0)),
                                            ..default()
                                        },
                                        BackgroundColor(theme::BTN_NORMAL),
                                        BorderColor::all(theme::BORDER_SUBTLE),
                                    ))
                                    .with_children(|b| {
                                        b.spawn((
                                            Text::new("[UP]"),
                                            TextFont::from_font_size(11.0),
                                            TextColor(theme::TEXT_PRIMARY),
                                        ));
                                    });
                            });

                        // Table Header Row
                        panel
                            .spawn(Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(28.0),
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                                border: UiRect::bottom(Val::Px(1.0)),
                                ..default()
                            })
                            .insert(BackgroundColor(Color::srgba(0.14, 0.16, 0.22, 1.0)))
                            .insert(BorderColor::all(theme::BORDER_SUBTLE))
                            .with_children(|header| {
                                header
                                    .spawn((
                                        BrowserSortHeaderBtn(CatalogSortColumn::Name),
                                        Button,
                                        Node {
                                            width: Val::Percent(46.0),
                                            ..default()
                                        },
                                    ))
                                    .with_children(|b| {
                                        b.spawn((
                                            Text::new("Level / Folder Name"),
                                            TextFont::from_font_size(11.0),
                                            TextColor(theme::TEXT_PRIMARY),
                                        ));
                                    });

                                header
                                    .spawn((
                                        BrowserSortHeaderBtn(CatalogSortColumn::Moves),
                                        Button,
                                        Node {
                                            width: Val::Percent(18.0),
                                            ..default()
                                        },
                                    ))
                                    .with_children(|b| {
                                        b.spawn((
                                            Text::new("Moves"),
                                            TextFont::from_font_size(11.0),
                                            TextColor(theme::TEXT_ACCENT),
                                        ));
                                    });

                                header
                                    .spawn((
                                        BrowserSortHeaderBtn(CatalogSortColumn::Difficulty),
                                        Button,
                                        Node {
                                            width: Val::Percent(20.0),
                                            ..default()
                                        },
                                    ))
                                    .with_children(|b| {
                                        b.spawn((
                                            Text::new("Diff"),
                                            TextFont::from_font_size(11.0),
                                            TextColor(theme::TEXT_GOLD),
                                        ));
                                    });

                                header
                                    .spawn((
                                        BrowserSortHeaderBtn(CatalogSortColumn::Blocks),
                                        Button,
                                        Node {
                                            width: Val::Percent(16.0),
                                            ..default()
                                        },
                                    ))
                                    .with_children(|b| {
                                        b.spawn((
                                            Text::new("Blocks"),
                                            TextFont::from_font_size(11.0),
                                            TextColor(theme::TEXT_MUTED),
                                        ));
                                    });
                            });

                        // Main List + Scroll Bar Container
                        panel
                            .spawn(Node {
                                width: Val::Percent(100.0),
                                flex_grow: 1.0,
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(6.0),
                                ..default()
                            })
                            .with_children(|content_row| {
                                // List container for dynamic rows
                                content_row.spawn((
                                    BrowserListContainer,
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
                                    BackgroundColor(Color::srgba(0.04, 0.05, 0.07, 0.8)),
                                    BorderColor::all(Color::srgba(0.2, 0.25, 0.35, 0.6)),
                                ));

                                // Vertical Scrollbar
                                content_row
                                    .spawn((
                                        Node {
                                            width: Val::Px(28.0),
                                            height: Val::Percent(100.0),
                                            flex_direction: FlexDirection::Column,
                                            align_items: AlignItems::Center,
                                            justify_content: JustifyContent::SpaceBetween,
                                            border: UiRect::all(Val::Px(1.0)),
                                            ..default()
                                        },
                                        BackgroundColor(theme::SCROLLBAR_TRACK),
                                        BorderColor::all(theme::BORDER_SUBTLE),
                                    ))
                                    .with_children(|sb| {
                                        sb.spawn((
                                            BrowserScrollUpBtn,
                                            Button,
                                            Node {
                                                width: Val::Percent(100.0),
                                                height: Val::Px(20.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            BackgroundColor(theme::BTN_NORMAL),
                                        ))
                                        .with_children(|b| {
                                            b.spawn((Text::new("^"), TextFont::from_font_size(11.0), TextColor(theme::TEXT_PRIMARY)));
                                        });

                                        sb.spawn((
                                            BrowserScrollBarTrack,
                                            Node {
                                                width: Val::Percent(100.0),
                                                flex_grow: 1.0,
                                                position_type: PositionType::Relative,
                                                ..default()
                                            },
                                        ))
                                        .with_children(|track| {
                                            track.spawn((
                                                BrowserScrollBarThumb,
                                                Node {
                                                    position_type: PositionType::Absolute,
                                                    width: Val::Percent(100.0),
                                                    top: Val::Percent(0.0),
                                                    height: Val::Percent(30.0),
                                                    ..default()
                                                },
                                                BackgroundColor(theme::SCROLLBAR_THUMB),
                                            ));
                                        });

                                        sb.spawn((
                                            BrowserScrollDownBtn,
                                            Button,
                                            Node {
                                                width: Val::Percent(100.0),
                                                height: Val::Px(20.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            BackgroundColor(theme::BTN_NORMAL),
                                        ))
                                        .with_children(|b| {
                                            b.spawn((Text::new("v"), TextFont::from_font_size(11.0), TextColor(theme::TEXT_PRIMARY)));
                                        });
                                    });
                            });

                        // Pagination & Status Footer
                        panel
                            .spawn(Node {
                                width: Val::Percent(100.0),
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
                                border: UiRect::top(Val::Px(1.0)),
                                ..default()
                            })
                            .insert(BorderColor::all(theme::BORDER_SUBTLE))
                            .with_children(|footer| {
                                footer.spawn((
                                    BrowserScrollStatusText,
                                    Text::new("Showing 1-10 of 10 items"),
                                    TextFont::from_font_size(11.0),
                                    TextColor(theme::TEXT_MUTED),
                                ));

                                footer
                                    .spawn(Node {
                                        flex_direction: FlexDirection::Row,
                                        column_gap: Val::Px(4.0),
                                        ..default()
                                    })
                                    .with_children(|pg_btns| {
                                        pg_btns
                                            .spawn((
                                                BrowserScrollPageUpBtn,
                                                Button,
                                                Node {
                                                    padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                                                    border: UiRect::all(Val::Px(1.0)),
                                                    ..default()
                                                },
                                                BackgroundColor(theme::BTN_NORMAL),
                                                BorderColor::all(theme::BORDER_SUBTLE),
                                            ))
                                            .with_children(|b| {
                                                b.spawn((Text::new("<<"), TextFont::from_font_size(10.0), TextColor(theme::TEXT_PRIMARY)));
                                            });

                                        pg_btns
                                            .spawn((
                                                BrowserScrollPageDownBtn,
                                                Button,
                                                Node {
                                                    padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                                                    border: UiRect::all(Val::Px(1.0)),
                                                    ..default()
                                                },
                                                BackgroundColor(theme::BTN_NORMAL),
                                                BorderColor::all(theme::BORDER_SUBTLE),
                                            ))
                                            .with_children(|b| {
                                                b.spawn((Text::new(">>"), TextFont::from_font_size(10.0), TextColor(theme::TEXT_PRIMARY)));
                                            });
                                    });
                            });
                    });

                // Right Panel: Selected Level Detail & Play Action Card
                middle
                    .spawn((
                        BrowserDetailsCard,
                        Node {
                            width: Val::Px(360.0),
                            padding: UiRect::all(Val::Px(16.0)),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(12.0),
                            border: UiRect::all(Val::Px(1.5)),
                            align_self: AlignSelf::FlexEnd,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.06, 0.08, 0.12, 0.94)),
                        BorderColor::all(Color::srgba(0.3, 0.6, 0.9, 0.8)),
                    ))
                    .with_children(|card| {
                        card.spawn((
                            BrowserDetailsTitleText,
                            Text::new("1. First Light"),
                            TextFont::from_font_size(20.0),
                            TextColor(theme::TEXT_GOLD),
                        ));

                        card.spawn((
                            BrowserDetailsPathText,
                            Text::new("File: shipped/simple_1.json"),
                            TextFont::from_font_size(12.0),
                            TextColor(theme::TEXT_CYAN),
                        ));

                        card.spawn((
                            BrowserDetailsStatsText,
                            Text::new("Moves: 5 | Size: 7x7x1 | Blocks: 4"),
                            TextFont::from_font_size(13.0),
                            TextColor(theme::TEXT_PRIMARY),
                        ));

                        card.spawn((
                            BrowserStartGameBtn,
                            Button,
                            Node {
                                width: Val::Percent(100.0),
                                padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::top(Val::Px(8.0)),
                                border: UiRect::all(Val::Px(1.5)),
                                ..default()
                            },
                            BackgroundColor(theme::BTN_PRIMARY),
                            BorderColor::all(theme::BORDER_FOCUS),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("PLAY LEVEL >"),
                                TextFont::from_font_size(16.0),
                                TextColor(theme::TEXT_PRIMARY),
                            ));
                        });
                    });
            });

            // Bottom Footer Keybinding Reference
            root.spawn(Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(4.0)),
                ..default()
            })
            .with_children(|foot| {
                foot.spawn((
                    Text::new("Controls: [Enter / Space] Play Selected Level  |  [Esc] Toggle Browser  |  Mouse Drag to orbit 3D Preview"),
                    TextFont::from_font_size(12.0),
                    TextColor(theme::TEXT_MUTED),
                ));
            });
        });
}
