use crate::{Scheduler, System, World};

#[derive(Default)]
pub struct App {
    world: World,
    scheduler: Scheduler,
}

impl App {
    pub fn new() -> Self {
        App::default()
    }

    pub fn add_system(&mut self, system: impl System) -> &mut Self {
        self.scheduler.add(system);
        self
    }

    pub fn update(&mut self) {
        self.scheduler.execute(&mut self.world);
    }
}
