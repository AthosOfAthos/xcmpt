
use core::any::TypeId;

use crate::storage::{Slot, ComponentMap};
use crate::{Component, ComponentRegistry};
use alloc::vec::Vec;
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
	components: ComponentMap,
}

impl ECS {
	pub fn new(registry: &ComponentRegistry) -> Self {
		const STARTING_LENGTH: usize = 512;
		
		let scene_id = RuntimeID::new();
		let mut entities = Vec::with_capacity(STARTING_LENGTH);
		entities.resize(STARTING_LENGTH, Entity { alive: false, generation: 0 });
		let components = ComponentMap::new(registry, STARTING_LENGTH);
		
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
			self.components.delete_index(entity.index);
			self.entities[entity.index].alive = false;
		}
	}

	pub fn has_component<C: Component>(&self, entity: &EntityID) -> bool {
		if !self.is_valid(entity) { return false; }
		return match self.components.get_array::<C>() {
		    Some(array) => array[entity.index].is_filled(),
		    None => false,
		}
	}

	pub fn add_component<C: Component>(&mut self, entity: &EntityID, component: C) {
		if !self.is_valid(entity) { return; }
		match unsafe { self.components.get_array_mut::<C>() } {
		    Some(array) => array[entity.index] = Slot::Some(component),
		    None => todo!(),
		}
	}

	pub fn remove_component<C: Component>(&mut self, entity: &EntityID) {
		if !self.is_valid(entity) { return; }
		match unsafe { self.components.get_array_mut::<C>() } {
			Some(array) => array[entity.index] = Slot::Empty,
			None => todo!(),
		}
	}

	pub fn get_component<C: Component>(&self, entity: &EntityID) -> Option<&C> {
		if !self.is_valid(entity) { return None; }
		match self.components.get_array::<C>() {
		    Some(array) => array[entity.index].as_option(),
		    None => None,
		}
	}

	pub fn get_component_mut<C: Component>(&mut self, entity: &EntityID) -> Option<&mut C> {
		if !self.is_valid(entity) { return None; }
		match unsafe { self.components.get_array_mut::<C>() } {
		    Some(array) => array[entity.index].as_option_mut(),
		    None => None,
		}
	}
}

#[cfg(test)]
mod test {
	use crate::{ComponentRegistry, ECS, Component};

	#[test]
	fn basic() {
		let registry = ComponentRegistry::new();
		let mut ecs = ECS::new(&registry);

		let entity = ecs.create_entity();
		assert!(ecs.is_valid(&entity));
		ecs.destroy_entity(entity);
		assert!(!ecs.is_valid(&entity));
	}

	#[derive(PartialEq, Eq)]
	struct TestComponent(usize);
	impl Component for TestComponent {}

	#[test]
	fn component_basic() {
		let mut registry = ComponentRegistry::new();
		registry.register::<TestComponent>();
		let mut ecs = ECS::new(&registry);

		let entity = ecs.create_entity();
		assert!(!ecs.has_component::<TestComponent>(&entity));
		let test_component = TestComponent(128);
		ecs.add_component(&entity, test_component);
		assert!(ecs.has_component::<TestComponent>(&entity));

		ecs.get_component_mut::<TestComponent>(&entity).unwrap().0 = 72;
		let component = ecs.get_component::<TestComponent>(&entity).unwrap();
		assert_eq!(component.0, 72);
		
		ecs.remove_component::<TestComponent>(&entity);
		assert!(!ecs.has_component::<TestComponent>(&entity));
	}

	mod drop {
		use crate::{ComponentRegistry, ECS, Component};
		use core::sync::atomic::{AtomicUsize, Ordering};

		static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);
		struct Dropper {}
		impl Component for Dropper {}
		impl Drop for Dropper {
		    fn drop(&mut self) {
				DROP_COUNT.fetch_add(1, Ordering::Relaxed);
		    }
		}

		#[test]
		fn drop_test() {
			const COUNT: usize = 231;
			let mut registry = ComponentRegistry::new();
			registry.register::<Dropper>();

			let mut ecs = ECS::new(&registry);
			for _ in 0..(COUNT - 1) {
				let entity = ecs.create_entity();
				ecs.add_component::<Dropper>(&entity, Dropper {});
			}

			let entity = ecs.create_entity();
			ecs.add_component::<Dropper>(&entity, Dropper {});
			ecs.remove_component::<Dropper>(&entity);
			drop(ecs);
			
			assert_eq!(DROP_COUNT.load(Ordering::Relaxed), COUNT);
		}
	}
}
