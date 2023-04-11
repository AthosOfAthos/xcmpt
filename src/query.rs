use crate::{Component, EntityID, storage::Slot, ECS};
use core::{iter::Iterator, any::TypeId};

pub trait Query {
	type Output<'a> where Self: 'a;
	type Array;

	unsafe fn get_array(ecs: &ECS) -> Self::Array;

	unsafe fn next<'a>(ecs: &'a ECS, index: &mut usize, array: *const Self::Array) -> Option<Self::Output<'a>>;
}

impl<C: Component> Query for C {
	type Output<'a> = (EntityID, &'a C);
	type Array = *const [Slot<C>];

	unsafe fn get_array(ecs: &ECS) -> Self::Array {
		ecs.components.get_array::<C>().unwrap()
    }

	unsafe fn next<'a>(ecs: &'a ECS, index: &mut usize, array: *const Self::Array) -> Option<Self::Output<'a>> {
		while (*index) < ecs.capacity {
			let id = ecs.get_index(*index);
			let element = &(**array)[*index];
			*index += 1;
			if element.is_filled() {
				return Some((id.unwrap(), element.as_option().unwrap()));
			}
		}
		None
    }
}

impl<C0: Component, C1: Component> Query for (C0, C1) {
	type Output<'a> = (EntityID, &'a C0, &'a C1);
	type Array = (*const [Slot<C0>], *const [Slot<C1>]);

	unsafe fn get_array(ecs: &ECS) -> Self::Array {
		if TypeId::of::<C0>() == TypeId::of::<C1>() {
			panic!("Cannot Query for multiple of the same Component type");
		}
		
		let c0_array = ecs.components.get_array::<C0>().unwrap();
		let c1_array = ecs.components.get_array::<C1>().unwrap();
		(c0_array, c1_array)
    }

	unsafe fn next<'a>(ecs: &'a ECS, index: &mut usize, array: *const Self::Array) -> Option<Self::Output<'a>> {
		while (*index) < ecs.capacity {
			let id = ecs.get_index(*index);
			let element_0 = &(*(*array).0)[*index];
			let element_1 = &(*(*array).1)[*index];
			
			*index += 1;
			if element_0.is_filled() && element_1.is_filled() {
				return Some((id.unwrap(), element_0.as_option().unwrap(), element_1.as_option().unwrap()));
			}
		}
		None
    }
}

pub struct QueryIter<'a, Q: Query + 'a> {
	ecs: &'a ECS,
	index: usize,
	array: Q::Array,
}

impl<'a, Q: Query> QueryIter<'a, Q> {
	pub(crate) fn new(ecs: &'a ECS) -> Self {
		let array = unsafe { Q::get_array(ecs) };
		QueryIter { ecs, index: 0, array }
	}
}

impl<'a, Q: Query> Iterator for QueryIter<'a, Q> {
	type Item = Q::Output<'a>;
	fn next(&mut self) -> Option<Self::Item> {
		unsafe { Q::next(self.ecs, &mut self.index, &self.array) }
    }
}
