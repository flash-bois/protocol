#[cfg(test)]
mod tests {
    use checked_decimal_macro::num_traits::ToPrimitive;
    use std::{
        ops::Range,
        slice::{Iter, IterMut},
    };
    use vec_macro::fixed_vector;

    #[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
    struct TestStruct {
        size: u8,
        quantity: u32,
    }

    #[fixed_vector(TestStruct, 128)]
    struct TestVec {
        head: u8,
        elements: [TestStruct; 128],
    }

    #[test]
    fn test_new() {
        let new_vec = TestVec::default();

        assert_eq!(new_vec.head, 0u8);
        assert_eq!(new_vec.last(), None);
    }

    #[test]
    fn test_add() {
        let mut new_vec = TestVec::default();

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

        assert_eq!(new_vec.head, 3);
        assert_eq!(new_vec.last(), Some(&t3));
        assert_eq!(new_vec.get(128), None);
        assert_eq!(new_vec.get(127), Some(&TestStruct::default()));
        assert_eq!(
            new_vec.get(new_vec.head_usize()),
            Some(&TestStruct::default())
        );

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
