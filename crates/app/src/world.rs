use crate::{
    entity::{Entity, EntityAllocator},
    Component, Components,
};

#[derive(Default)]
pub struct World {
    components: Components,
    entity_allocator: EntityAllocator,
}

pub struct WorldExt<'a> {
    components: &'a mut Components,
    entity: Entity,
}

impl<'a> WorldExt<'a> {
    pub fn with<C: Component>(&mut self, component: C) -> &mut Self {
        self.components.insert(self.entity, component);
        self
    }

    pub fn build(&mut self) -> Entity {
        self.entity
    }
}

impl World {
    pub fn spawn(&mut self) -> WorldExt {
        WorldExt {
            components: &mut self.components,
            entity: self.entity_allocator.alloc(),
        }
    }

    pub fn despawn(&mut self, entity: Entity) {
        if let Some(components) = self.entity_allocator.delloc(entity) {
            components
                .iter()
                .for_each(|id| self.components.remove_raw(id, entity))
        }
    }

    pub fn add_component<C: Component>(&mut self, entity: Entity, component: C) {
        if self.entity_allocator.is_live(entity) {
            self.components.insert(entity, component);
            self.entity_allocator.add_component::<C>(entity);
        }
    }

    pub fn remove_commponent<C: Component>(&mut self, entity: Entity) {
        if self.entity_allocator.is_live(entity) {
            self.components.remove::<C>(entity);
            self.entity_allocator.remove_commponent::<C>(entity);
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
