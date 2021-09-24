use std::{any::type_name, collections::VecDeque, fmt, marker::PhantomData, sync::Arc};

use crate::{Component, Entity, World};

pub trait WorldWritable: Send + Sync {
    fn write(self: Arc<Self>, world: &mut World, cmd: &CommandBuffer);
}

struct DespawnCommand(Entity);
impl WorldWritable for DespawnCommand {
    fn write(self: Arc<Self>, world: &mut World, _cmd: &CommandBuffer) {
        world.despawn(self.0)
    }
}

struct RemoveComponentCommand<C> {
    entity: Entity,
    _marker: PhantomData<C>,
}

impl<C: Component> WorldWritable for RemoveComponentCommand<C> {
    fn write(self: Arc<Self>, world: &mut World, _cmd: &CommandBuffer) {
        world.remove_commponent::<C>(self.entity)
    }
}

struct AddComponentCommand<C> {
    entity: Entity,
    component: C,
}

impl<T> fmt::Debug for AddComponentCommand<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "AddComponentCommand<{}>({:?})",
            type_name::<T>(),
            self.entity
        ))
    }
}

impl<C: Component> WorldWritable for AddComponentCommand<C> {
    fn write(self: Arc<Self>, world: &mut World, _cmd: &CommandBuffer) {
        let comsumed = Arc::try_unwrap(self).unwrap();
        world.add_component(comsumed.entity, comsumed.component);
    }
}

enum Command {
    WriteWorld(Arc<dyn WorldWritable>),
}

pub struct CommandBuffer {
    commands: VecDeque<Command>,
}

impl CommandBuffer {
    pub fn new() -> Self {
        CommandBuffer {
            commands: Default::default(),
        }
    }

    fn push_writer<W: 'static + WorldWritable>(&mut self, writer: W) {
        self.commands
            .push_front(Command::WriteWorld(Arc::new(writer)));
    }

    pub fn despawn(&mut self, entity: Entity) -> &mut Self {
        self.push_writer(DespawnCommand(entity));
        self
    }

    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> &mut Self {
        self.push_writer(RemoveComponentCommand {
            entity,
            _marker: PhantomData::<T>,
        });
        self
    }

    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) -> &mut Self {
        self.push_writer(AddComponentCommand { component, entity });
        self
    }

    pub fn flush(&mut self, world: &mut World) {
        world.entities_mut().flush();
        while let Some(command) = self.commands.pop_back() {
            match command {
                Command::WriteWorld(arc) => arc.write(world, self),
            }
        }
    }
}
