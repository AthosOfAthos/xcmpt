use crate::component::ComponentID;
use crate::{Component, component::ComponentInfo};
use alloc::alloc::{alloc_zeroed, dealloc};
use core::alloc::Layout;
use core::ptr::{copy_nonoverlapping, slice_from_raw_parts, slice_from_raw_parts_mut};
use core::any::TypeId;
use hashbrown::HashMap;

#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Slot<C: Component> {
	Empty = 0,
	Some(C),
}

impl<C: Component> Slot<C> {
	pub(crate) fn is_filled(&self) -> bool {
		match self {
		    Slot::Empty => false,
		    Slot::Some(_) => true,
		}
	}

	pub(crate) fn as_option(&self) -> Option<&C> {
		match self {
			Slot::Empty => None,
			Slot::Some(component) => Some(component),
		}
	}
	
	pub(crate) fn as_option_mut(&mut self) -> Option<&mut C> {
		match self {
			Slot::Empty => None,
			Slot::Some(component) => Some(component),
		}
	}
}

struct ComponentArray {
	array: *mut u8,
	length: usize,
	array_layout: Layout,
	component_info: ComponentInfo,
}

impl ComponentArray {
	fn new(component_info: ComponentInfo, length: usize) -> Self {
		let array_layout = Layout::from_size_align(component_info.stride * length, component_info.layout.align()).unwrap();
		let array = unsafe { alloc_zeroed(array_layout) };
		assert!(!array.is_null());

		ComponentArray { array, length, array_layout, component_info }
	}

	fn resize(&mut self, new_length: usize) {
		let new_layout = Layout::from_size_align(self.component_info.stride * new_length, self.component_info.layout.align()).unwrap();
		unsafe {
			let new_array = alloc_zeroed(new_layout);
			copy_nonoverlapping(self.array, new_array, self.component_info.stride * self.length);
			dealloc(self.array, self.array_layout);

			self.array = new_array;
			self.length = new_length;
			self.array_layout = new_layout;
		}
	}

	fn delete_index(&mut self, index: usize) {
		unsafe {
			let ptr = ((self.array as usize) + (index * self.component_info.stride)) as *mut u8;
			(self.component_info.drop)(ptr);
			*ptr = 0;
		}
	}

	/// Get slice of internal array. DOES NOT VALIDATE THAT INTERNAL ARRAY IS OF TYPE C
	unsafe fn get_slice<C: Component>(&self) -> &[Slot<C>] {
		&*slice_from_raw_parts(self.array as *const Slot<C>, self.length)
	}

	/// Get mut slice of internal array. DOES NOT VALIDATE AND WILL ALIAS MUTS
	unsafe fn get_slice_mut<C: Component>(&self) -> &mut [Slot<C>] {
		&mut *slice_from_raw_parts_mut(self.array as *mut Slot<C>, self.length)
	}
}

impl Drop for ComponentArray {
    fn drop(&mut self) {
		for index in 0..self.length {
			self.delete_index(index);
		}

		unsafe { dealloc(self.array, self.array_layout) }
    }
}

pub(crate) struct ComponentMap {
	map: HashMap<ComponentID, ComponentArray>,
}

impl ComponentMap {
	pub(crate) fn new() -> Self {
		ComponentMap { map: HashMap::new() }
	}

	// Length must be uniform across ComponentArrays
	pub(crate) fn register(&mut self, id: ComponentID, info: ComponentInfo, length: usize) {
		let array = ComponentArray::new(info, length);
		self.map.insert(id, array);
	}

	pub(crate) fn resize(&mut self, new_length: usize) {
		for component_array in self.map.values_mut() {
			component_array.resize(new_length);
		}
	}

	pub(crate) fn delete_index(&mut self, index: usize) {
		for component in self.map.values_mut() {
			component.delete_index(index);
		}
	}

	pub(crate) fn get_array<C: Component>(&self) -> Option<&[Slot<C>]> {
		let array = self.map.get(&TypeId::of::<C>())?;
		unsafe { Some(array.get_slice::<C>()) }
	}

	/// DO NOT LET THIS ALIAS COMPONENTARRAYS
	pub(crate) unsafe fn get_array_mut<C: Component>(&self) -> Option<&mut [Slot<C>]> {
		let array = self.map.get(&TypeId::of::<C>())?;
		Some(array.get_slice_mut::<C>())
	}
}

#[cfg(test)]
mod test {
    use crate::{Component, component::ComponentInfo, storage::{ComponentArray, Slot}};

	#[derive(Clone, Copy, Debug, PartialEq, Eq)]
	pub struct TestComponent(usize);
	impl Component for TestComponent {}
	const INFO: ComponentInfo = ComponentInfo::new::<TestComponent>();

	#[test]
	fn delete_index() {
		const LENGTH: usize = 32;
		let mut array = ComponentArray::new(INFO, LENGTH);

		unsafe {
			let slice_mut = array.get_slice_mut::<TestComponent>();
			for index in 0..LENGTH {
				slice_mut[index] = Slot::Some(TestComponent(index));
			}
		}

		for index in 0..LENGTH {
			array.delete_index(index);
		}

		unsafe {
			let slice = array.get_slice::<TestComponent>();
			for index in 0..LENGTH {
				assert_eq!(slice[index], Slot::Empty);
			}
		}
	}

	#[test]
	fn resize() {
		const STARTING_LENGTH: usize = 64;
		const RESIZED_LENGTH: usize = 128;
		let mut array = ComponentArray::new(INFO, STARTING_LENGTH);

		unsafe {
			let slice_mut = array.get_slice_mut::<TestComponent>();
			for index in 0..STARTING_LENGTH {
				slice_mut[index] = Slot::Some(TestComponent(index));
			}
		}

		array.resize(RESIZED_LENGTH);

		unsafe {
			let slice = array.get_slice::<TestComponent>();
			for index in 0..RESIZED_LENGTH {
				let component = if index < STARTING_LENGTH {
					Slot::Some(TestComponent(index))
				} else {
					Slot::Empty
				};
				assert_eq!(slice[index], component);
			}
		}
	}
}
