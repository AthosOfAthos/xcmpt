use crate::{Component, component::ComponentInfo};
use alloc::alloc::{alloc_zeroed, dealloc};
use core::alloc::Layout;
use core::ptr::{copy_nonoverlapping, slice_from_raw_parts, slice_from_raw_parts_mut};

#[repr(u8)]
pub(crate) enum Slot<C: Component> {
	Empty = 0,
	Some(C),
}

pub(crate) struct ComponentArray {
	array: *mut u8,
	length: usize,
	array_layout: Layout,
	component_info: ComponentInfo,
}

impl ComponentArray {
	pub(crate) fn new(component_info: ComponentInfo, length: usize) -> Self {
		let array_layout = Layout::from_size_align(component_info.stride * length, component_info.layout.align()).unwrap();
		let array = unsafe { alloc_zeroed(array_layout) };

		ComponentArray { array, length, array_layout, component_info }
	}

	pub(crate) fn resize(&mut self, new_length: usize) {
		let new_layout = Layout::from_size_align(self.component_info.stride * new_length, self.component_info.layout.align()).unwrap();
		unsafe {
			let new_array = alloc_zeroed(new_layout);
			copy_nonoverlapping(self.array, new_array, self.component_info.stride * new_length);
			dealloc(self.array, self.array_layout);

			self.array = new_array;
			self.length = new_length;
			self.array_layout = new_layout;
		}
	}

	pub(crate) fn delete_index(&mut self, index: usize) {
		unsafe {
			let ptr = ((self.array as usize) + (index * self.component_info.stride)) as *mut u8;
			(self.component_info.drop)(ptr);
			*ptr = 0;
		}
	}

	pub(crate) unsafe fn get_slice<C: Component>(&self) -> &[Slot<C>] {
		&*slice_from_raw_parts(self.array as *const Slot<C>, self.length)
	}

	pub(crate) unsafe fn get_slice_mut<C: Component>(&mut self) -> &mut [Slot<C>] {
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
