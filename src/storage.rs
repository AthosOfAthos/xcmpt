use crate::{Component, component::ComponentInfo};

#[repr(u8)]
pub(crate) enum Slot<C: Component> {
	Empty = 0,
	Some(C),
}

pub(crate) struct ComponentArray {
	data: *mut u8,
	component_info: ComponentInfo,
}

impl ComponentArray {
	pub(crate) fn new(component_info: ComponentInfo, length: usize) -> Self {
		unimplemented!()
	}

	pub(crate) fn resize(&mut self, length: usize) {
		unimplemented!()
	}

	pub(crate) fn delete_index(&mut self, index: usize) {
		unimplemented!()
	}

	pub(crate) unsafe fn get_slice<C: Component>(&self) -> &[Slot<C>] {
		unimplemented!()
	}

	pub(crate) unsafe fn get_slice_mut<C: Component>(&mut self) -> &mut [Slot<C>] {
		unimplemented!()
	}
}
