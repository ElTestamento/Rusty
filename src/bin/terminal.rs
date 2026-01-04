use world::{Particle, World};

fn main() {
    println!("########################Simulation startet##################");
    let h = 20;
    let b = 20;
    println!(
        "Welt erstellen mit einer Höhe von {} und einer Breite von {}",
        h, b
    );
    let mut world = World::new(h, b);
    let gravity: [f32; 2] = [0.0, -0.5];
    let mass = 10.0;
    println!("Es wirkt eine Schwerkraft von {:?}", gravity);

    let mut prtl: Particle = Particle::new(1, [0.0, 10.0], [0.0, 0.0], mass);
    let mut prtl2: Particle = Particle::new(2, [0.0, 12.0], [0.0, 0.0], mass);

    for tick in 1..=20 {
        world.calc_pressure_on_all_position();

        prtl.update_velocity(gravity, &world);
        prtl2.update_velocity(gravity, &world);

        prtl.update_position(&mut world);
        prtl2.update_position(&mut world);

        prtl.resolve_pressure(&mut world);
        prtl2.resolve_pressure(&mut world);

        prtl.fall_down(&mut world);
        prtl2.fall_down(&mut world);

        println!(
            "Tick {}: P1 {:?} / P2 {:?}",
            tick,
            prtl.get_position(),
            prtl2.get_position()
        );
    }

    world.calc_pressure_on_all_position();
    world.give_world();
}