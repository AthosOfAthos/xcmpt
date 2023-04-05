
use crate::{Component, ComponentRegistry, component::ComponentID, storage::ComponentArray};
use alloc::rc::Rc;
use alloc::vec::Vec;
use hashbrown::HashMap;
use runtime_id::RuntimeID;

type Index = usize;
type Generation = usize;

#[derive(Clone, Copy)]
pub struct EntityID {
	scene_id: RuntimeID,
	index: Index,
	generation: Generation,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Entity {
	alive: bool,
	generation: Generation,
}

pub struct ECS {
	scene_id: RuntimeID,
	length: usize,
	entities: Vec<Entity>,
	components: HashMap<ComponentID, ComponentArray>,
}

impl ECS {
	pub fn new(registry: &ComponentRegistry) -> Self {
		const STARTING_LENGTH: usize = 512;
		
		let scene_id = RuntimeID::new();
		let mut entities = Vec::with_capacity(STARTING_LENGTH);
		entities.resize(STARTING_LENGTH, Entity { alive: false, generation: 0 });
		let mut components = HashMap::new();
		for (component_id, component_info) in &registry.components {
			let array = ComponentArray::new(*component_info, STARTING_LENGTH);
			components.insert(*component_id, array);
		}

		ECS { scene_id, length: STARTING_LENGTH, entities, components }
	}

	fn allocate_entity(&mut self) -> Option<(Index, Generation)> {
		for index in 0..self.length {
			let entity = &mut self.entities[index];
			if !entity.alive {
				entity.alive = true;
				entity.generation += 1;
				return Some((index, entity.generation));
			}
		}
		None
	}

	pub fn is_valid(&self, entity: &EntityID) -> bool {
		if entity.scene_id != self.scene_id { return false; }
		self.entities[entity.index] == Entity { alive: true, generation: entity.generation }
	}

	pub fn create_entity(&mut self) -> EntityID {
		let (index, generation) = self.allocate_entity().unwrap();
		
		EntityID { scene_id: self.scene_id, index, generation }
	}

	pub fn destroy_entity(&mut self, entity: EntityID) {
		if self.is_valid(&entity) {
			for component_array in self.components.values_mut() {
				component_array.delete_index(entity.index);
			}

			self.entities[entity.index].alive = false;
		}
	}

	pub fn has_component<C: Component>(&self, entity: &EntityID) -> bool {
		unimplemented!()
	}

	pub fn add_component<C: Component>(&mut self, entity: &EntityID, component: C) {
		unimplemented!()
	}

	pub fn remove_component<C: Component>(&mut self, entity: &EntityID) -> Option<C> {
		unimplemented!()
	}

	pub fn get_component<C: Component>(&self, entity: &EntityID) -> Option<&C> {
		unimplemented!()
	}

	pub fn get_component_mut<C: Component>(&mut self, entity: &EntityID) -> Option<&mut C> {
		unimplemented!()
	}
}

#[cfg(test)]
mod test {
	use crate::{ComponentRegistry, ECS};

	#[test]
	fn basic() {
		let registry = ComponentRegistry::new();
		let mut ecs = ECS::new(&registry);

		let entity = ecs.create_entity();
		assert!(ecs.is_valid(&entity));
		ecs.destroy_entity(entity);
		assert!(!ecs.is_valid(&entity));
	}
}
