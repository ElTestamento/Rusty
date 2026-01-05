use bevy::prelude::*;
use world::{Particle as SimParticle, Object as SimObject, World as SimWorld};

const GRID_WIDTH: usize = 40;
const GRID_HEIGHT: usize = 30;
const CELL_SIZE: f32 = 20.0;

// === KOMPONENTEN & RESSOURCEN ===

#[derive(Component)]
struct ParticleSprite(usize);

#[derive(Component)]
struct ObjectSprite {
    object_idx: usize,
    grid_i: usize,
    grid_j: usize,
}

#[derive(Resource)]
struct Simulation {
    world: SimWorld,
    particles: Vec<SimParticle>,
    objects: Vec<SimObject>,
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
struct ObjectCounter(i32);

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
            objects: Vec::new(),
            gravity: [0.0, -1.0],
        })
        .insert_resource(Timers {
            sim: Timer::from_seconds(0.05, TimerMode::Repeating),
            spawn: Timer::from_seconds(0.08, TimerMode::Repeating),
        })
        .insert_resource(ParticleCounter(0))
        .insert_resource(ObjectCounter(0))
        .add_systems(Startup, setup)
        .add_systems(Update, (spawn_particles, spawn_object, run_simulation, update_sprites, update_object_sprites))
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

fn spawn_object(
    mut commands: Commands,
    mut sim: ResMut<Simulation>,
    mut object_counter: ResMut<ObjectCounter>,
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

    // Grenzen prüfen
    if grid_x < 0 || grid_x >= GRID_WIDTH as i32 - 2 || grid_y < 0 || grid_y >= GRID_HEIGHT as i32 - 2 {
        return;
    }

    // Prüfen ob Platz frei ist (3x3)
    for di in 0..3 {
        for dj in 0..3 {
            if sim.world.give_occupation_on_position((grid_x + dj) as usize, (grid_y + di) as usize) {
                return;
            }
        }
    }

    object_counter.0 += 1;
    let obj_id = object_counter.0;

    // Object erstellen (3x3, Masse 90 = 10 pro Partikel)
    let object = SimObject::new(
        obj_id,
        [grid_x as f32, grid_y as f32],
        [0.0, 0.0],
        90.0,
        3,
        3,
    );

    // World updaten für alle Partikel des Objects
    for particle in object.get_object_elements() {
        sim.world.update_occupation_on_position(particle.position);
        sim.world.update_mass_on_position(particle.position, particle.mass);
    }

    let obj_idx = sim.objects.len();
    sim.objects.push(object);

    // Sprites für jedes Partikel im Object spawnen
    let block_color = Color::rgb(0.4, 0.2, 0.1);
    for i in 0..3 {
        for j in 0..3 {
            let px = grid_x as f32 + j as f32;
            let py = grid_y as f32 + i as f32;
            let (screen_x, screen_y) = grid_to_screen(px, py);

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: block_color,
                        custom_size: Some(Vec2::new(CELL_SIZE - 1.0, CELL_SIZE - 1.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(screen_x, screen_y, 2.0),
                    ..default()
                },
                ObjectSprite {
                    object_idx: obj_idx,
                    grid_i: i,
                    grid_j: j,
                },
            ));
        }
    }

    println!("Object {} bei ({}, {}) gespawnt!", obj_id, grid_x, grid_y);
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

    let gravity = sim.gravity;

    // Particles updaten
    let Simulation { world, particles, .. } = &mut *sim;

    for p in particles.iter_mut() {
        p.update_velocity(gravity, world);
        p.update_position(world);
    }

    for p in particles.iter_mut() {
        p.resolve_pressure(world);
    }

    for p in particles.iter_mut() {
        p.fall_down(world);
    }

    // Objects updaten
    let Simulation { world, objects, .. } = &mut *sim;

    for obj in objects.iter_mut() {
        obj.update_object_velocity(gravity, world);
        obj.update_object_position(world);
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

fn update_object_sprites(
    sim: Res<Simulation>,
    mut query: Query<(&ObjectSprite, &mut Transform)>,
) {
    for (obj_sprite, mut transform) in query.iter_mut() {
        if obj_sprite.object_idx >= sim.objects.len() {
            continue;
        }
        let object = &sim.objects[obj_sprite.object_idx];
        let particle = &object.get_object_elements()[obj_sprite.grid_i * 3 + obj_sprite.grid_j];
        let (screen_x, screen_y) = grid_to_screen(particle.position[0], particle.position[1]);
        transform.translation.x = screen_x;
        transform.translation.y = screen_y;
    }
}