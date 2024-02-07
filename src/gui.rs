/*- Global allowings -*/

/*- Imports -*/
use bevy::{input::mouse::MouseWheel, prelude::*, window::PrimaryWindow};
use bevy_egui::{
    egui::{self, Color32},
    EguiContexts,
};
// use rand::Rng;
use crate::cell_system::{CellParams, CellPosition, CellSet};

/*- Constants -*/
const BG_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);
// const CELL_COLOR: Color = Color::rgb(0., 0.9, 0.);
const SCALE_DEFAULT: f32 = 1. / 15.;
const SCALE_MAX: f32 = 1.0;
const GENERATION_SIZE: isize = 100;

/*- Structs, enums & unions -*/
pub struct GuiSystem;

#[derive(Resource, Debug)]
pub struct GuiParams {
    pub grid_enabled: bool,
}

impl Default for GuiParams {
    fn default() -> Self {
        GuiParams {
            grid_enabled: false,
        }
    }
}

impl Plugin for GuiSystem {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(BG_COLOR))
            .insert_resource(GuiParams::default())
            .add_systems(Startup, init_camera)
            .add_systems(Update, system_control_mouse)
            .add_systems(Update, system_control_keyboard)
            .add_systems(Update, system_control_scroll)
            .add_systems(Update, system_draw_new_cells.before(CellSet))
            .add_systems(
                Update,
                system_draw_grid
                    .after(system_draw_new_cells)
                    .run_if(|gui_params: Res<GuiParams>| gui_params.grid_enabled));
    }
}

/*- Initialize -*/
fn init_camera(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scale = SCALE_DEFAULT;
    commands.spawn(camera);
}

fn system_control_scroll(
    mut scroll: EventReader<MouseWheel>,
    mut q_camera: Query<&mut OrthographicProjection, With<GlobalTransform>>,
) {
    const SCALE_INTENSITY: f32 = 0.05;
    let mut camera_proj = q_camera.get_single_mut().unwrap();

    for ev in scroll.read() {
        camera_proj.scale = (camera_proj.scale - ev.y * SCALE_INTENSITY).min(SCALE_MAX).max(0.05);
    }
}

fn system_control_keyboard(
    keys: Res<Input<KeyCode>>,
    mut q_camera_transform: Query<&mut Transform, With<Camera>>,
    mut cell_params: ResMut<CellParams>,
    // mut gui_params: ResMut<GuiParams>,
    q_camera_proj: Query<&OrthographicProjection>,
    q_cells: Query<Entity, With<CellPosition>>,
    mut commands: Commands,
) {

    //Camera controls
    let zoom = q_camera_proj.single().scale;
    let mut speed = 2. * zoom.max(0.2);
    let (mut x, mut y) = (0, 0);
    if keys.pressed(KeyCode::A) { x -= 1 }
    if keys.pressed(KeyCode::D) { x += 1 }
    if keys.pressed(KeyCode::S) { y -= 1 }
    if keys.pressed(KeyCode::W) { y += 1 }
    if keys.pressed(KeyCode::ShiftLeft) { speed = speed * 2.}

    let mut transform = q_camera_transform.single_mut();
    transform.translation += Vec3::new(x as f32 * speed, y as f32 * speed, 0.);

    //Simulation Controls
    if keys.any_just_pressed([KeyCode::Space, KeyCode::K]) {
        cell_params.playing = !cell_params.playing;
    }
    if keys.just_pressed(KeyCode::L) && !cell_params.playing {
        cell_params.compute_next_generation = true;
    }

    if keys.pressed(KeyCode::R) {
        cell_params.playing = false;
        clear_cells(&mut commands, &q_cells);
        // random_cells(&mut commands, -GENERATION_SIZE/2, -GENERATION_SIZE/2, GENERATION_SIZE, GENERATION_SIZE);
    }
}

fn system_control_mouse(
    mut commands: Commands,
    buttons: Res<Input<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    q_cellpos: Query<(Entity, &CellPosition)>,
    // cell_params: Res<CellParams>,
) {

    if !buttons.just_pressed(MouseButton::Left) { // cell_params.playing || 
        return;
    }

    let Some(cursor_position) = q_windows.single().cursor_position() else {
        return;
    };
    let (camera, camera_transform) = q_camera.single();
    // if buttons.pressed(MouseButton::Left) {
    let Some(target_pos) = camera.viewport_to_world(camera_transform, cursor_position).map(|ray| ray.origin.truncate().round())
    else { return };
    let new_cell = CellPosition {
        x: target_pos.x as isize,
        y: target_pos.y as isize,
    };
    for (entity, cell_pos) in q_cellpos.iter() {
        if cell_pos == &new_cell {
            commands.entity(entity).despawn();
            return;
        }
    }
    // println!("Cell created at: {:?}", target_pos);
    
    commands.spawn(new_cell);
    // }
}

fn system_draw_new_cells(
    mut commands: Commands,
    query: Query<(Entity, &CellPosition), Added<CellPosition>>,
) {
    for (entity, position) in query.iter() {
        let highest_pos_value: f32 = return_highest_value(position.x.abs(), position.y.abs()) as f32 % (GENERATION_SIZE as f32 * 1.5);
        let color_boundaries: f32 = GENERATION_SIZE as f32 / 2.;
        let percentage_half_generation_size: f32 = highest_pos_value / color_boundaries;
        commands.entity(entity)
            .insert(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb( 
                        (1. - percentage_half_generation_size).max(0.) + (percentage_half_generation_size - 2.).max(0.),
                        (percentage_half_generation_size).min(1.) - (percentage_half_generation_size - 1.).max(0.),
                        (percentage_half_generation_size - 1.).max(0.) - (highest_pos_value / (color_boundaries / 2.) - 4.).max(0.)
                    ),
                    custom_size: Some(Vec2::new(1., 1.)),
                    ..Default::default()
                },
                transform: Transform::from_translation(Vec3::new(position.x as f32, position.y as f32, 0.)),
                ..Default::default()
            });
    }
}

fn system_draw_grid(
    mut contexts: EguiContexts,
    q_camera: Query<(&Camera, &OrthographicProjection, &GlobalTransform)>
) {
    const LINE_COLOR: Color32 = Color32::BLACK;
    let (camera, camera_proj, camera_transform) = q_camera.get_single().unwrap();
    let ctx = contexts.ctx_mut();
    let transparent_frame = egui::containers::Frame {
        fill: Color32::TRANSPARENT,
        ..Default::default()
    };
    let line_width =
        (1. / (camera_proj.scale - SCALE_DEFAULT) / (SCALE_MAX - SCALE_DEFAULT)).powi(10);
    egui::CentralPanel::default()
        .frame(transparent_frame)
        .show(ctx, |ui| {
            let (response, painter) = ui.allocate_painter(
                bevy_egui::egui::Vec2::new(ui.available_width(), ui.available_height()),
                egui::Sense {
                    click: false,
                    drag: false,
                    focusable: false,
                },
            );
            let visible_top_left = camera
                .viewport_to_world(camera_transform, Vec2 { x: 0., y: 0. })
                .map(|ray| ray.origin.truncate())
                .unwrap();
            let (x_min, y_max) = (
                visible_top_left.x.round() as isize,
                visible_top_left.y.round() as isize,
            );
            let visible_bottom_right = camera
                .viewport_to_world(
                    camera_transform,
                    Vec2 {
                        x: response.rect.right(),
                        y: response.rect.bottom(),
                    },
                )
                .map(|ray| ray.origin.truncate())
                .unwrap();
            let (x_max, y_min) = (
                visible_bottom_right.x.round() as isize,
                visible_bottom_right.y.round() as isize,
            );

            for x in x_min..=x_max {
                let start = camera
                    .world_to_viewport(
                        camera_transform,
                        Vec3 {
                            x: x as f32 - 0.5,
                            y: y_min as f32 - 0.5,
                            z: 0.,
                        },
                    )
                    .unwrap();
                let start = egui::Pos2::new(start.x, start.y);
                let end = camera
                    .world_to_viewport(
                        camera_transform,
                        Vec3 {
                            x: x as f32 - 0.5,
                            y: y_max as f32 + 0.5,
                            z: 0.,
                        },
                    )
                    .unwrap();
                let end = egui::Pos2::new(end.x, end.y);
                painter.add(egui::Shape::LineSegment {
                    points: [start, end],
                    stroke: egui::Stroke {
                        width: line_width,
                        color: LINE_COLOR,
                    },
                });
            }
            for y in y_min..=y_max {
                let start = camera
                    .world_to_viewport(
                        camera_transform,
                        Vec3 {
                            x: x_min as f32 - 0.5,
                            y: y as f32 - 0.5,
                            z: 0.,
                        },
                    )
                    .unwrap();
                let start = egui::Pos2::new(start.x, start.y);
                let end = camera
                    .world_to_viewport(
                        camera_transform,
                        Vec3 {
                            x: x_max as f32 + 0.5,
                            y: y as f32 - 0.5,
                            z: 0.,
                        },
                    )
                    .unwrap();
                let end = egui::Pos2::new(end.x, end.y);
                painter.add(egui::Shape::LineSegment {
                    points: [start, end],
                    stroke: egui::Stroke {
                        width: line_width,
                        color: LINE_COLOR
                    },
                });
            }
        });

}

/*- Method implementations - */
fn clear_cells(
    commands: &mut Commands,
    q_cells: &Query<Entity, With<CellPosition>>
) {
    for entity in q_cells.iter() {
        commands.entity(entity).despawn();
    }
}

// fn random_cells(
//     commands: &mut Commands,
//     x: isize,
//     y: isize,
//     width: isize,
//     height: isize,
// ) {
//     let mut rng = rand::thread_rng();
//     for coord_x in x..(x + width as isize) {
//         for coord_y in y..(y + height as isize) {
//             if rng.gen::<bool>() {
//                 commands.spawn(CellPosition { x: coord_x, y: coord_y });
//             }
//         }
//     }
// }

fn return_highest_value(val1: isize, val2: isize) -> isize {
    if val1 > val2 {
        return val1;
    } else {
        return val2;
    }
}

