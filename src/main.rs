use bevy::{prelude::*};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BoxState {
    Empty,
    X,
    O,
}

impl std::fmt::Display for BoxState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BoxState::Empty => " ",
                BoxState::X => "X",
                BoxState::O => "O",
            }
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Player {
    X,
    O,
}

impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::X => "X",
                Self::O => "O",
            }
        )
    }
}


impl Into<BoxState> for Player {
    fn into(self) -> BoxState {
        match self {
            Self::X => BoxState::X,
            Self::O => BoxState::O,
        }
    }
}

struct GameState {
    grid: Vec<Vec<BoxState>>,
    current_player: Player,
    game_is_over: bool,
}

impl GameState {
    pub fn next_player(&mut self) {
        self.current_player = match self.current_player {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct GridButton {
    row: usize,
    col: usize,
}

struct InfoText;
struct NewGameButton;

/// This example illustrates how to create a button that changes color and text based on its
/// interaction state.
fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .init_resource::<ButtonMaterials>()
        .insert_resource(GameState {
            grid: vec![vec![BoxState::Empty; 3]; 3],
            current_player: Player::X,
            game_is_over: false,
        })
        .add_startup_system(setup.system())
        .add_system(button_system.system())
        .add_system(update_info_text.system())
        .run();
}

struct ButtonMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    pressed: Handle<ColorMaterial>,
}

impl FromWorld for ButtonMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
        }
    }
}

fn check_victory(game_state: &mut GameState, last_player: Player) -> Option<Player> {
    // rows
    for j in 0..3 {
        if game_state.grid[j].iter().all(|f| *f == last_player.into()) {
            return Some(last_player);
        }
    }

    // cols
    for i in 0..3 {
        let mut won = true;
        for j in 0..3 {
            if game_state.grid[j][i] != last_player.into() {
                won = false;
                break;
            }
        }
        if won {
            return Some(last_player);
        }
    }

    // diag-1
    if game_state.grid[0][0] == game_state.grid[1][1]
        && game_state.grid[1][1] == game_state.grid[2][2]
        && game_state.grid[2][2] == last_player.into()
    {
        return Some(last_player);
    }

    // diag-2
    if game_state.grid[0][2] == game_state.grid[1][1]
        && game_state.grid[1][1] == game_state.grid[2][0]
        && game_state.grid[2][0] == last_player.into()
    {
        return Some(last_player);
    }

    // else no victor
    None
}

fn update_info_text(
    game_state: ResMut<GameState>,
    mut text_query: Query<&mut Text, With<InfoText>>,
) {
    if let Ok(mut info_text) = text_query.single_mut() {
        info_text.sections[0].value = match game_state.game_is_over {
            false => format!("Player {}'s turn", game_state.current_player.to_string()),
            true => format!("Player {} Won!!", game_state.current_player.to_string())
        }
    }
}

fn button_system(
    button_materials: Res<ButtonMaterials>,
    mut game_state: ResMut<GameState>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut Handle<ColorMaterial>,
            &Children,
            &GridButton,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut material, children, grid_button) in interaction_query.iter_mut() {
        let mut text = text_query.get_mut(children[0]).unwrap();
        let mut box_state = game_state.grid[grid_button.row][grid_button.col];
        match *interaction {
            Interaction::Clicked => {
                if box_state == BoxState::Empty && !game_state.game_is_over {
                    box_state = game_state.current_player.into();
                    game_state.grid[grid_button.row][grid_button.col] = box_state;
                    let current_player = game_state.current_player;
                    match check_victory(&mut *game_state, current_player) {
                        Some(player) => {
                            info!("Player {:?} won! Good for them", player);
                            game_state.game_is_over = true;
                        }
                        None => game_state.next_player(),
                    }

                    text.sections[0].value = box_state.to_string();
                    *material = button_materials.pressed.clone();
                }
            }
            Interaction::Hovered => {
                text.sections[0].value = box_state.to_string();
                *material = button_materials.hovered.clone();
            }
            Interaction::None => {
                text.sections[0].value = box_state.to_string();
                *material = button_materials.normal.clone();
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    button_materials: Res<ButtonMaterials>,
) {
    commands.spawn_bundle(UiCameraBundle::default());

    let mut root_node = commands.spawn_bundle(NodeBundle {
        style: Style {
            justify_content: JustifyContent::Center,
            size: Size::new(Val::Percent(100.), Val::Percent(100.)),
            flex_direction: FlexDirection::ColumnReverse,
            ..Default::default()
        },
        material: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
        ..Default::default()
    });

    // TOP BAR
    root_node.with_children(|parent| {
        parent
            .spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(720.), Val::Px(60.)),
                    margin: Rect::all(Val::Auto),
                    align_content: AlignContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Row,
                    ..Default::default()
                },
                material: materials.add(Color::rgba(0.25, 0.25, 1.0, 0.0).into()),
                ..Default::default()
            })
            .with_children(|parent| {
                parent
                    .spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Player X's Turn",
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 40.0,
                                color: Color::rgb(0.0, 0.9, 0.2),
                            },
                            Default::default(),
                        ),
                        ..Default::default()
                    })
                    .insert(InfoText);

                // New Game Button
                parent
                    .spawn_bundle(ButtonBundle {
                        visible: Visible { is_visible: false, ..Default::default() },
                        style: Style {
                            margin: Rect::all(Val::Auto),
                            justify_content: JustifyContent::FlexEnd,
                            align_items: AlignItems::Center,
                            ..Default::default()
                        },
                        material: button_materials.normal.clone(),
                        ..Default::default()
                    })
                    .insert(NewGameButton)
                    .with_children(|parent| {
                        parent.spawn_bundle(TextBundle {
                            visible: Visible { is_visible: false, ..Default::default() },
                            text: Text::with_section(
                                "New Game",
                                TextStyle {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 40.0,
                                    color: Color::rgb(0.9, 0.9, 0.9),
                                },
                                Default::default(),
                            ),
                            ..Default::default()
                        });
                    });
            });
    });

    root_node.with_children(|parent| {
        let mut grid = parent.spawn_bundle(NodeBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                margin: Rect::all(Val::Auto),
                //size: Size::new(Val::Px(490.0), Val::Px(500.0)),
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            material: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            ..Default::default()
        });
        for j in 0..3 {
            grid.with_children(|parent| {
                let mut row = parent.spawn_bundle(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        size: Size::new(Val::Px(490.0), Val::Px(190.0)),
                        ..Default::default()
                    },
                    material: materials.add(Color::rgb(0.95, 0.25, 0.25).into()),
                    ..Default::default()
                });
                for i in 0..3 {
                    row.with_children(|parent| {
                        parent
                            .spawn_bundle(ButtonBundle {
                                style: Style {
                                    size: Size::new(Val::Px(150.0), Val::Px(150.0)),
                                    // center button
                                    margin: Rect::all(Val::Auto),
                                    // horizontally center child text
                                    justify_content: JustifyContent::Center,
                                    // vertically center child text
                                    align_items: AlignItems::Center,
                                    ..Default::default()
                                },
                                material: button_materials.normal.clone(),
                                ..Default::default()
                            })
                            .insert(GridButton { row: j, col: i })
                            .with_children(|parent| {
                                parent.spawn_bundle(TextBundle {
                                    text: Text::with_section(
                                        " ",
                                        TextStyle {
                                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                            font_size: 40.0,
                                            color: Color::rgb(0.9, 0.9, 0.9),
                                        },
                                        Default::default(),
                                    ),
                                    ..Default::default()
                                });
                            });
                    });
                }
            });
        }
    });
}
