use std::cell::UnsafeCell;

use crate::Components;
use util::rayon::prelude::*;

#[derive(Default)]
pub struct Scheduler {
    systems: Vec<SystemBox>,
}

pub trait System: 'static + Send + Sync {
    fn run(&mut self, components: &Components);
}

struct SystemBox(UnsafeCell<Box<dyn System>>);
unsafe impl Send for SystemBox {}
unsafe impl Sync for SystemBox {}

impl Scheduler {
    pub fn add(&mut self, system: impl System) -> &mut Self {
        self.systems
            .push(SystemBox(UnsafeCell::new(Box::new(system))));
        self
    }

    pub fn execute(&mut self, components: &Components) {
        self.systems.par_iter_mut().for_each(|system| {
            let system = system.0.get_mut();
            system.run(components);
        });
    }
}
