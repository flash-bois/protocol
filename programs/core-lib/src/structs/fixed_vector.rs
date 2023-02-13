use checked_decimal_macro::num_traits::ToPrimitive;
use std::{
    ops::Range,
    slice::{Iter, IterMut},
};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FixedSizeVector<T, const N: usize>
where
    T: Default + Sized,
{
    head: u8,
    elements: [T; N],
}

impl<T, const N: usize> Default for FixedSizeVector<T, N>
where
    T: Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> FixedSizeVector<T, N>
where
    T: Default + Sized,
{
    pub fn new() -> Self {
        assert!(N.to_u8().is_some(), "size expands u8 range");

        Self {
            head: 0,
            elements: [(); N].map(|_| T::default()),
        }
    }

    pub fn iter<'a>(&'a self) -> Option<Iter<'a, T>> {
        if self.head == 0 {
            return None;
        }

        let range = ..self.head_usize();

        Some(self.elements.get(range)?.iter())
    }

    pub fn iter_mut<'a>(&'a mut self) -> Option<IterMut<'a, T>> {
        if self.head == 0 {
            return None;
        }

        let range = ..self.head_usize();

        Some(self.elements.get_mut(range)?.iter_mut())
    }

    pub fn indexes(&self) -> Range<usize> {
        0..(self.head as usize)
    }

    fn head(&self) -> u8 {
        self.head
    }

    fn head_usize(&self) -> usize {
        self.head().to_usize().unwrap()
    }

    /// checks if index is in allocated range
    fn index_in_range(&self, id: usize) -> Option<()> {
        if id < N {
            Some(())
        } else {
            None
        }
    }

    /// checks if index is in useful range
    fn index_before_head(&self, id: usize) -> bool {
        self.head > 0 && id < self.head_usize()
    }

    fn get_mut(&mut self, id: usize) -> Option<&mut T> {
        self.index_in_range(id)?;
        self.elements.get_mut(id)
    }

    /// get mut checks if index is in initialized range
    pub fn get_mut_checked(&mut self, id: usize) -> Option<&mut T> {
        let id = self.index_before_head(id).then_some(id)?;
        self.elements.get_mut(id)
    }

    pub fn get_checked(&self, id: usize) -> Option<&T> {
        let id = self.index_before_head(id).then_some(id)?;
        self.elements.get(id)
    }

    pub fn get(&self, id: usize) -> Option<&T> {
        self.index_in_range(id)?;
        self.elements.get(id)
    }

    fn current_mut(&mut self) -> Option<&mut T> {
        self.get_mut(self.head_usize())
    }

    pub fn last_mut(&mut self) -> Option<&mut T> {
        if self.head > 0 {
            self.get_mut(self.head_usize() - 1)
        } else {
            None
        }
    }

    pub fn last(&self) -> Option<&T> {
        if self.head > 0 {
            self.get(self.head_usize() - 1)
        } else {
            None
        }
    }

    pub fn add(&mut self, el: T) -> Result<(), ()> {
        if self.head_usize() == N {
            return Err(());
        }

        *self.current_mut().ok_or(())? = el;
        self.head += 1;

        Ok(())
    }

    pub fn remove(&mut self) -> Option<&T> {
        if self.head() == 0 {
            return None;
        }

        self.head -= 1;
        self.get(self.head_usize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
    struct TestStruct {
        size: u8,
        quantity: u32,
    }

    #[test]
    fn test_new() {
        let new_vec = FixedSizeVector::<TestStruct, 3>::new();

        assert_eq!(new_vec.head(), 0u8);
        assert_eq!(new_vec.last(), None);
    }

    #[test]
    fn test_add() {
        let mut new_vec = FixedSizeVector::<TestStruct, 128>::new();

        let t1 = TestStruct {
            size: 11,
            quantity: 11,
        };

        let t2 = TestStruct {
            size: 22,
            quantity: 22,
        };

        let t3 = TestStruct {
            size: 33,
            quantity: 33,
        };

        assert!(
            new_vec.add(t1.clone()).is_ok()
                && new_vec.add(t2.clone()).is_ok()
                && new_vec.add(t3.clone()).is_ok(),
            "can add to vec"
        );

        assert_eq!(new_vec.head(), 3);
        assert_eq!(new_vec.last(), Some(&t3));
        assert_eq!(new_vec.get(128), None);
        assert_eq!(new_vec.get(127), Some(&TestStruct::default()));
        assert_eq!(new_vec.current_mut(), Some(&mut TestStruct::default()));

        let mut range = new_vec.indexes();
        assert_eq!(range.next(), Some(0));
        assert_eq!(range.next(), Some(1));
        assert_eq!(range.next(), Some(2));
        assert_eq!(range.next(), None);

        assert_eq!(new_vec.remove(), Some(&t3));
        assert_eq!(new_vec.remove(), Some(&t2));
        assert_eq!(new_vec.remove(), Some(&t1));
        assert_eq!(new_vec.remove(), None);
        //new_vec.remove(),
    }
}
