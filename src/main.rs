// === BEVY IMPORT ===
// bevy::prelude::* importiert alle häufig genutzten Bevy-Typen:
// - App, Commands, Query, Res, ResMut (Systemparameter)
// - Component, Resource (Makros für eigene Typen)
// - Transform, Sprite, SpriteBundle (Rendering)
// - Timer, Time (Zeitsteuerung)
// - Color, Vec2 (Mathematik/Grafik)
use bevy::prelude::*;

// Import aus unserer Physik-Bibliothek (lib.rs)
// "as SimParticle" etc. sind Aliase um Namenskonflikte mit Bevy zu vermeiden
use world::{Particle as SimParticle, Object as SimObject, World as SimWorld, MaterialTyp};

// === KONSTANTEN ===
const GRID_WIDTH: usize = 40;
const GRID_HEIGHT: usize = 30;
const CELL_SIZE: f32 = 20.0;

// === KOMPONENTEN ===
// #[derive(Component)] macht einen Struct zu einer Bevy-Komponente.
// Komponenten sind Daten die an Entities (Spielobjekte) gehängt werden.
// Bevy nutzt ein "Entity Component System" (ECS):
// - Entity = ID (z.B. "Sprite #42")
// - Component = Daten (z.B. Position, Farbe)
// - System = Logik die auf Komponenten arbeitet

// Diese Komponente verknüpft ein Sprite mit einem Partikel-Index
#[derive(Component)]
struct ParticleSprite(usize);

// Diese Komponente verknüpft ein Sprite mit einem Object und seiner Grid-Position
#[derive(Component)]
struct ObjectSprite {
    object_idx: usize,
    grid_i: usize,
    grid_j: usize,
}

// === RESSOURCEN ===
// #[derive(Resource)] macht einen Struct zu einer Bevy-Ressource.
// Ressourcen sind globale Daten die es nur einmal gibt (Singleton).
// Zugriff in Systemen via Res<T> (readonly) oder ResMut<T> (mutable).

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

/// Wandelt Grid-Koordinaten in Bildschirm-Koordinaten um.
/// Bevy hat den Ursprung (0,0) in der Bildschirmmitte,
/// unser Grid hat (0,0) unten links.
fn grid_to_screen(x: f32, y: f32) -> (f32, f32) {
    let screen_x = (x - GRID_WIDTH as f32 / 2.0 + 0.5) * CELL_SIZE;
    let screen_y = (y - GRID_HEIGHT as f32 / 2.0 + 0.5) * CELL_SIZE;
    (screen_x, screen_y)
}

/// Hilfsfunktion: Wandelt unser (r,g,b) Tuple in Bevy's Color um.
/// So bleibt lib.rs unabhängig von Bevy.
fn material_to_color(material: MaterialTyp) -> Color {
    let (r, g, b) = material.color();
    Color::rgb(r, g, b)
}

/// Spawnt einen statischen Block (Boden, Hindernisse).
/// commands.spawn() erstellt eine neue Entity mit den angegebenen Komponenten.
/// SpriteBundle ist ein vordefiniertes Bundle das alles für ein 2D-Sprite enthält.
fn spawn_static_block(commands: &mut Commands, x: i32, y: i32, color: Color) {
    let (screen_x, screen_y) = grid_to_screen(x as f32, y as f32);

    // spawn() erstellt eine Entity
    // SpriteBundle enthält: Sprite, Transform, GlobalTransform, Texture, Visibility
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color,
            custom_size: Some(Vec2::new(CELL_SIZE - 1.0, CELL_SIZE - 1.0)),
            ..default()  // Restliche Felder mit Standardwerten füllen
        },
        transform: Transform::from_xyz(screen_x, screen_y, 0.0),
        ..default()
    });
}

/// Spawnt ein Partikel-Sprite und verknüpft es mit dem Partikel-Index.
/// Das Tuple (SpriteBundle, ParticleSprite) fügt beide Komponenten zur Entity hinzu.
fn spawn_particle_sprite(commands: &mut Commands, x: f32, y: f32, color: Color, z: f32, idx: usize) {
    let (screen_x, screen_y) = grid_to_screen(x, y);

    // Tuple-Syntax: (KomponenteA, KomponenteB) fügt mehrere Komponenten hinzu
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
        ParticleSprite(idx),  // Unsere eigene Komponente für die Verknüpfung
    ));
}

// === MAIN ===
fn main() {
    // App::new() erstellt eine neue Bevy-Anwendung
    // Methoden werden verkettet (Builder Pattern)
    App::new()
        // DefaultPlugins: Fenster, Rendering, Input, etc.
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "World Simulation".into(),
                resolution: (GRID_WIDTH as f32 * CELL_SIZE, GRID_HEIGHT as f32 * CELL_SIZE).into(),
                ..default()
            }),
            ..default()
        }))
        // insert_resource: Fügt eine Ressource hinzu (globaler State)
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
        // add_systems(Startup, ...): Läuft einmal beim Start
        .add_systems(Startup, setup)
        // add_systems(Update, ...): Läuft jeden Frame
        .add_systems(Update, (spawn_particles, spawn_object, run_simulation, update_sprites, update_object_sprites))
        // run() startet die Gameloop
        .run();
}

// === SETUP ===
// Bevy-Systeme sind normale Funktionen.
// Die Parameter werden automatisch von Bevy "injected":
// - Commands: Zum Erstellen/Löschen von Entities
// - ResMut<T>: Mutable Zugriff auf Ressource T
fn setup(mut commands: Commands, mut sim: ResMut<Simulation>) {
    // Kamera erstellen - ohne Kamera sieht man nichts!
    commands.spawn(Camera2dBundle::default());

    // Boden erstellen
    for x in 0..GRID_WIDTH {
        sim.world.grid[0][x] = (true, 1000.0, 0.0);
        spawn_static_block(&mut commands, x as i32, 0, Color::GRAY);
    }

    // Statischer Klotz in der Mitte (Hindernis)
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
    time: Res<Time>,  // Res<Time> ist Bevy's Zeitressource (readonly)
) {
    // Timer ticken lassen - time.delta() ist die Zeit seit letztem Frame
    timers.spawn.tick(time.delta());

    // Nur spawnen wenn Timer abgelaufen UND Limit nicht erreicht
    if !timers.spawn.just_finished() || counter.0 >= 300 {
        return;
    }

    // Zufällige X-Position nahe der Mitte
    let spawn_x = GRID_WIDTH as i32 / 2 + (rand::random::<i32>() % 3) - 1;
    let spawn_y = (GRID_HEIGHT - 2) as i32;

    // Nicht spawnen wenn Position belegt
    if sim.world.give_occupation_on_position(spawn_x as usize, spawn_y as usize) {
        return;
    }

    counter.0 += 1;

    // === MATERIAL DEFINIEREN ===
    // Hier wird das Material festgelegt - später durch UI ersetzbar
    let material = MaterialTyp::Sand;

    // Partikel erstellen - Material bestimmt automatisch die Masse
    let particle = SimParticle::new(
        counter.0,
        [spawn_x as f32, spawn_y as f32],
        [0.0, 0.0],
        material,
    );

    // World updaten
    sim.world.update_occupation_on_position(particle.position);
    sim.world.update_mass_on_position(particle.position, particle.mass());

    let idx = sim.particles.len();
    sim.particles.push(particle);

    // Farbe aus Material ableiten - nicht hardcoded!
    let color = material_to_color(material);
    spawn_particle_sprite(&mut commands, spawn_x as f32, spawn_y as f32, color, 1.0, idx);
}

fn spawn_object(
    mut commands: Commands,
    mut sim: ResMut<Simulation>,
    mut object_counter: ResMut<ObjectCounter>,
    mouse_button: Res<Input<MouseButton>>,  // Bevy's Input-System
    windows: Query<&Window>,  // Query holt alle Entities mit Window-Komponente
) {
    // Nur bei Linksklick
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    // windows.single() holt das einzige Window (wir haben nur eins)
    let window = windows.single();

    // Cursor-Position holen (kann None sein wenn Maus außerhalb)
    let cursor_pos = match window.cursor_position() {
        Some(pos) => pos,
        None => return,
    };

    // Bildschirm-Koordinaten zu Grid-Koordinaten
    let grid_x = (cursor_pos.x / CELL_SIZE) as i32;
    let grid_y = (GRID_HEIGHT as f32 - cursor_pos.y / CELL_SIZE) as i32;

    // Grenzen prüfen (3x3 Block muss reinpassen)
    if grid_x < 0 || grid_x >= GRID_WIDTH as i32 - 2 || grid_y < 0 || grid_y >= GRID_HEIGHT as i32 - 2 {
        return;
    }

    // Prüfen ob Platz frei ist (3x3 Bereich)
    for di in 0..3 {
        for dj in 0..3 {
            if sim.world.give_occupation_on_position((grid_x + dj) as usize, (grid_y + di) as usize) {
                return;
            }
        }
    }

    object_counter.0 += 1;
    let obj_id = object_counter.0;

    // === MATERIAL DEFINIEREN ===
    // Hier wird das Material festgelegt - später durch UI ersetzbar
    // Probiere verschiedene: MaterialTyp::Stein, MaterialTyp::Metall, MaterialTyp::Holz
    let material = MaterialTyp::Metall;

    // Object erstellen - Material bestimmt automatisch die Masse
    let object = SimObject::new(
        obj_id,
        [grid_x as f32, grid_y as f32],
        [0.0, 0.0],
        material,
        3,
        3,
    );

    // World updaten für alle Partikel des Objects
    for particle in object.get_object_elements() {
        sim.world.update_occupation_on_position(particle.position);
        sim.world.update_mass_on_position(particle.position, particle.mass());
    }

    let obj_idx = sim.objects.len();
    sim.objects.push(object);

    // Farbe aus Material ableiten - nicht hardcoded!
    let color = material_to_color(material);

    // Sprites für jedes Partikel im Object spawnen
    for i in 0..3 {
        for j in 0..3 {
            let px = grid_x as f32 + j as f32;
            let py = grid_y as f32 + i as f32;
            let (screen_x, screen_y) = grid_to_screen(px, py);

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color,
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

    println!("Object {} ({:?}) bei ({}, {}) gespawnt!", obj_id, material, grid_x, grid_y);
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

    // Druck im gesamten Grid neu berechnen
    sim.world.calc_pressure_on_all_position();

    let gravity = sim.gravity;

    // Rust Borrowing: Wir brauchen mutable Zugriff auf particles UND world
    // Destructuring erlaubt das gleichzeitig
    let Simulation { world, particles, .. } = &mut *sim;

    // Partikel-Physik: Velocity, Position, Druck, Fallen
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

    // Object-Physik
    let Simulation { world, objects, .. } = &mut *sim;

    for obj in objects.iter_mut() {
        obj.update_object_velocity(gravity, world);
        obj.update_object_position(world);
    }
}

// === RENDERING ===

/// Aktualisiert Sprite-Positionen basierend auf Partikel-Positionen.
/// Query<(A, B)> holt alle Entities die BEIDE Komponenten haben.
fn update_sprites(
    sim: Res<Simulation>,
    mut query: Query<(&ParticleSprite, &mut Transform)>,
) {
    // iter_mut() iteriert über alle passenden Entities
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
        // Sicherheitscheck: Object existiert noch?
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