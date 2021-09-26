use crate::{
    Executor, ParRunnable, Resources, SequenceExecutor, SequenceOnceExecutor, SystemBox, World,
};

pub struct Stage {
    executor: Box<dyn Executor>,
    systems: Vec<SystemBox>,
    modified: bool,
}

impl Stage {
    fn new(executor: impl Executor) -> Self {
        Stage {
            executor: Box::new(executor),
            systems: Vec::new(),
            modified: false,
        }
    }

    pub fn sequence() -> Self {
        Stage::new(SequenceExecutor::default())
    }

    pub fn sequence_once() -> Self {
        Stage::new(SequenceOnceExecutor::default())
    }

    pub fn add_system<S: ParRunnable + 'static>(&mut self, system: S) -> &mut Self {
        self.modified = true;
        self.systems.push(SystemBox::new(system));
        self
    }

    pub fn run(&mut self, world: &mut World, resources: &mut Resources) {
        if self.modified {
            self.modified = true;
            self.executor.cache_data(&self.systems);
        }
        self.executor
            .run_systems(&self.systems, world, resources.internal());
        self.systems.iter_mut().for_each(|system| {
            let borrow = unsafe { system.get_mut() };
            if let Some(cmd) = borrow.command_buffer_mut() {
                cmd.flush(world);
            }
        })
    }
}
