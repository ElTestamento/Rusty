use bevy::prelude::*;
use std::collections::HashSet;
use world::{Particle as SimParticle, World as SimWorld};

const GRID_WIDTH: usize = 40;
const GRID_HEIGHT: usize = 30;
const CELL_SIZE: f32 = 20.0;

#[derive(Component)]
struct ParticleSprite(usize);

#[derive(Resource)]
struct Simulation {
    world: SimWorld,
    particles: Vec<SimParticle>,
    gravity: [f32; 2],
}

#[derive(Resource)]
struct SimTimer(Timer);

#[derive(Resource)]
struct SpawnTimer(Timer);

#[derive(Resource)]
struct ParticleCounter(i32);

#[derive(Resource)]
struct BlockCounter(i32);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Sand Simulation".into(),
                resolution: (GRID_WIDTH as f32 * CELL_SIZE, GRID_HEIGHT as f32 * CELL_SIZE).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Simulation {
            world: SimWorld::new(GRID_HEIGHT, GRID_WIDTH),
            particles: Vec::new(),
            gravity: [0.0, -1.0],
        })
        .insert_resource(SimTimer(Timer::from_seconds(0.05, TimerMode::Repeating)))
        .insert_resource(SpawnTimer(Timer::from_seconds(0.08, TimerMode::Repeating)))
        .insert_resource(ParticleCounter(0))
        .insert_resource(BlockCounter(0))
        .add_systems(Startup, setup)
        .add_systems(Update, (spawn_particles, spawn_heavy_block, run_simulation, update_sprites))
        .run();
}

fn setup(mut commands: Commands, mut sim: ResMut<Simulation>) {
    commands.spawn(Camera2dBundle::default());

    // Boden
    for x in 0..GRID_WIDTH {
        sim.world.grid[0][x] = (true, 1000.0, 0.0);
        spawn_block(&mut commands, x as i32, 0, Color::GRAY);
    }

    // Klotz in der Mitte
    let klotz_x = GRID_WIDTH as i32 / 2 - 3;
    for dx in 0..6 {
        for dy in 1..5 {
            let x = (klotz_x + dx) as usize;
            let y = dy as usize;
            sim.world.grid[y][x] = (true, 1000.0, 0.0);
            spawn_block(&mut commands, klotz_x + dx, dy, Color::DARK_GRAY);
        }
    }
}

fn spawn_block(commands: &mut Commands, x: i32, y: i32, color: Color) {
    let screen_x = (x as f32 - GRID_WIDTH as f32 / 2.0 + 0.5) * CELL_SIZE;
    let screen_y = (y as f32 - GRID_HEIGHT as f32 / 2.0 + 0.5) * CELL_SIZE;

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color,
            custom_size: Some(Vec2::new(CELL_SIZE - 1.0, CELL_SIZE - 1.0)),
            ..default()
        },
        transform: Transform::from_xyz(screen_x, screen_y, 0.0),
        ..default()
    });
}

fn spawn_particles(
    mut commands: Commands,
    mut sim: ResMut<Simulation>,
    mut spawn_timer: ResMut<SpawnTimer>,
    mut counter: ResMut<ParticleCounter>,
    time: Res<Time>,
) {
    spawn_timer.0.tick(time.delta());

    if spawn_timer.0.just_finished() && counter.0 < 300 {
        let spawn_x = GRID_WIDTH as i32 / 2 + (rand::random::<i32>() % 3) - 1;
        let spawn_y = (GRID_HEIGHT - 2) as i32;

        if !sim.world.give_occupation_on_position(spawn_x as usize, spawn_y as usize) {
            counter.0 += 1;

            let particle = SimParticle::new(
                counter.0,
                [spawn_x as f32, spawn_y as f32],
                [0.0, 0.0],
                10.0,
            );

            sim.world.update_occupation_on_position(particle.position);
            sim.world.update_mass_on_position(particle.position, particle.mass);

            let idx = sim.particles.len();
            sim.particles.push(particle);

            let screen_x = (spawn_x as f32 - GRID_WIDTH as f32 / 2.0 + 0.5) * CELL_SIZE;
            let screen_y = (spawn_y as f32 - GRID_HEIGHT as f32 / 2.0 + 0.5) * CELL_SIZE;

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(0.9, 0.75, 0.4),
                        custom_size: Some(Vec2::new(CELL_SIZE - 1.0, CELL_SIZE - 1.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(screen_x, screen_y, 1.0),
                    ..default()
                },
                ParticleSprite(idx),
            ));
        }
    }
}

fn spawn_heavy_block(
    mut commands: Commands,
    mut sim: ResMut<Simulation>,
    mut counter: ResMut<ParticleCounter>,
    mut block_counter: ResMut<BlockCounter>,
    mouse_button: Res<Input<MouseButton>>,
    windows: Query<&Window>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        let window = windows.single();

        if let Some(cursor_pos) = window.cursor_position() {
            let grid_x = (cursor_pos.x / CELL_SIZE) as i32;
            let grid_y = (GRID_HEIGHT as f32 - cursor_pos.y / CELL_SIZE) as i32;

            block_counter.0 += 1;
            let current_block_id = block_counter.0;

            for dx in -1..=1 {
                for dy in -1..=1 {
                    let x = grid_x + dx;
                    let y = grid_y + dy;

                    if x < 0 || x >= GRID_WIDTH as i32 || y < 0 || y >= GRID_HEIGHT as i32 {
                        continue;
                    }

                    if !sim.world.give_occupation_on_position(x as usize, y as usize) {
                        counter.0 += 1;

                        let particle = SimParticle::new_solid(
                            counter.0,
                            [x as f32, y as f32],
                            100.0,
                            current_block_id,
                        );

                        sim.world.update_occupation_on_position(particle.position);
                        sim.world.update_mass_on_position(particle.position, particle.mass);

                        let idx = sim.particles.len();
                        sim.particles.push(particle);

                        let screen_x = (x as f32 - GRID_WIDTH as f32 / 2.0 + 0.5) * CELL_SIZE;
                        let screen_y = (y as f32 - GRID_HEIGHT as f32 / 2.0 + 0.5) * CELL_SIZE;

                        commands.spawn((
                            SpriteBundle {
                                sprite: Sprite {
                                    color: Color::rgb(0.4, 0.2, 0.1),
                                    custom_size: Some(Vec2::new(CELL_SIZE - 1.0, CELL_SIZE - 1.0)),
                                    ..default()
                                },
                                transform: Transform::from_xyz(screen_x, screen_y, 2.0),
                                ..default()
                            },
                            ParticleSprite(idx),
                        ));
                    }
                }
            }
            println!("Block {} bei ({}, {}) gespawnt!", current_block_id, grid_x, grid_y);
        }
    }
}

fn run_simulation(
    mut sim: ResMut<Simulation>,
    mut timer: ResMut<SimTimer>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());

    if !timer.0.just_finished() {
        return;
    }

    sim.world.calc_pressure_on_all_position();

    let gravity = sim.gravity;
    let Simulation { world, particles, .. } = &mut *sim;

    // Prüfen welche Blöcke blockiert sind
    let mut blocked_blocks: HashSet<i32> = HashSet::new();

    for particle in particles.iter() {
        if let Some(block_id) = particle.block_id {
            let x = particle.position[0] as i32;
            let y = particle.position[1] as i32;

            if y <= 0 {
                blocked_blocks.insert(block_id);
            } else {
                let below_occupied = world.give_occupation_on_position(x as usize, (y - 1) as usize);
                if below_occupied {
                    let is_same_block = particles.iter().any(|p| {
                        p.block_id == Some(block_id) &&
                            p.position[0] as i32 == x &&
                            p.position[1] as i32 == y - 1
                    });
                    if !is_same_block {
                        blocked_blocks.insert(block_id);
                    }
                }
            }
        }
    }

    // Velocity update
    for particle in particles.iter_mut() {
        particle.update_velocity(gravity, world);
    }

    // Position update
    for particle in particles.iter_mut() {
        particle.update_position(world);
    }

    // Pressure resolve
    for particle in particles.iter_mut() {
        particle.resolve_pressure(world);
    }

    // Fall down - Blöcke nur wenn nicht blockiert
    for particle in particles.iter_mut() {
        if let Some(block_id) = particle.block_id {
            if !blocked_blocks.contains(&block_id) {
                particle.fall_down(world);
            }
        } else {
            particle.fall_down(world);
        }
    }
}

fn update_sprites(
    sim: Res<Simulation>,
    mut query: Query<(&ParticleSprite, &mut Transform)>,
) {
    for (particle_sprite, mut transform) in query.iter_mut() {
        let particle = &sim.particles[particle_sprite.0];
        let screen_x = (particle.position[0] - GRID_WIDTH as f32 / 2.0 + 0.5) * CELL_SIZE;
        let screen_y = (particle.position[1] - GRID_HEIGHT as f32 / 2.0 + 0.5) * CELL_SIZE;
        transform.translation.x = screen_x;
        transform.translation.y = screen_y;
    }
}

