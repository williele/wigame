use crate::{
    entity::{Entities, Entity},
    Component, Components,
};

#[derive(Default)]
pub struct World {
    components: Components,
    entities: Entities,
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
            entity: self.entities.alloc(),
        }
    }

    pub fn despawn(&mut self, entity: Entity) {
        if let Some(components) = self.entities.delloc(entity) {
            components
                .iter()
                .for_each(|id| self.components.remove_raw(id, entity))
        }
    }

    pub fn add_component<C: Component>(&mut self, entity: Entity, component: C) {
        if self.entities.is_live(entity) {
            self.components.insert(entity, component);
            self.entities.add_component::<C>(entity);
        }
    }

    pub fn remove_commponent<C: Component>(&mut self, entity: Entity) {
        if self.entities.is_live(entity) {
            self.components.remove::<C>(entity);
            self.entities.remove_commponent::<C>(entity);
        }
    }

    pub fn flush(&mut self) {
        self.entities.flush();
    }

    pub(crate) fn components(&self) -> &Components {
        &self.components
    }

    // pub(crate) fn components_mut(&mut self) -> &mut Components {
    //     &mut self.components
    // }

    pub(crate) fn entities(&self) -> &Entities {
        &self.entities
    }

    pub(crate) fn entities_mut(&mut self) -> &mut Entities {
        &mut self.entities
    }
}
