use std::{collections::BTreeSet, time::Duration};

use bevy::{prelude::*, utils::HashMap};

static NEIGHBOR_DELTA: [(isize, isize); 8] = [
    (-1, -1), (0, -1), (1, -1),
    (-1,  0),          (1,  0),
    (-1,  1), (0,  1), (1,  1),
];

#[derive(SystemSet, Hash, Clone, Debug, Eq, PartialEq)]
pub struct CellSet;

#[derive(Clone, Component, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct CellPosition {
    pub x: isize,
    pub y: isize,
}

#[derive(Resource, Debug)]
pub struct CellParams {
    pub playing: bool,
    pub period: Duration,
    pub compute_next_generation: bool,
}

impl Default for CellParams {
    fn default() -> Self {
        CellParams {
            playing: true,
            period: Duration::from_millis(1),
            compute_next_generation: false,
        }
    }
}

#[derive(Resource)]
pub struct NextGenerationTimer(Timer);

pub struct CellSystem;

impl Plugin for CellSystem {
    fn build(&self, app: &mut App) {
        let cell_params = CellParams::default();
        let period = cell_params.period;
        app.insert_resource(cell_params)
            .insert_resource(NextGenerationTimer(Timer::new(period, TimerMode::Repeating)))
            .add_systems(Update, check_for_param_change)
            .add_systems(Startup, init_cells.in_set(CellSet))
            .add_systems(Update, system_cells.in_set(CellSet));
    }
}

fn init_cells(mut commands: Commands) {
    commands.spawn(CellPosition { x: 0, y: 0 });
    commands.spawn(CellPosition { x: -1, y: 0 });
    commands.spawn(CellPosition { x: 0, y: -1 });
    commands.spawn(CellPosition { x: 0, y: 1 });
    commands.spawn(CellPosition { x: 1, y: 1 });
}

fn check_for_param_change(res: Res<CellParams>, mut timer: ResMut<NextGenerationTimer>) {
    if !res.is_changed() {
        return;
    }

    if res.period != timer.0.duration() {
        timer.0.set_duration(res.period);
        timer.0.reset();
    }
}

fn system_cells(
    mut commands: Commands,
    query: Query<(Entity, &CellPosition)>,
    mut timer: ResMut<NextGenerationTimer>,
    mut cell_params: ResMut<CellParams>,
    time: Res<Time>,
) {
    if cell_params.playing {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            cell_params.compute_next_generation = true;
        }
    } else if cell_params.compute_next_generation {
        cell_params.compute_next_generation = false;
    } else {
        return;
    }

    let mut neighbours = HashMap::new();
    let mut spawn_candidates = BTreeSet::new();

    for (_, cell) in &query {
        for position_delta in NEIGHBOR_DELTA.iter() {
            let scan_pos = CellPosition {
                x: cell.x + position_delta.0,
                y: cell.y + position_delta.1,
            };
            
            let neighbours_count = match neighbours.get(&scan_pos) {
                Some(previous_value) => previous_value + 1,
                None => 1,
            };

            neighbours.insert(scan_pos.clone(), neighbours_count);
            if neighbours_count == 3 {
                spawn_candidates.insert(scan_pos);
            } else if neighbours_count == 4 {
                spawn_candidates.remove(&scan_pos);
            }
        }
    }

    for (entity, cell) in &query {
        let neighbours_count = *neighbours.get(cell).unwrap_or(&0);

        match neighbours_count {
            0..=1 => commands.entity(entity).despawn(),
            2 => (),
            3 => {
                spawn_candidates.remove(cell);
            },
            _ => commands.entity(entity).despawn(),
        }
    }

    for new_cell in spawn_candidates {
        commands.spawn(new_cell);
    }
}