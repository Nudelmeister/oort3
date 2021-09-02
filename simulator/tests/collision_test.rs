use oort_simulator::ship::{fighter, missile};
use oort_simulator::simulation;
use oort_simulator::simulation::WORLD_SIZE;
use oort_simulator::{bullet, scenario, ship};
use rand::Rng;
use test_env_log::test;

#[test]
fn test_world_edge() {
    let mut rng = rand::thread_rng();
    let mut sim = simulation::Simulation::new("test", 0, "");
    scenario::add_walls(&mut sim);

    for _ in 0..100 {
        let s = 500.0;
        let r = rng.gen_range(10.0..20.0);
        let x = rng.gen_range((r - WORLD_SIZE / 2.0)..(WORLD_SIZE / 2.0 - r));
        let y = rng.gen_range((r - WORLD_SIZE / 2.0)..(WORLD_SIZE / 2.0 - r));
        let h = rng.gen_range(0.0..(2.0 * std::f32::consts::PI));
        let vx = rng.gen_range(-s..s);
        let vy = rng.gen_range(-s..s);
        ship::create(
            &mut sim,
            x as f64,
            y as f64,
            vx as f64,
            vy as f64,
            h as f64,
            fighter(0),
        );
    }

    for _ in 0..1000 {
        sim.step();
    }

    for &index in sim.ships.iter() {
        let ship = sim.ship(index);
        assert!(ship.position().x >= -WORLD_SIZE / 2.0);
        assert!(ship.position().x <= WORLD_SIZE / 2.0);
        assert!(ship.position().y >= -WORLD_SIZE / 2.0);
        assert!(ship.position().y <= WORLD_SIZE / 2.0);
    }
}

#[test]
fn test_head_on_collision() {
    let mut sim = simulation::Simulation::new("test", 0, "");

    let ship0 = ship::create(&mut sim, -100.0, 0.0, 100.0, 0.0, 0.0, fighter(0));
    let ship1 = ship::create(&mut sim, 100.0, 0.0, -100.0, 0.0, 0.0, fighter(0));

    assert!(sim.ship(ship0).velocity().x > 0.0);
    assert!(sim.ship(ship1).velocity().x < 0.0);

    for _ in 0..1000 {
        sim.step();
    }

    assert!(sim.ship(ship0).velocity().x < 0.0);
    assert!(sim.ship(ship1).velocity().x > 0.0);
}

#[test]
fn test_fighter_bullet_collision_same_team() {
    let mut sim = simulation::Simulation::new("test", 0, "");

    let ship = ship::create(&mut sim, 100.0, 0.0, 0.0, 0.0, 0.0, fighter(0));
    bullet::create(
        &mut sim,
        0.0,
        0.0,
        1000.0,
        0.0,
        bullet::BulletData {
            team: 0,
            damage: 10.0,
        },
    );

    for _ in 0..60 {
        sim.step();
    }

    assert_ne!(sim.ship(ship).velocity().x, 0.0);
    assert_eq!(sim.bullets.len(), 0);
}

#[test]
fn test_fighter_bullet_collision_different_team() {
    let mut sim = simulation::Simulation::new("test", 0, "");

    let ship = ship::create(&mut sim, 100.0, 0.0, 0.0, 0.0, 0.0, fighter(0));
    bullet::create(
        &mut sim,
        0.0,
        0.0,
        1000.0,
        0.0,
        bullet::BulletData {
            team: 1,
            damage: 10.0,
        },
    );

    for _ in 0..60 {
        sim.step();
    }

    assert_ne!(sim.ship(ship).velocity().x, 0.0);
    assert_eq!(sim.bullets.len(), 0);
}

#[test]
fn test_missile_bullet_collision_same_team() {
    let mut sim = simulation::Simulation::new("test", 0, "");

    let msl = ship::create(&mut sim, 100.0, 0.0, 0.0, 0.0, 0.0, missile(0));
    let blt = bullet::create(
        &mut sim,
        0.0,
        0.0,
        1000.0,
        0.0,
        bullet::BulletData {
            team: 0,
            damage: 10.0,
        },
    );

    for _ in 0..60 {
        sim.step();
    }

    assert_eq!(sim.ship(msl).velocity().x, 0.0);
    assert_eq!(sim.bullet(blt).body().linvel().x, 1000.0);
}

#[test]
fn test_missile_bullet_collision_different_team() {
    let mut sim = simulation::Simulation::new("test", 0, "");

    ship::create(&mut sim, 100.0, 0.0, 0.0, 0.0, 0.0, missile(0));
    bullet::create(
        &mut sim,
        0.0,
        0.0,
        1000.0,
        0.0,
        bullet::BulletData {
            team: 1,
            damage: 10.0,
        },
    );

    for _ in 0..60 {
        sim.step();
    }

    assert_eq!(sim.ships.len(), 0);
    assert_eq!(sim.bullets.len(), 0);
}

#[test]
fn test_missile_fighter_collision_same_team() {
    let mut sim = simulation::Simulation::new("test", 0, "");

    let msl = ship::create(&mut sim, 0.0, 0.0, 400.0, 0.0, 0.0, missile(0));
    let ship = ship::create(&mut sim, 100.0, 0.0, 0.0, 0.0, 0.0, fighter(0));

    for _ in 0..60 {
        sim.step();
    }

    assert_ne!(sim.ship(ship).velocity().x, 0.0);
    assert_ne!(sim.ship(msl).body().linvel().x, 400.0);
}

#[test]
fn test_missile_fighter_collision_different_team() {
    let mut sim = simulation::Simulation::new("test", 0, "");

    let msl = ship::create(&mut sim, 0.0, 0.0, 400.0, 0.0, 0.0, missile(0));
    let ship = ship::create(&mut sim, 100.0, 0.0, 0.0, 0.0, 0.0, fighter(1));

    for _ in 0..60 {
        sim.step();
    }

    assert_ne!(sim.ship(ship).velocity().x, 0.0);
    assert_ne!(sim.ship(msl).body().linvel().x, 400.0);
}
