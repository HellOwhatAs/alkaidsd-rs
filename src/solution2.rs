#[derive(Debug, Clone, Default)]
struct NodeData<T> {
    succ: usize,
    pred: usize,
    data: T,
    index_in_used_nodes: usize,
}

#[derive(Debug, Clone)]
pub struct LinkedList<T> {
    node_data: Vec<NodeData<T>>,
    used_nodes: Vec<usize>,
    unused_nodes: Vec<usize>,
}

impl<T: Default> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Default> LinkedList<T> {
    pub fn new() -> Self {
        let mut solution = Self {
            node_data: Vec::new(),
            used_nodes: Vec::new(),
            unused_nodes: Vec::new(),
        };
        // Add depot node
        solution.node_data.push(NodeData::default());
        solution
    }
}

impl<T> LinkedList<T> {
    pub fn data(&self, node: usize) -> &T {
        unsafe { &self.node_data.get_unchecked(node).data }
    }

    pub fn data_mut(&mut self, node: usize) -> &mut T {
        unsafe { &mut self.node_data.get_unchecked_mut(node).data }
    }

    pub fn new_node(&mut self, data: T) -> usize {
        let new_data = NodeData {
            succ: 0,
            pred: 0,
            data,
            index_in_used_nodes: self.used_nodes.len(),
        };
        let node = if let Some(idx) = self.unused_nodes.pop() {
            self.node_data[idx] = new_data;
            idx
        } else {
            let idx = self.node_data.len();
            self.node_data.push(new_data);
            idx
        };
        self.used_nodes.push(node);
        node
    }

    pub fn nodes(&self) -> &[usize] {
        &self.used_nodes
    }

    pub fn max_node_index(&self) -> usize {
        self.node_data.len() - 1
    }

    pub fn predecessor(&self, node: usize) -> usize {
        unsafe { self.node_data.get_unchecked(node).pred }
    }

    pub fn successor(&self, node: usize) -> usize {
        unsafe { self.node_data.get_unchecked(node).succ }
    }

    pub fn set_predecessor(&mut self, node: usize, predecessor: usize) {
        unsafe { self.node_data.get_unchecked_mut(node).pred = predecessor };
    }

    pub fn set_successor(&mut self, node: usize, successor: usize) {
        unsafe { self.node_data.get_unchecked_mut(node).succ = successor };
    }

    pub fn link(&mut self, predecessor: usize, successor: usize) {
        self.set_predecessor(successor, predecessor);
        self.set_successor(predecessor, successor);
    }

    pub fn remove(&mut self, node: usize) {
        let predecessor = self.predecessor(node);
        let successor = self.successor(node);
        self.link(predecessor, successor);

        // Move to unused pool using swap-remove for O(1)
        let index_in_used = self.node_data[node].index_in_used_nodes;
        self.unused_nodes
            .push(self.used_nodes.swap_remove(index_in_used));
        // Update index_in_used_nodes
        if let Some(&swapped_node) = self.used_nodes.get(index_in_used) {
            unsafe {
                self.node_data
                    .get_unchecked_mut(swapped_node)
                    .index_in_used_nodes = index_in_used
            };
        }
    }

    pub fn insert(&mut self, data: T, predecessor: usize, successor: usize) -> usize {
        let node = self.new_node(data);
        self.link(predecessor, node);
        self.link(node, successor);
        node
    }

    pub fn reversed_link(
        &mut self,
        left: usize,
        right: usize,
        predecessor: usize,
        successor: usize,
    ) {
        let mut current = right;
        let mut new_pred = predecessor;

        loop {
            let original_predecessor = self.predecessor(current);
            self.link(new_pred, current);
            if current == left {
                break;
            }
            new_pred = current;
            current = original_predecessor;
        }
        self.link(left, successor);
    }
}

pub struct FuncIter<F> {
    func: F,
    current: usize,
    end: usize,
}

impl<F> FuncIter<F>
where
    F: Fn(usize) -> usize,
{
    pub fn new(func: F, start: usize, end: usize) -> Self {
        FuncIter {
            func,
            current: start,
            end,
        }
    }
}

impl<F> Iterator for FuncIter<F>
where
    F: Fn(usize) -> usize,
{
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.current;
        if self.current == self.end {
            None
        } else {
            self.current = (self.func)(self.current);
            Some(ret)
        }
    }
}

pub struct SlicesSplitIter<F, const N: usize, const I: usize> {
    next: F,
    slices: [(usize, usize); N],
    current: usize,
}

impl<F, const N: usize, const I: usize> SlicesSplitIter<F, N, I>
where
    F: Fn(usize) -> usize,
{
    pub const I: usize = I;

    pub fn new(next: F, slices: [(usize, usize); N]) -> Self {
        Self {
            next,
            slices,
            current: slices[I].0,
        }
    }

    pub fn to_index<const I1: usize>(self) -> SlicesSplitIter<F, N, I1> {
        SlicesSplitIter {
            next: self.next,
            slices: self.slices,
            current: self.slices[I1].0,
        }
    }
}

impl<F, const N: usize, const I: usize> Iterator for SlicesSplitIter<F, N, I>
where
    F: Fn(usize) -> usize,
{
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.current;
        if self.current == self.slices[I].1 {
            None
        } else {
            self.current = (self.next)(self.current);
            Some(ret)
        }
    }
}

pub struct SlicesSplitWindowsIter<F, const N: usize, const I1: usize, const I2: usize> {
    next: F,
    slices: [(usize, usize); N],
    pub current1: usize,
    pub current2: usize,
}

pub struct SlicesSplitWindowsIterBuilder<F, const N: usize, const I1: usize, const I2: usize> {
    next: F,
    slices: [(usize, usize); N],
    pub current1: usize,
    pub current2: usize,
}

impl<F, const N: usize, const I1: usize, const I2: usize>
    SlicesSplitWindowsIterBuilder<F, N, I1, I2>
where
    F: Fn(usize) -> usize,
{
    pub fn new(next: F, slices: [(usize, usize); N], window: usize) -> Result<Self, usize> {
        const { assert!(I1 < N && I2 < N) };
        let mut cnt = window;
        let mut current = slices[I2].0;
        let mut current1 = current;
        while cnt > 0 {
            cnt -= 1;
            if current == slices[I2].1 {
                return Err(cnt);
            } else {
                current1 = current;
                current = (next)(current);
            }
        }
        Ok(Self {
            next,
            slices,
            current1,
            current2: current,
        })
    }

    pub fn build(self) -> SlicesSplitWindowsIter<F, N, I1, I2> {
        SlicesSplitWindowsIter {
            next: self.next,
            slices: self.slices,
            current1: self.slices[I1].0,
            current2: self.current2,
        }
    }
}

impl<F, const N: usize, const I1: usize, const I2: usize> SlicesSplitWindowsIter<F, N, I1, I2>
where
    F: Fn(usize) -> usize,
{
    pub fn from01<const I3: usize>(other: SlicesSplitWindowsIter<F, N, I1, I3>) -> Self {
        const { assert!(I1 < N && I2 < N && I3 < N) };
        const { assert!(I3 + 1 == I2) };
        let current1 = (other.next)(other.current1);
        Self {
            next: other.next,
            slices: other.slices,
            current1,
            current2: other.slices[I2].0,
        }
    }

    pub fn from10<const I3: usize>(other: SlicesSplitWindowsIter<F, N, I3, I2>) -> Self {
        const { assert!(I1 < N && I2 < N && I3 < N) };
        const { assert!(I3 + 1 == I1) };
        let current2 = (other.next)(other.current2);
        Self {
            next: other.next,
            slices: other.slices,
            current1: other.slices[I1].0,
            current2,
        }
    }

    pub fn from11<const I3: usize, const I4: usize>(
        other: SlicesSplitWindowsIter<F, N, I3, I4>,
    ) -> Self {
        const { assert!(I1 < N && I2 < N && I3 < N && I4 < N) };
        const { assert!(I3 + 1 == I1) };
        const { assert!(I4 + 1 == I2) };
        Self {
            next: other.next,
            slices: other.slices,
            current1: other.slices[I1].0,
            current2: other.slices[I2].0,
        }
    }

    pub fn first_exhausted(&self) -> bool {
        self.current1 == self.slices[I1].1
    }

    pub fn second_exhausted(&self) -> bool {
        self.current2 == self.slices[I2].1
    }

    #[inline(always)]
    pub const fn i1(&self) -> usize {
        I1
    }

    #[inline(always)]
    pub const fn i2(&self) -> usize {
        I2
    }
}

impl<F, const N: usize, const I1: usize, const I2: usize> Iterator
    for SlicesSplitWindowsIter<F, N, I1, I2>
where
    F: Fn(usize) -> usize,
{
    type Item = ((usize, usize), (usize, usize));
    fn next(&mut self) -> Option<Self::Item> {
        let (ret1, ret2) = (self.current1, self.current2);
        if self.first_exhausted() || self.second_exhausted() {
            None
        } else {
            (self.current1, self.current2) =
                ((self.next)(self.current1), (self.next)(self.current2));
            Some(((ret1, self.current1), (ret2, self.current2)))
        }
    }
}

#[macro_export]
macro_rules! concat_arrays {
    ($(($arr:expr, $len:literal)),+ $(,)?) => {{
        const TOTAL: usize = 0usize $(+ $len)+;
        let mut out = ::std::mem::MaybeUninit::<[_; TOTAL]>::uninit();
        unsafe {
            #[inline(always)]
            unsafe fn copy_into<T, const D: usize, const S: usize>(
                dst: *mut [T; D], src: &[T; S], offset: usize,
            ) {
                ::std::ptr::copy_nonoverlapping(src.as_ptr(), (dst as *mut T).add(offset), S);
            }
            let mut offset = 0usize;
            $(
                copy_into(out.as_mut_ptr(), &{$arr}, offset);
                offset += $len;
            )+
            let _ = offset;
            out.assume_init()
        }
    }};
}

#[macro_export]
macro_rules! slice_array {
    ($arr:expr, $a:literal..$b:literal) => {{
        let arr = $arr;
        let mut result = ::std::mem::MaybeUninit::<[_; $b - $a]>::uninit();
        #[allow(unused_unsafe)]
        unsafe {
            ::std::ptr::copy_nonoverlapping(
                arr.as_ptr().add($a),
                ::std::ptr::addr_of_mut!((*result.as_mut_ptr())[0]),
                $b - $a,
            );
            result.assume_init()
        }
    }};
}

include!("../split_windows_test.rs");

#[test]
fn test() {
    let slices = [(1, 3), (7, 7), (9, 13), (100, 101)];
    let succ = |x: usize| x + 1;
    split_windows_4(succ, slices);
}
