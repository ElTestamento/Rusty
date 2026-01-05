use bevy::prelude::*;
use std::collections::HashSet;
use world::{Particle as SimParticle, World as SimWorld};

const GRID_WIDTH: usize = 40;
const GRID_HEIGHT: usize = 30;
const CELL_SIZE: f32 = 20.0;

// === KOMPONENTEN & RESSOURCEN ===

#[derive(Component)]
struct ParticleSprite(usize);

#[derive(Resource)]
struct Simulation {
    world: SimWorld,
    particles: Vec<SimParticle>,
    gravity: [f32; 2],
}

#[derive(Resource)]
struct Timers {
    sim: Timer,
    spawn: Timer,
}

#[derive(Resource)]
struct ParticleCounter(i32);

#[derive(Resource)]
struct BlockCounter(i32);

// === HELPER FUNKTIONEN ===

fn grid_to_screen(x: f32, y: f32) -> (f32, f32) {
    let screen_x = (x - GRID_WIDTH as f32 / 2.0 + 0.5) * CELL_SIZE;
    let screen_y = (y - GRID_HEIGHT as f32 / 2.0 + 0.5) * CELL_SIZE;
    (screen_x, screen_y)
}

fn spawn_static_block(commands: &mut Commands, x: i32, y: i32, color: Color) {
    let (screen_x, screen_y) = grid_to_screen(x as f32, y as f32);

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

fn spawn_particle_sprite(commands: &mut Commands, x: f32, y: f32, color: Color, z: f32, idx: usize) {
    let (screen_x, screen_y) = grid_to_screen(x, y);

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(CELL_SIZE - 1.0, CELL_SIZE - 1.0)),
                ..default()
            },
            transform: Transform::from_xyz(screen_x, screen_y, z),
            ..default()
        },
        ParticleSprite(idx),
    ));
}

fn find_blocked_blocks(particles: &[SimParticle], world: &SimWorld) -> HashSet<i32> {
    let mut blocked = HashSet::new();

    for p in particles {
        let block_id = match p.block_id {
            Some(id) => id,
            None => continue,
        };

        let x = p.position[0] as i32;
        let y = p.position[1] as i32;

        // Am Boden?
        if y <= 0 {
            blocked.insert(block_id);
            continue;
        }

        // Etwas darunter?
        if world.give_occupation_on_position(x as usize, (y - 1) as usize) {
            let same_block = particles.iter().any(|other| {
                other.block_id == Some(block_id)
                    && other.position[0] as i32 == x
                    && other.position[1] as i32 == y - 1
            });
            if !same_block {
                blocked.insert(block_id);
            }
        }
    }

    blocked
}

// === MAIN ===

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "World Simulation".into(),
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
        .insert_resource(Timers {
            sim: Timer::from_seconds(0.05, TimerMode::Repeating),
            spawn: Timer::from_seconds(0.08, TimerMode::Repeating),
        })
        .insert_resource(ParticleCounter(0))
        .insert_resource(BlockCounter(0))
        .add_systems(Startup, setup)
        .add_systems(Update, (spawn_particles, spawn_heavy_block, run_simulation, update_sprites))
        .run();
}

// === SETUP ===

fn setup(mut commands: Commands, mut sim: ResMut<Simulation>) {
    commands.spawn(Camera2dBundle::default());

    // Boden
    for x in 0..GRID_WIDTH {
        sim.world.grid[0][x] = (true, 1000.0, 0.0);
        spawn_static_block(&mut commands, x as i32, 0, Color::GRAY);
    }

    // Klotz in der Mitte
    let klotz_x = GRID_WIDTH as i32 / 2 - 3;
    for dx in 0..6 {
        for dy in 1..5 {
            let x = (klotz_x + dx) as usize;
            let y = dy as usize;
            sim.world.grid[y][x] = (true, 1000.0, 0.0);
            spawn_static_block(&mut commands, klotz_x + dx, dy, Color::DARK_GRAY);
        }
    }
}

// === SPAWN SYSTEMS ===

fn spawn_particles(
    mut commands: Commands,
    mut sim: ResMut<Simulation>,
    mut timers: ResMut<Timers>,
    mut counter: ResMut<ParticleCounter>,
    time: Res<Time>,
) {
    timers.spawn.tick(time.delta());

    if !timers.spawn.just_finished() || counter.0 >= 300 {
        return;
    }

    let spawn_x = GRID_WIDTH as i32 / 2 + (rand::random::<i32>() % 3) - 1;
    let spawn_y = (GRID_HEIGHT - 2) as i32;

    if sim.world.give_occupation_on_position(spawn_x as usize, spawn_y as usize) {
        return;
    }

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

    let sand_color = Color::rgb(0.9, 0.75, 0.4);
    spawn_particle_sprite(&mut commands, spawn_x as f32, spawn_y as f32, sand_color, 1.0, idx);
}

fn spawn_heavy_block(
    mut commands: Commands,
    mut sim: ResMut<Simulation>,
    mut counter: ResMut<ParticleCounter>,
    mut block_counter: ResMut<BlockCounter>,
    mouse_button: Res<Input<MouseButton>>,
    windows: Query<&Window>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let window = windows.single();
    let cursor_pos = match window.cursor_position() {
        Some(pos) => pos,
        None => return,
    };

    let grid_x = (cursor_pos.x / CELL_SIZE) as i32;
    let grid_y = (GRID_HEIGHT as f32 - cursor_pos.y / CELL_SIZE) as i32;

    block_counter.0 += 1;
    let current_block_id = block_counter.0;

    let block_color = Color::rgb(0.4, 0.2, 0.1);

    for dx in -1..=1 {
        for dy in -1..=1 {
            let x = grid_x + dx;
            let y = grid_y + dy;

            // Grenzen prüfen
            if x < 0 || x >= GRID_WIDTH as i32 || y < 0 || y >= GRID_HEIGHT as i32 {
                continue;
            }

            // Schon belegt?
            if sim.world.give_occupation_on_position(x as usize, y as usize) {
                continue;
            }

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

            spawn_particle_sprite(&mut commands, x as f32, y as f32, block_color, 2.0, idx);
        }
    }

    println!("Block {} bei ({}, {}) gespawnt!", current_block_id, grid_x, grid_y);
}

// === SIMULATION ===

fn run_simulation(
    mut sim: ResMut<Simulation>,
    mut timers: ResMut<Timers>,
    time: Res<Time>,
) {
    timers.sim.tick(time.delta());

    if !timers.sim.just_finished() {
        return;
    }

    sim.world.calc_pressure_on_all_position();

    let blocked = find_blocked_blocks(&sim.particles, &sim.world);
    let gravity = sim.gravity;

    // Hier destructuren!
    let Simulation { world, particles, .. } = &mut *sim;

    // Velocity + Position updaten
    for p in particles.iter_mut() {
        p.update_velocity(gravity, world);
        p.update_position(world);
    }

    // Pressure resolven
    for p in particles.iter_mut() {
        p.resolve_pressure(world);
    }

    // Fallen lassen
    for p in particles.iter_mut() {
        let is_blocked = p.block_id.map(|id| blocked.contains(&id)).unwrap_or(false);
        if !is_blocked {
            p.fall_down(world);
        }
    }
}

// === RENDERING ===

fn update_sprites(
    sim: Res<Simulation>,
    mut query: Query<(&ParticleSprite, &mut Transform)>,
) {
    for (particle_sprite, mut transform) in query.iter_mut() {
        let particle = &sim.particles[particle_sprite.0];
        let (screen_x, screen_y) = grid_to_screen(particle.position[0], particle.position[1]);
        transform.translation.x = screen_x;
        transform.translation.y = screen_y;
    }
}