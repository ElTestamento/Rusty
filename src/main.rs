use bevy::prelude::*;
use world::{Particle as SimParticle, Object as SimObject, World as SimWorld, MaterialTyp, ParticleRef};

const GRID_WIDTH: usize = 120;
const GRID_HEIGHT: usize = 100;
const CELL_SIZE: f32 = 8.0;
const WINDOW_WIDTH: f32 = 960.0;
const WINDOW_HEIGHT: f32 = 800.0;
const CAMERA_SPEED: f32 = 400.0;

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

struct FragmentEvent {
    object_idx: usize,
    fragments: Vec<Vec<(usize, usize)>>,
}

#[derive(Resource, Default)]
struct FragmentEvents {
    events: Vec<FragmentEvent>,
}

#[derive(Resource)]
struct SelectedMaterial(MaterialTyp);

impl Default for SelectedMaterial {
    fn default() -> Self {
        SelectedMaterial(MaterialTyp::Sand)
    }
}

fn grid_to_screen(x: f32, y: f32) -> (f32, f32) {
    let screen_x = (x - GRID_WIDTH as f32 / 2.0 + 0.5) * CELL_SIZE;
    let screen_y = (y - GRID_HEIGHT as f32 / 2.0 + 0.5) * CELL_SIZE;
    (screen_x, screen_y)
}

fn material_to_color(material: MaterialTyp) -> Color {
    let (r, g, b) = material.color();
    Color::rgb(r, g, b)
}

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
        .add_systems(Update, camera_movement)
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

fn setup(mut commands: Commands, mut sim: ResMut<Simulation>) {
    commands.spawn((Camera2dBundle::default(), MainCamera));

    // Boden
    for x in 0..GRID_WIDTH {
        sim.world.update_occupation_on_position([x as f32, 0.0], ParticleRef::Static);
        sim.world.update_mass_on_position([x as f32, 0.0], 1000.0);

        let (screen_x, screen_y) = grid_to_screen(x as f32, 0.0);
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::GRAY,
                custom_size: Some(Vec2::new(CELL_SIZE - 1.0, CELL_SIZE - 1.0)),
                ..default()
            },
            transform: Transform::from_xyz(screen_x, screen_y, 0.0),
            ..default()
        });
    }

    // Debug-Label
    commands.spawn((
        TextBundle::from_section("", TextStyle { font_size: 16.0, color: Color::WHITE, ..default() })
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            }),
        DebugLabel,
    ));

    // Material-Label
    commands.spawn((
        TextBundle::from_section("", TextStyle { font_size: 18.0, color: Color::rgb(0.0, 1.0, 0.5), ..default() })
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                ..default()
            }),
        MaterialLabel,
    ));
}

fn camera_movement(
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    let mut camera_transform = camera_query.single_mut();
    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::W) || keyboard.pressed(KeyCode::Up) { direction.y += 1.0; }
    if keyboard.pressed(KeyCode::S) || keyboard.pressed(KeyCode::Down) { direction.y -= 1.0; }
    if keyboard.pressed(KeyCode::A) || keyboard.pressed(KeyCode::Left) { direction.x -= 1.0; }
    if keyboard.pressed(KeyCode::D) || keyboard.pressed(KeyCode::Right) { direction.x += 1.0; }

    if direction != Vec3::ZERO {
        camera_transform.translation += direction.normalize() * CAMERA_SPEED * time.delta_seconds();
    }
}

fn change_material(keyboard: Res<Input<KeyCode>>, mut selected: ResMut<SelectedMaterial>) {
    if keyboard.just_pressed(KeyCode::Key1) { selected.0 = MaterialTyp::Sand; }
    else if keyboard.just_pressed(KeyCode::Key2) { selected.0 = MaterialTyp::Stein; }
    else if keyboard.just_pressed(KeyCode::Key3) { selected.0 = MaterialTyp::Metall; }
    else if keyboard.just_pressed(KeyCode::Key4) { selected.0 = MaterialTyp::Holz; }
    else if keyboard.just_pressed(KeyCode::Key5) { selected.0 = MaterialTyp::Wasser; }
}

fn update_material_label(selected: Res<SelectedMaterial>, mut query: Query<&mut Text, With<MaterialLabel>>) {
    let mut text = query.single_mut();
    let mat_name = match selected.0 {
        MaterialTyp::Sand => "Sand [1]",
        MaterialTyp::Stein => "Stein [2]",
        MaterialTyp::Metall => "Metall [3]",
        MaterialTyp::Holz => "Holz [4]",
        MaterialTyp::Wasser => "Wasser [5]",
        MaterialTyp::Luft => "Luft",
    };
    text.sections[0].value = format!("Material: {}\n\n1-5=Material\nShift+Klick=Quadrant\nWASD=Kamera", mat_name);
}

fn spawn_particles(
    _commands: Commands,
    _sim: ResMut<Simulation>,
    _timers: ResMut<Timers>,
    _counter: ResMut<ParticleCounter>,
    _selected: Res<SelectedMaterial>,
    _time: Res<Time>,
) {
    // Deaktiviert - kein automatisches Spawning mehr
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
    if !mouse_button.just_pressed(MouseButton::Left) { return; }

    let window = windows.single();
    let camera_transform = camera_query.single();
    let cursor_pos = match window.cursor_position() {
        Some(pos) => pos,
        None => return,
    };

    let world_x = cursor_pos.x - WINDOW_WIDTH / 2.0 + camera_transform.translation.x;
    let world_y = WINDOW_HEIGHT / 2.0 - cursor_pos.y + camera_transform.translation.y;
    let grid_x = (world_x / CELL_SIZE + GRID_WIDTH as f32 / 2.0) as i32;
    let grid_y = (world_y / CELL_SIZE + GRID_HEIGHT as f32 / 2.0) as i32;

    let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let block_size = if shift_held { 4 } else { 3 };

    if grid_x < 0 || grid_x >= GRID_WIDTH as i32 - (block_size - 1)
        || grid_y < 0 || grid_y >= GRID_HEIGHT as i32 - (block_size - 1) { return; }

    for di in 0..block_size {
        for dj in 0..block_size {
            if sim.world.give_occupation_on_position((grid_x + dj) as usize, (grid_y + di) as usize).is_some() { return; }
        }
    }

    object_counter.0 += 1;
    let obj_id = object_counter.0;
    let obj_idx = sim.objects.len();

    if shift_held {
        let object = SimObject::new_quadrant(obj_id, obj_idx, [grid_x as f32, grid_y as f32], [0.0, 0.0]);

        for particle in object.get_object_elements() {
            sim.world.update_occupation_on_position(particle.position, particle.particle_ref);
            sim.world.update_mass_on_position(particle.position, particle.mass());
        }

        for i in 0..4 {
            for j in 0..4 {
                let particle = object.get_particle_at(i, j);
                let (screen_x, screen_y) = grid_to_screen(grid_x as f32 + j as f32, grid_y as f32 + i as f32);
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: material_to_color(particle.material),
                            custom_size: Some(Vec2::new(CELL_SIZE - 1.0, CELL_SIZE - 1.0)),
                            ..default()
                        },
                        transform: Transform::from_xyz(screen_x, screen_y, 2.0),
                        ..default()
                    },
                    ObjectSprite { object_idx: obj_idx, grid_i: i, grid_j: j },
                ));
            }
        }
        sim.objects.push(object);
    } else {
        let material = selected.0;
        let color = material_to_color(material);
        let object = SimObject::new(obj_id, obj_idx, [grid_x as f32, grid_y as f32], [0.0, 0.0], material, 3, 3);

        for particle in object.get_object_elements() {
            sim.world.update_occupation_on_position(particle.position, particle.particle_ref);
            sim.world.update_mass_on_position(particle.position, particle.mass());
        }

        for i in 0..3 {
            for j in 0..3 {
                let (screen_x, screen_y) = grid_to_screen(grid_x as f32 + j as f32, grid_y as f32 + i as f32);
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
                    ObjectSprite { object_idx: obj_idx, grid_i: i, grid_j: j },
                ));
            }
        }
        sim.objects.push(object);
    }
}

fn run_simulation(
    mut sim: ResMut<Simulation>,
    mut timers: ResMut<Timers>,
    mut fragment_events: ResMut<FragmentEvents>,
    time: Res<Time>,
) {
    timers.sim.tick(time.delta());
    if !timers.sim.just_finished() { return; }

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

    // Flüssigkeiten breiten sich seitlich aus
    for p in particles.iter_mut() {
        p.flow_sideways(world);
    }

    let Simulation { world, objects, .. } = &mut *sim;
    for (obj_idx, obj) in objects.iter_mut().enumerate() {
        if obj.is_destroyed { continue; }

        if let Some(fragments) = obj.update_object_velocity(gravity, world) {
            fragment_events.events.push(FragmentEvent { object_idx: obj_idx, fragments });
            continue;
        }

        if !obj.is_destroyed {
            obj.update_object_position(world);
        }
    }

    let Simulation { world, objects, .. } = &mut *sim;
    for (obj_idx, obj) in objects.iter_mut().enumerate() {
        if obj.is_destroyed { continue; }

        let vel = obj.get_object_velocity();
        if vel[1] != 0.0 { continue; }

        let broken_bonds = obj.check_pressure_fracture(world);
        if !broken_bonds.is_empty() {
            let fragments = obj.find_fragments(&broken_bonds);
            if fragments.len() > 1 {
                fragment_events.events.push(FragmentEvent { object_idx: obj_idx, fragments });
            }
        }
    }
}

fn handle_fragments(
    mut commands: Commands,
    mut sim: ResMut<Simulation>,
    mut fragment_events: ResMut<FragmentEvents>,
    mut counter: ResMut<ParticleCounter>,
    mut object_counter: ResMut<ObjectCounter>,
    object_sprites: Query<(Entity, &ObjectSprite)>,
) {
    if fragment_events.events.is_empty() { return; }

    for event in fragment_events.events.drain(..) {
        let obj_idx = event.object_idx;
        if obj_idx >= sim.objects.len() || sim.objects[obj_idx].is_destroyed { continue; }

        let old_velocity = sim.objects[obj_idx].get_object_velocity();
        let fragment_data: Vec<Vec<([f32; 2], MaterialTyp)>> = event.fragments.iter()
            .map(|frag| sim.objects[obj_idx].extract_fragment_data(frag))
            .collect();

        let Simulation { world, objects, .. } = &mut *sim;
        objects[obj_idx].clear_from_world(world);
        objects[obj_idx].is_destroyed = true;

        for (entity, sprite) in object_sprites.iter() {
            if sprite.object_idx == obj_idx {
                commands.entity(entity).despawn();
            }
        }

        for frag_data in fragment_data {
            if frag_data.len() == 1 {
                let (pos, material) = frag_data[0];
                counter.0 += 1;
                let idx = sim.particles.len();

                let particle = SimParticle::new(counter.0, pos, [0.0, 0.0], material, ParticleRef::Free(idx));
                sim.world.update_occupation_on_position(particle.position, particle.particle_ref);
                sim.world.update_mass_on_position(particle.position, particle.mass());
                sim.particles.push(particle);

                let color = material_to_color(material);
                let (screen_x, screen_y) = grid_to_screen(pos[0], pos[1]);
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color,
                            custom_size: Some(Vec2::new(CELL_SIZE - 1.0, CELL_SIZE - 1.0)),
                            ..default()
                        },
                        transform: Transform::from_xyz(screen_x, screen_y, 1.0),
                        ..default()
                    },
                    ParticleSprite(idx),
                ));
            } else {
                object_counter.0 += 1;
                let new_obj_idx = sim.objects.len();

                let new_object = SimObject::new_from_fragment(object_counter.0, new_obj_idx, &frag_data, old_velocity);

                for particle in new_object.get_object_elements() {
                    if particle.material != MaterialTyp::Luft {
                        sim.world.update_occupation_on_position(particle.position, particle.particle_ref);
                        sim.world.update_mass_on_position(particle.position, particle.mass());
                    }
                }

                let h = new_object.get_height();
                let w = new_object.get_width();
                for i in 0..h {
                    for j in 0..w {
                        let particle = new_object.get_particle_at(i, j);
                        if particle.material != MaterialTyp::Luft {
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
                                ObjectSprite { object_idx: new_obj_idx, grid_i: i, grid_j: j },
                            ));
                        }
                    }
                }
                sim.objects.push(new_object);
            }
        }
    }
}

fn update_sprites(sim: Res<Simulation>, mut query: Query<(&ParticleSprite, &mut Transform)>) {
    for (particle_sprite, mut transform) in query.iter_mut() {
        if particle_sprite.0 >= sim.particles.len() { continue; }
        let particle = &sim.particles[particle_sprite.0];
        let (screen_x, screen_y) = grid_to_screen(particle.position[0], particle.position[1]);
        transform.translation.x = screen_x;
        transform.translation.y = screen_y;
    }
}

fn update_object_sprites(sim: Res<Simulation>, mut query: Query<(&ObjectSprite, &mut Transform, &mut Visibility)>) {
    for (obj_sprite, mut transform, mut visibility) in query.iter_mut() {
        if obj_sprite.object_idx >= sim.objects.len() {
            *visibility = Visibility::Hidden;
            continue;
        }

        let object = &sim.objects[obj_sprite.object_idx];
        if object.is_destroyed {
            *visibility = Visibility::Hidden;
            continue;
        }

        *visibility = Visibility::Visible;
        let particle = object.get_particle_at(obj_sprite.grid_i, obj_sprite.grid_j);
        let (screen_x, screen_y) = grid_to_screen(particle.position[0], particle.position[1]);
        transform.translation.x = screen_x;
        transform.translation.y = screen_y;
    }
}

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
        None => { text.sections[0].value = "".to_string(); return; }
    };

    let world_x = cursor_pos.x - WINDOW_WIDTH / 2.0 + camera_transform.translation.x;
    let world_y = WINDOW_HEIGHT / 2.0 - cursor_pos.y + camera_transform.translation.y;
    let grid_x = ((world_x / CELL_SIZE + GRID_WIDTH as f32 / 2.0) as i32).max(0) as usize;
    let grid_y = ((world_y / CELL_SIZE + GRID_HEIGHT as f32 / 2.0) as i32).max(0) as usize;

    if grid_x >= GRID_WIDTH || grid_y >= GRID_HEIGHT {
        text.sections[0].value = "".to_string();
        return;
    }

    let pressure = sim.world.give_pressure_on_position(grid_x, grid_y);

    match sim.world.give_occupation_on_position(grid_x, grid_y) {
        Some(ParticleRef::Free(idx)) => {
            if idx < sim.particles.len() {
                let p = &sim.particles[idx];
                text.sections[0].value = format!(
                    "PARTIKEL #{}\nMaterial: {:?}\nDruck: {:.1}",
                    idx, p.material, pressure
                );
            }
        }
        Some(ParticleRef::InObject(obj_idx, i, j)) => {
            if obj_idx < sim.objects.len() && !sim.objects[obj_idx].is_destroyed {
                let obj = &sim.objects[obj_idx];
                let vel = obj.get_object_velocity();
                let particle = obj.get_particle_at(i, j);
                text.sections[0].value = format!(
                    "OBJECT #{}\nMaterial: {:?}\nVel: [{:.1}, {:.1}]\nDruck: {:.1}",
                    obj_idx, particle.material, vel[0], vel[1], pressure
                );
            }
        }
        Some(ParticleRef::Static) => {
            text.sections[0].value = format!("STATIC\nDruck: {:.1}", pressure);
        }
        None => {
            text.sections[0].value = format!("Leer [{}, {}]\nDruck: {:.1}", grid_x, grid_y, pressure);
        }
    }
}