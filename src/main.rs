// === BEVY IMPORT ===
use bevy::prelude::*;

// Import aus unserer Physik-Bibliothek (lib.rs)
use world::{Particle as SimParticle, Object as SimObject, World as SimWorld, MaterialTyp, ParticleRef};

// === KONSTANTEN ===
const GRID_WIDTH: usize = 120;     // Breiter
const GRID_HEIGHT: usize = 100;    // Höher (mehr Platz für Türme!)
const CELL_SIZE: f32 = 8.0;        // Noch feiner
const WINDOW_WIDTH: f32 = 960.0;   // Fenster-Breite
const WINDOW_HEIGHT: f32 = 800.0;  // Fenster-Höhe
const CAMERA_SPEED: f32 = 400.0;   // Schneller

// === KOMPONENTEN ===

#[derive(Component)]
struct ParticleSprite(usize);

#[derive(Component)]
struct ObjectSprite {
    object_idx: usize,
    grid_i: usize,
    grid_j: usize,
}

#[derive(Component)]
struct DebugLabel;

#[derive(Component)]
struct MaterialLabel;

#[derive(Component)]
struct MainCamera;

// === RESSOURCEN ===

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

/// Event für Fragment-Verarbeitung.
/// Enthält: Object-Index und die Fragmente (Liste von Grid-Koordinaten)
struct FragmentEvent {
    object_idx: usize,
    fragments: Vec<Vec<(usize, usize)>>,
}

/// Resource die Fragment-Events sammelt.
/// Wird in run_simulation gefüllt und in handle_fragments verarbeitet.
#[derive(Resource, Default)]
struct FragmentEvents {
    events: Vec<FragmentEvent>,
}

/// Aktuell ausgewähltes Material für Spawning.
/// Tasten 1-6 wechseln das Material.
#[derive(Resource)]
struct SelectedMaterial(MaterialTyp);

impl Default for SelectedMaterial {
    fn default() -> Self {
        SelectedMaterial(MaterialTyp::Sand)
    }
}

// === HELPER FUNKTIONEN ===

fn grid_to_screen(x: f32, y: f32) -> (f32, f32) {
    let screen_x = (x - GRID_WIDTH as f32 / 2.0 + 0.5) * CELL_SIZE;
    let screen_y = (y - GRID_HEIGHT as f32 / 2.0 + 0.5) * CELL_SIZE;
    (screen_x, screen_y)
}

fn material_to_color(material: MaterialTyp) -> Color {
    let (r, g, b) = material.color();
    Color::rgb(r, g, b)
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
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
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
        .insert_resource(FragmentEvents::default())
        .insert_resource(SelectedMaterial::default())
        .add_systems(Startup, setup)
        .add_systems(Update, camera_movement)  // Kamera separat (immer aktiv)
        .add_systems(Update, (
            change_material,
            spawn_particles,
            spawn_object,
            run_simulation,
            handle_fragments,
            update_sprites,
            update_object_sprites,
            update_debug_label,
            update_material_label,
        ).chain())
        .run();
}

// === SETUP ===
fn setup(mut commands: Commands, mut sim: ResMut<Simulation>) {
    // Kamera mit MainCamera Component
    commands.spawn((
        Camera2dBundle::default(),
        MainCamera,
    ));

    // Boden erstellen
    for x in 0..GRID_WIDTH {
        sim.world.grid[0][x] = (Some(ParticleRef::Static), 1000.0, 0.0);
        spawn_static_block(&mut commands, x as i32, 0, Color::GRAY);
    }

    // Statischer Klotz in der Mitte (etwas größer für das größere Grid)
    let klotz_x = GRID_WIDTH as i32 / 2 - 5;
    for dx in 0..10 {
        for dy in 1..8 {
            let x = (klotz_x + dx) as usize;
            let y = dy as usize;
            sim.world.grid[y][x] = (Some(ParticleRef::Static), 1000.0, 0.0);
            spawn_static_block(&mut commands, klotz_x + dx, dy, Color::DARK_GRAY);
        }
    }

    // Debug-Label erstellen
    commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 16.0,
                color: Color::WHITE,
                ..default()
            },
        )
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            }),
        DebugLabel,
    ));

    // Material-Label erstellen (oben rechts)
    commands.spawn((
        TextBundle::from_section(
            "Material: Sand [1]\n\n1=Sand 2=Stein 3=Metall\n4=Holz 5=Wasser\nShift+Klick=Quadrant-Block",
            TextStyle {
                font_size: 14.0,
                color: Color::YELLOW,
                ..default()
            },
        )
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                ..default()
            }),
        MaterialLabel,
    ));
}

// === KAMERA STEUERUNG ===

/// Kamera-Bewegung mit WASD oder Pfeiltasten
fn camera_movement(
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    let mut camera_transform = camera_query.single_mut();

    let mut direction = Vec3::ZERO;

    // WASD oder Pfeiltasten
    if keyboard.pressed(KeyCode::W) || keyboard.pressed(KeyCode::Up) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::S) || keyboard.pressed(KeyCode::Down) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::A) || keyboard.pressed(KeyCode::Left) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::D) || keyboard.pressed(KeyCode::Right) {
        direction.x += 1.0;
    }

    // Normalisieren und bewegen
    if direction != Vec3::ZERO {
        direction = direction.normalize();
        camera_transform.translation += direction * CAMERA_SPEED * time.delta_seconds();
    }
}

// === MATERIAL AUSWAHL ===

/// Tastatur-System für Material-Auswahl
/// 1 = Sand, 2 = Stein, 3 = Metall, 4 = Holz, 5 = Wasser
fn change_material(
    keyboard: Res<Input<KeyCode>>,
    mut selected: ResMut<SelectedMaterial>,
) {
    if keyboard.just_pressed(KeyCode::Key1) {
        selected.0 = MaterialTyp::Sand;
    } else if keyboard.just_pressed(KeyCode::Key2) {
        selected.0 = MaterialTyp::Stein;
    } else if keyboard.just_pressed(KeyCode::Key3) {
        selected.0 = MaterialTyp::Metall;
    } else if keyboard.just_pressed(KeyCode::Key4) {
        selected.0 = MaterialTyp::Holz;
    } else if keyboard.just_pressed(KeyCode::Key5) {
        selected.0 = MaterialTyp::Wasser;
    }
}

/// Update Material-Label
fn update_material_label(
    selected: Res<SelectedMaterial>,
    mut query: Query<&mut Text, With<MaterialLabel>>,
) {
    let mut text = query.single_mut();
    let mat_name = match selected.0 {
        MaterialTyp::Sand => "Sand [1]",
        MaterialTyp::Stein => "Stein [2]",
        MaterialTyp::Metall => "Metall [3]",
        MaterialTyp::Holz => "Holz [4]",
        MaterialTyp::Wasser => "Wasser [5]",
        MaterialTyp::Luft => "Luft",
    };
    text.sections[0].value = format!(
        "Material: {}\n\n1-5=Material\nShift+Klick=Quadrant\nWASD=Kamera",
        mat_name
    );
}

// === SPAWN SYSTEMS ===

fn spawn_particles(
    mut commands: Commands,
    mut sim: ResMut<Simulation>,
    mut timers: ResMut<Timers>,
    mut counter: ResMut<ParticleCounter>,
    selected: Res<SelectedMaterial>,
    time: Res<Time>,
) {
    timers.spawn.tick(time.delta());

    if !timers.spawn.just_finished() || counter.0 >= 500 {  // Erhöht von 300
        return;
    }

    let spawn_x = GRID_WIDTH as i32 / 2 + (rand::random::<i32>() % 5) - 2;  // Breiter streuen
    let spawn_y = (GRID_HEIGHT - 2) as i32;

    if sim.world.give_occupation_on_position(spawn_x as usize, spawn_y as usize).is_some() {
        return;
    }

    counter.0 += 1;

    let material = selected.0;  // NEU: Gewähltes Material verwenden
    let idx = sim.particles.len();

    let particle = SimParticle::new(
        counter.0,
        [spawn_x as f32, spawn_y as f32],
        [0.0, 0.0],
        material,
        ParticleRef::Free(idx),
    );

    sim.world.update_occupation_on_position(particle.position, particle.particle_ref);
    sim.world.update_mass_on_position(particle.position, particle.mass());

    sim.particles.push(particle);

    let color = material_to_color(material);
    spawn_particle_sprite(&mut commands, spawn_x as f32, spawn_y as f32, color, 1.0, idx);
}

fn spawn_object(
    mut commands: Commands,
    mut sim: ResMut<Simulation>,
    mut object_counter: ResMut<ObjectCounter>,
    mouse_button: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    selected: Res<SelectedMaterial>,
    windows: Query<&Window>,
    camera_query: Query<&Transform, With<MainCamera>>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let window = windows.single();
    let camera_transform = camera_query.single();

    let cursor_pos = match window.cursor_position() {
        Some(pos) => pos,
        None => return,
    };

    // Maus-Position in Welt-Koordinaten umrechnen (mit Kamera-Offset)
    // cursor_pos ist relativ zum Fenster (0,0 = oben links)
    // Wir müssen: Fenster-Mitte als Referenz + Kamera-Offset
    let world_x = cursor_pos.x - WINDOW_WIDTH / 2.0 + camera_transform.translation.x;
    let world_y = WINDOW_HEIGHT / 2.0 - cursor_pos.y + camera_transform.translation.y;

    // Welt-Koordinaten zu Grid-Koordinaten
    let grid_x = (world_x / CELL_SIZE + GRID_WIDTH as f32 / 2.0) as i32;
    let grid_y = (world_y / CELL_SIZE + GRID_HEIGHT as f32 / 2.0) as i32;

    // Shift gedrückt? → Quadrant-Block (4x4), sonst normaler Block (3x3)
    let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let block_size = if shift_held { 4 } else { 3 };

    // Bounds check
    if grid_x < 0 || grid_x >= GRID_WIDTH as i32 - (block_size - 1) || grid_y < 0 || grid_y >= GRID_HEIGHT as i32 - (block_size - 1) {
        return;
    }

    // Occupation check
    for di in 0..block_size {
        for dj in 0..block_size {
            if sim.world.give_occupation_on_position((grid_x + dj) as usize, (grid_y + di) as usize).is_some() {
                return;
            }
        }
    }

    object_counter.0 += 1;
    let obj_id = object_counter.0;
    let obj_idx = sim.objects.len();

    if shift_held {
        // Shift+Klick: Quadrant-Block (4x4 gemischt)
        let object = SimObject::new_quadrant(
            obj_id,
            obj_idx,
            [grid_x as f32, grid_y as f32],
            [0.0, 0.0],
        );

        for particle in object.get_object_elements() {
            sim.world.update_occupation_on_position(particle.position, particle.particle_ref);
            sim.world.update_mass_on_position(particle.position, particle.mass());
        }

        // Sprites spawnen
        for i in 0..4 {
            for j in 0..4 {
                let particle = object.get_particle_at(i, j);
                let color = material_to_color(particle.material);

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

        sim.objects.push(object);
    } else {
        // Normaler Klick: 3x3 Block aus gewähltem Material
        let material = selected.0;
        let object = SimObject::new(
            obj_id,
            obj_idx,
            [grid_x as f32, grid_y as f32],
            [0.0, 0.0],
            material,
            3,
            3,
        );

        for particle in object.get_object_elements() {
            sim.world.update_occupation_on_position(particle.position, particle.particle_ref);
            sim.world.update_mass_on_position(particle.position, particle.mass());
        }

        // Sprites spawnen
        let color = material_to_color(material);
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

        sim.objects.push(object);
    }
}

// === SIMULATION ===

fn run_simulation(
    mut sim: ResMut<Simulation>,
    mut timers: ResMut<Timers>,
    mut fragment_events: ResMut<FragmentEvents>,
    time: Res<Time>,
) {
    timers.sim.tick(time.delta());

    if !timers.sim.just_finished() {
        return;
    }

    sim.world.calc_pressure_on_all_position();

    let gravity = sim.gravity;

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

    // Object-Physik mit Fragment-Detection
    let Simulation { world, objects, .. } = &mut *sim;

    for (obj_idx, obj) in objects.iter_mut().enumerate() {
        // Überspringe zerstörte Objects
        if obj.is_destroyed {
            continue;
        }

        // update_object_velocity gibt Some(fragments) zurück wenn Impact-Bruch passiert
        if let Some(fragments) = obj.update_object_velocity(gravity, world) {
            // Event zur späteren Verarbeitung speichern
            fragment_events.events.push(FragmentEvent {
                object_idx: obj_idx,
                fragments,
            });
            continue; // Nicht weiter prüfen wenn schon gebrochen
        }

        // Nur Position updaten wenn nicht zerstört
        if !obj.is_destroyed {
            obj.update_object_position(world);
        }
    }

    // DRUCK-PRÜFUNG: Nur für stehende Objects (nach Position-Update)
    let Simulation { world, objects, .. } = &mut *sim;

    for (obj_idx, obj) in objects.iter_mut().enumerate() {
        // Überspringe zerstörte oder bewegte Objects
        if obj.is_destroyed {
            continue;
        }

        // Nur prüfen wenn Object still steht (nicht in Bewegung)
        let vel = obj.get_object_velocity();
        if vel[1] != 0.0 {
            continue;
        }

        // Druck-Bruch prüfen
        let broken_bonds = obj.check_pressure_fracture(world);

        if !broken_bonds.is_empty() {
            // Fragmente finden und Event speichern
            let fragments = obj.find_fragments(&broken_bonds);

            // Nur wenn mehr als 1 Fragment (sonst kein echter Bruch)
            if fragments.len() > 1 {
                fragment_events.events.push(FragmentEvent {
                    object_idx: obj_idx,
                    fragments,
                });
            }
        }
    }
}

// === FRAGMENT HANDLING ===

/// Verarbeitet Fragment-Events: Zerstört alte Objects, erstellt neue Partikel/Objects.
fn handle_fragments(
    mut commands: Commands,
    mut sim: ResMut<Simulation>,
    mut fragment_events: ResMut<FragmentEvents>,
    mut counter: ResMut<ParticleCounter>,
    mut object_counter: ResMut<ObjectCounter>,
    object_sprites: Query<(Entity, &ObjectSprite)>,
) {
    // Keine Events? Nichts zu tun.
    if fragment_events.events.is_empty() {
        return;
    }

    // Events verarbeiten
    for event in fragment_events.events.drain(..) {
        let obj_idx = event.object_idx;

        // Object existiert noch?
        if obj_idx >= sim.objects.len() || sim.objects[obj_idx].is_destroyed {
            continue;
        }

        // Velocity vom alten Object übernehmen
        let old_velocity = sim.objects[obj_idx].get_object_velocity();

        // Partikel-Daten VOR dem Zerstören extrahieren
        let fragment_data: Vec<Vec<([f32; 2], MaterialTyp)>> = event.fragments.iter()
            .map(|frag| sim.objects[obj_idx].extract_fragment_data(frag))
            .collect();

        // Object aus World-Grid entfernen
        let Simulation { world, objects, .. } = &mut *sim;
        objects[obj_idx].clear_from_world(world);
        objects[obj_idx].is_destroyed = true;

        // Alte ObjectSprites für dieses Object despawnen
        for (entity, sprite) in object_sprites.iter() {
            if sprite.object_idx == obj_idx {
                commands.entity(entity).despawn();
            }
        }

        // Für jedes Fragment: Neues Partikel oder Object erstellen
        for frag_data in fragment_data {
            if frag_data.len() == 1 {
                // Ein einzelnes Partikel → freies Partikel erstellen
                let (pos, material) = frag_data[0];
                counter.0 += 1;
                let idx = sim.particles.len();

                let particle = SimParticle::new(
                    counter.0,
                    pos,
                    [0.0, 0.0],
                    material,
                    ParticleRef::Free(idx),
                );

                sim.world.update_occupation_on_position(particle.position, particle.particle_ref);
                sim.world.update_mass_on_position(particle.position, particle.mass());
                sim.particles.push(particle);

                // Sprite spawnen
                let color = material_to_color(material);
                spawn_particle_sprite(&mut commands, pos[0], pos[1], color, 1.0, idx);
            } else {
                // Mehrere Partikel → neues Object erstellen!
                object_counter.0 += 1;
                let new_obj_id = object_counter.0;
                let new_obj_idx = sim.objects.len();

                let new_object = SimObject::new_from_fragment(
                    new_obj_id,
                    new_obj_idx,
                    &frag_data,
                    old_velocity,
                );

                // Neue Partikel im World-Grid registrieren
                for particle in new_object.get_object_elements() {
                    // Luft-Partikel nicht registrieren (Löcher im Fragment)
                    if particle.material != MaterialTyp::Luft {
                        sim.world.update_occupation_on_position(particle.position, particle.particle_ref);
                        sim.world.update_mass_on_position(particle.position, particle.mass());
                    }
                }

                // Sprites für neues Object spawnen
                let h = new_object.get_height();
                let w = new_object.get_width();
                for i in 0..h {
                    for j in 0..w {
                        let particle = new_object.get_particle_at(i, j);
                        // Keine Sprites für Luft (Löcher)
                        if particle.material == MaterialTyp::Luft {
                            continue;
                        }
                        let color = material_to_color(particle.material);
                        let (screen_x, screen_y) = grid_to_screen(particle.position[0], particle.position[1]);

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
                                object_idx: new_obj_idx,
                                grid_i: i,
                                grid_j: j,
                            },
                        ));
                    }
                }

                sim.objects.push(new_object);
            }
        }
    }
}

// === RENDERING ===

fn update_sprites(
    sim: Res<Simulation>,
    mut query: Query<(&ParticleSprite, &mut Transform)>,
) {
    for (particle_sprite, mut transform) in query.iter_mut() {
        // Bounds check
        if particle_sprite.0 >= sim.particles.len() {
            continue;
        }
        let particle = &sim.particles[particle_sprite.0];
        let (screen_x, screen_y) = grid_to_screen(particle.position[0], particle.position[1]);
        transform.translation.x = screen_x;
        transform.translation.y = screen_y;
    }
}

fn update_object_sprites(
    sim: Res<Simulation>,
    mut query: Query<(&ObjectSprite, &mut Transform, &mut Visibility)>,
) {
    for (obj_sprite, mut transform, mut visibility) in query.iter_mut() {
        // Object existiert noch und ist nicht zerstört?
        if obj_sprite.object_idx >= sim.objects.len() {
            *visibility = Visibility::Hidden;
            continue;
        }

        let object = &sim.objects[obj_sprite.object_idx];

        // Zerstörte Objects ausblenden
        if object.is_destroyed {
            *visibility = Visibility::Hidden;
            continue;
        }

        // Variable Grid-Breite verwenden statt hardcoded 3
        let w = object.get_width();
        let particle = &object.get_object_elements()[obj_sprite.grid_i * w + obj_sprite.grid_j];
        let (screen_x, screen_y) = grid_to_screen(particle.position[0], particle.position[1]);
        transform.translation.x = screen_x;
        transform.translation.y = screen_y;
    }
}

// === DEBUG LABEL ===

fn update_debug_label(
    sim: Res<Simulation>,
    windows: Query<&Window>,
    camera_query: Query<&Transform, With<MainCamera>>,
    mut query: Query<&mut Text, With<DebugLabel>>,
) {
    let window = windows.single();
    let camera_transform = camera_query.single();
    let mut text = query.single_mut();

    let cursor_pos = match window.cursor_position() {
        Some(pos) => pos,
        None => {
            text.sections[0].value = "".to_string();
            return;
        }
    };

    // Maus-Position in Welt-Koordinaten (mit Kamera-Offset)
    let world_x = cursor_pos.x - WINDOW_WIDTH / 2.0 + camera_transform.translation.x;
    let world_y = WINDOW_HEIGHT / 2.0 - cursor_pos.y + camera_transform.translation.y;

    // Welt zu Grid
    let grid_x = ((world_x / CELL_SIZE + GRID_WIDTH as f32 / 2.0) as i32).max(0) as usize;
    let grid_y = ((world_y / CELL_SIZE + GRID_HEIGHT as f32 / 2.0) as i32).max(0) as usize;

    if grid_x >= GRID_WIDTH || grid_y >= GRID_HEIGHT {
        text.sections[0].value = "".to_string();
        return;
    }

    match sim.world.give_occupation_on_position(grid_x, grid_y) {
        Some(ParticleRef::Free(idx)) => {
            if idx < sim.particles.len() {
                let p = &sim.particles[idx];
                text.sections[0].value = format!(
                    "PARTIKEL #{}\nMaterial: {:?}\nVelocity: [{:.1}, {:.1}]",
                    idx, p.material, p.velocity[0], p.velocity[1]
                );
            } else {
                text.sections[0].value = format!("PARTIKEL #{} (ungültig)", idx);
            }
        }
        Some(ParticleRef::InObject(obj_idx, i, j)) => {
            if obj_idx < sim.objects.len() && !sim.objects[obj_idx].is_destroyed {
                let obj = &sim.objects[obj_idx];
                let vel = obj.get_object_velocity();
                // NEU: Material vom RICHTIGEN Partikel holen (i, j aus ParticleRef)
                let particle = obj.get_particle_at(i, j);
                let mat = particle.material;
                text.sections[0].value = format!(
                    "OBJECT #{}\nPos: ({},{})\nMaterial: {:?}\nVelocity: [{:.1}, {:.1}]",
                    obj_idx, i, j, mat, vel[0], vel[1]
                );
            } else {
                text.sections[0].value = format!("OBJECT #{} (zerstört)", obj_idx);
            }
        }
        Some(ParticleRef::Static) => {
            text.sections[0].value = "STATIC\n(Boden/Hindernis)".to_string();
        }
        None => {
            text.sections[0].value = format!("Leer\n[{}, {}]", grid_x, grid_y);
        }
    }
}