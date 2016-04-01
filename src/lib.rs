use std::ptr;

// Implementation heavily inspired by
// C++ Program to Implement Pairing Heap
// http://www.sanfoundry.com/cpp-program-implement-pairing-heap/

// TODO: make a max heap to match stdlib?

// TODO: lifetime tied to heap?
#[derive(Debug)]
pub struct Token<T>(*mut Node<T>);

impl<T> Copy for Token<T> {}
impl<T> Clone for Token<T> {
    fn clone(&self) -> Self { *self }
}

struct Node<T> {
    value: T,
    first_child: *mut Node<T>,
    prev: *mut Node<T>,
    next: *mut Node<T>,
}

impl<T> Node<T> {
    fn new(value: T) -> Self {
        Node {
            value: value,
            first_child: ptr::null_mut(),
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
        }
    }
}

pub struct Heap<T> {
    root: *mut Node<T>,
    combine_siblings: CombineSiblings<T>,
}

impl<T> Heap<T>
    where T: Ord,
{
    pub fn new() -> Heap<T> {
        Heap {
            root: ptr::null_mut(),
            combine_siblings: CombineSiblings::new(),
        }
    }

    pub fn push(&mut self, value: T) -> Token<T> {
        let node = Box::into_raw(Box::new(Node::new(value)));

        self.root = if self.root.is_null() {
            node
        } else {
            compare_and_link(self.root, node)
        };

        Token(node)
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.root.is_null() { return None }

        let root = unsafe { Box::from_raw(self.root) };

        self.root = if root.first_child.is_null() {
            ptr::null_mut()
        } else {
            self.combine_siblings.combine_siblings(root.first_child)
        };

        Some(root.value)
    }

    /// Do not increase the key!
    pub fn decrease_key<F>(&mut self, token: Token<T>, f: F)
        where F: FnOnce(&mut T),
    {
        let node = token.0;
        let node_r = unsafe { &mut *node };

        // Apply the change that decreases the key
        f(&mut node_r.value);

        if node == self.root { return }

        if let Some(p_next_r) = unsafe { into_mut(node_r.next) } {
            p_next_r.prev = node_r.prev;
        }

        let node_prev_r = unsafe { &mut *node_r.prev };

        if node_prev_r.first_child == node {
            node_prev_r.first_child = node_r.next;
        } else {
            node_prev_r.next = node_r.next;
        }

        node_r.next = ptr::null_mut();;
        self.root = compare_and_link(self.root, node);
    }
}

fn compare_and_link<T>(first: *mut Node<T>, second: *mut Node<T>) -> *mut Node<T>
    where T: Ord
{
    if second.is_null() { return first }

    let first_r = unsafe { &mut *first };
    let second_r = unsafe { &mut *second };

    if second_r.value < first_r.value {
        second_r.prev = first_r.prev;
        first_r.prev = second;
        first_r.next = second_r.first_child;
        if let Some(first_next_r) = unsafe { into_mut(first_r.next) } {
            first_next_r.prev = first;
        }
        second_r.first_child = first;
        second
    } else {
        second_r.prev = first;
        first_r.next = second_r.next;
        if let Some(first_next_r) = unsafe { into_mut(first_r.next) } {
            first_next_r.prev = first;
        }
        second_r.next = first_r.first_child;
        if let Some(second_next_r) = unsafe { into_mut(second_r.next) } {
            second_next_r.prev = second;
        }
        first_r.first_child = second;
        first
    }
}

unsafe fn into_mut<'a, T>(v: *mut T) -> Option<&'a mut T> {
    if v.is_null() {
        None
    } else {
        Some(&mut *v)
    }
}

struct CombineSiblings<T> {
    tree_array: Vec<*mut Node<T>>,
}

impl<T> CombineSiblings<T>
    where T: Ord
{
    fn new() -> Self {
        CombineSiblings {
            tree_array: Vec::with_capacity(5),
        }
    }

    fn combine_siblings(&mut self, mut first_sibling: *mut Node<T>) -> *mut Node<T> {
        let first_sibling_r = unsafe { &mut *first_sibling };

        if first_sibling_r.next.is_null() {
            return first_sibling;
        }

        self.tree_array.clear();

        while let Some(first_sibling_r) = unsafe { into_mut(first_sibling) } {
            self.tree_array.push(first_sibling);
            let first_sibling_prev_r = unsafe { &mut *first_sibling_r.prev };
            first_sibling_prev_r.next = ptr::null_mut();
            first_sibling = first_sibling_r.next;
        }

        // Pad with a NULL to ensure all siblings are in an even amount
        self.tree_array.push(ptr::null_mut());
        let logical_length = self.tree_array.len() / 2 * 2;
        let tree_array = &mut self.tree_array[0..logical_length];

        // Walk forward in pairs, leaving the result in the {0,2,4,...}
        // indexes. Then walk backward across the {[2,4], [0,2]} results,
        // leaving the result in the first index. The final result will be
        // in index 0.

        // 0     1  2   3  4  5
        // ----------------
        // 1     2  3   4  5
        // 1     2  3   4  5  N -- Padded with NULL
        // 12    X  3   4  5  N
        // 12    X  34  X  5  N
        // 12    X  34  X  5  X -- forward pass done
        // 12    X  345 X  X  X
        // 12345 X  X   X  X  X -- backward pass done

        for chunk in tree_array.chunks_mut(2) {
            chunk[0] = compare_and_link(chunk[0], chunk[1]);
        }

        if logical_length >= 4 {
            let mut end_idx = logical_length - 2;

            while end_idx >= 2  {
                let start_idx = end_idx - 2;
                tree_array[start_idx] = compare_and_link(tree_array[start_idx], tree_array[end_idx]);
                end_idx -= 2;
            }
        }

        tree_array[0]
    }
}

#[cfg(test)]
mod test {
    use Heap;

    #[test]
    fn empty_heap_pops_none() {
        let mut h = Heap::<u8>::new();
        assert_eq!(None, h.pop());
    }

    #[test]
    fn heap_with_one_value() {
        let mut h = Heap::new();
        h.push(1);
        assert_eq!(Some(1), h.pop());
        assert_eq!(None, h.pop());
    }

    #[test]
    fn multiple_values_inserted_in_order_returns_them_in_order() {
        let mut h = Heap::new();
        h.push(1);
        h.push(2);
        assert_eq!(Some(1), h.pop());
        assert_eq!(Some(2), h.pop());
        assert_eq!(None, h.pop());
    }

    #[test]
    fn multiple_values_inserted_out_of_order_returns_them_in_order() {
        let mut h = Heap::new();
        h.push(2);
        h.push(1);
        assert_eq!(Some(1), h.pop());
        assert_eq!(Some(2), h.pop());
        assert_eq!(None, h.pop());
    }

    #[test]
    fn duplicate_values_are_kept() {
        let mut h = Heap::new();

        for i in 0..5 { h.push(i); }
        for i in 0..5 { h.push(i); }

        for i in 0..5 {
            assert_eq!(Some(i), h.pop());
            assert_eq!(Some(i), h.pop());
        }
        assert_eq!(None, h.pop());
    }

    #[test]
    fn interleaved_push_and_pop() {
        let mut h = Heap::new();

        h.push(0);
        assert_eq!(Some(0), h.pop());

        h.push(1);
        h.push(1);
        assert_eq!(Some(1), h.pop());

        h.push(2);
        assert_eq!(Some(1), h.pop());
        assert_eq!(Some(2), h.pop());
        assert_eq!(None, h.pop());
    }

    #[test]
    fn many_values_inserted_returns_them_in_order() {
        let count = 123;

        let mut h = Heap::new();
        for i in 0..count {
            h.push(i);
        }

        for i in 0..count {
            assert_eq!(Some(i), h.pop());
        }
        assert_eq!(None, h.pop());
    }

    #[test]
    fn decreasing_a_key_brings_it_to_the_front() {
        let mut h = Heap::new();

        h.push(10);
        let t2 = h.push(20);

        h.decrease_key(t2, |v| *v = 5);

        assert_eq!(Some(5), h.pop());
        assert_eq!(Some(10), h.pop());
        assert_eq!(None, h.pop());
    }

    #[test]
    fn decreasing_a_key_of_many_brings_it_to_the_front() {
        let mut h = Heap::new();

        let mut t = h.push(10);
        for i in 11..100 {
            t = h.push(i);
        }

        h.decrease_key(t, |v| *v = 1);

        assert_eq!(Some(1), h.pop());

        for i in 10..99 {
            assert_eq!(Some(i), h.pop());
        }

        assert_eq!(None, h.pop());
    }
}
