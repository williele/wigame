use crate::{
    entity::{Entity, EntityAllocator},
    Component, Components,
};

#[derive(Default)]
pub struct World {
    components: Components,
    entity_allocator: EntityAllocator,
}

pub struct WorldEntityEditor<'a> {
    world: &'a mut World,
    entity: Entity,
}

impl<'a> WorldEntityEditor<'a> {
    pub fn add<T: Component>(&mut self, component: T) -> &mut Self {
        self.world.add_component(self.entity, component);
        self
    }

    pub fn remove<T: Component>(&mut self) -> &mut Self {
        self.world.remove_commponent::<T>(self.entity);
        self
    }

    pub fn despawn(&mut self) -> Entity {
        self.world.despawn(self.entity);
        self.entity
    }

    pub fn entity(&mut self) -> Entity {
        self.entity
    }
}

impl World {
    pub fn spawn(&mut self) -> WorldEntityEditor {
        let entity = self.entity_allocator.alloc();
        WorldEntityEditor {
            world: self,
            entity,
        }
    }

    pub fn despawn(&mut self, entity: Entity) {
        if let Some(components) = self.entity_allocator.delloc(entity) {
            components
                .iter()
                .for_each(|id| self.components.remove_raw(id, entity))
        }
    }

    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) {
        if self.entity_allocator.is_live(entity) {
            self.components.insert(entity, component);
            self.entity_allocator.add_component::<T>(entity);
        }
    }

    pub fn remove_commponent<T: Component>(&mut self, entity: Entity) {
        if self.entity_allocator.is_live(entity) {
            self.components.remove::<T>(entity);
            self.entity_allocator.remove_commponent::<T>(entity);
        }
    }

    pub(crate) fn reserve_entity(&self) -> Entity {
        self.entity_allocator.reserve()
    }

    pub(crate) fn flush(&mut self) {
        self.entity_allocator.flush();
    }

    pub(crate) fn components(&self) -> &Components {
        &self.components
    }

    pub(crate) fn entity_allocator(&self) -> &EntityAllocator {
        &self.entity_allocator
    }
}
