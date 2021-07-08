use crate::bullet::{BulletAccessor, BulletHandle};
use crate::index_set::IndexSet;
use crate::ship::{ShipAccessor, ShipAccessorMut, ShipHandle};
use rapier2d_f64::prelude::*;

pub const WORLD_SIZE: f64 = 1000.0;

pub(crate) const WALL_COLLISION_GROUP: u32 = 0;
pub(crate) const SHIP_COLLISION_GROUP: u32 = 1;
pub(crate) const BULLET_COLLISION_GROUP: u32 = 2;

pub struct Simulation {
    pub ships: IndexSet<ShipHandle>,
    pub bullets: IndexSet<BulletHandle>,
    pub(crate) bodies: RigidBodySet,
    pub(crate) colliders: ColliderSet,
    pub(crate) joints: JointSet,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    pub(crate) island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    ccd_solver: CCDSolver,
    event_collector: rapier2d_f64::pipeline::ChannelEventCollector,
    contact_recv: crossbeam::channel::Receiver<ContactEvent>,
    intersection_recv: crossbeam::channel::Receiver<IntersectionEvent>,
    pub collided: bool,
}

impl Simulation {
    pub fn new() -> Simulation {
        let (contact_send, contact_recv) = crossbeam::channel::unbounded();
        let (intersection_send, intersection_recv) = crossbeam::channel::unbounded();
        Simulation {
            ships: IndexSet::new(),
            bullets: IndexSet::new(),
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            joints: JointSet::new(),
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            ccd_solver: CCDSolver::new(),
            event_collector: ChannelEventCollector::new(intersection_send, contact_send),
            contact_recv,
            intersection_recv,
            collided: false,
        }
    }

    pub fn ship(self: &Simulation, handle: ShipHandle) -> ShipAccessor {
        ShipAccessor {
            simulation: self,
            handle,
        }
    }

    pub fn ship_mut(self: &mut Simulation, handle: ShipHandle) -> ShipAccessorMut {
        ShipAccessorMut {
            simulation: self,
            handle,
        }
    }

    pub fn bullet(self: &Simulation, handle: BulletHandle) -> BulletAccessor {
        BulletAccessor {
            simulation: self,
            handle,
        }
    }

    pub fn step(self: &mut Simulation) {
        let gravity = vector![0.0, 0.0];
        let physics_hooks = ();

        self.physics_pipeline.step(
            &gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joints,
            &mut self.ccd_solver,
            &physics_hooks,
            &self.event_collector,
        );

        while let Ok(event) = self.contact_recv.try_recv() {
            if let ContactEvent::Started(h1, h2) = event {
                let get_index = |h| self.colliders.get(h).and_then(|x| x.parent()).map(|x| x.0);
                if let (Some(idx1), Some(idx2)) = (get_index(h1), get_index(h2)) {
                    if self.bullets.contains(BulletHandle(idx1))
                        && self.ships.contains(ShipHandle(idx2))
                    {
                        self.ship_mut(ShipHandle(idx2)).explode();
                    } else if self.bullets.contains(BulletHandle(idx2))
                        && self.ships.contains(ShipHandle(idx1))
                    {
                        self.ship_mut(ShipHandle(idx1)).explode();
                    }
                }

                self.collided = true;
            }
        }

        while self.intersection_recv.try_recv().is_ok() {}
    }
}

impl Default for Simulation {
    fn default() -> Self {
        Simulation::new()
    }
}

pub struct CollisionEventHandler {
    pub collision: crossbeam::atomic::AtomicCell<bool>,
}

impl CollisionEventHandler {
    pub fn new() -> CollisionEventHandler {
        CollisionEventHandler {
            collision: crossbeam::atomic::AtomicCell::new(false),
        }
    }
}

impl EventHandler for CollisionEventHandler {
    fn handle_intersection_event(&self, _event: IntersectionEvent) {}

    fn handle_contact_event(&self, event: ContactEvent, _contact_pair: &ContactPair) {
        if let ContactEvent::Started(_, _) = event {
            self.collision.store(true);
            //println!("Collision: {:?}", event);
        }
    }
}

impl Default for CollisionEventHandler {
    fn default() -> Self {
        CollisionEventHandler::new()
    }
}
