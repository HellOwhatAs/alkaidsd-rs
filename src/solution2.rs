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

impl<F, const N: usize, const I1: usize, const I2: usize> SlicesSplitWindowsIter<F, N, I1, I2>
where
    F: Fn(usize) -> usize,
{
    pub fn new(next: F, slices: [(usize, usize); N], window: usize) -> Result<Self, usize> {
        const { assert!(I1 < N && I2 < N) };
        let mut cnt = window;
        let mut current = slices[I2].0;
        while cnt > 0 {
            cnt -= 1;
            if current == slices[I2].1 {
                return Err(cnt);
            } else {
                current = (next)(current);
            }
        }
        Ok(Self {
            next,
            slices,
            current1: slices[I1].0,
            current2: current,
        })
    }

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
}

impl<F, const N: usize, const I1: usize, const I2: usize> Iterator
    for SlicesSplitWindowsIter<F, N, I1, I2>
where
    F: Fn(usize) -> usize,
{
    type Item = (usize, usize);
    fn next(&mut self) -> Option<Self::Item> {
        let (ret1, ret2) = (self.current1, self.current2);
        if self.first_exhausted() || self.second_exhausted() {
            None
        } else {
            (self.current1, self.current2) =
                ((self.next)(self.current1), (self.next)(self.current2));
            Some((ret1, ret2))
        }
    }
}

#[test]
fn test() {
    let slices = [(1, 3), (4, 4), (9, 13), (100, 101)];
    let w11 = |mut wit: SlicesSplitWindowsIter<fn(usize) -> usize, 4, 1, 1>| {
        println!("{:?}", (&mut wit).collect::<Vec<_>>());
        println!("<{:?}>", (wit.first_exhausted(), wit.second_exhausted()));
        match (wit.first_exhausted(), wit.second_exhausted()) {
            (true, true) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 2, 2>::from11(wit);
                w11(wit);
            },
            (true, false) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 2, 1>::from10(wit);
                w10(wit);
            },
            (false, true) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 1, 2>::from01(wit);
                w01(wit);
            },
            (false, false) => unreachable!(),
        }
    };
    let w10 = |mut wit: SlicesSplitWindowsIter<fn(usize) -> usize, 4, 1, 0>| {
        println!("{:?}", (&mut wit).collect::<Vec<_>>());
        println!("<{:?}>", (wit.first_exhausted(), wit.second_exhausted()));
        match (wit.first_exhausted(), wit.second_exhausted()) {
            (true, true) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 2, 1>::from11(wit);
                w21(wit);
            },
            (true, false) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 2, 0>::from10(wit);
                w20(wit);
            },
            (false, true) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 1, 1>::from01(wit);
                w11(wit);
            },
            (false, false) => unreachable!(),
        }
    };
    let w01 = |mut wit: SlicesSplitWindowsIter<fn(usize) -> usize, 4, 0, 1>| {
        println!("{:?}", (&mut wit).collect::<Vec<_>>());
        println!("<{:?}>", (wit.first_exhausted(), wit.second_exhausted()));
        match (wit.first_exhausted(), wit.second_exhausted()) {
            (true, true) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 1, 2>::from11(wit);
                w12(wit);
            }
            (true, false) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 1, 1>::from10(wit);
                w11(wit);
            }
            (false, true) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 0, 2>::from01(wit);
                w02(wit);
            }
            (false, false) => unreachable!(),
        }
    };
    let w00 = |mut wit: SlicesSplitWindowsIter<fn(usize) -> usize, 4, 0, 0>| {
        println!("{:?}", (&mut wit).collect::<Vec<_>>());
        println!("<{:?}>", (wit.first_exhausted(), wit.second_exhausted()));
        match (wit.first_exhausted(), wit.second_exhausted()) {
            (true, true) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 1, 1>::from11(wit);
                w11(wit);
            }
            (true, false) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 1, 0>::from10(wit);
                w10(wit);
            }
            (false, true) => {
                println!("{:?}", (wit.current1, wit.current2));
                let wit = SlicesSplitWindowsIter::<_, _, 0, 1>::from01(wit);
                w01(wit);
            }
            (false, false) => unreachable!(),
        }
    };
    match SlicesSplitWindowsIter::<_, _, 0, 0>::new(|x| x + 1, slices, 3) {
        Ok(mut wit) => {
            println!("{:?}", (&mut wit).collect::<Vec<_>>());
            println!("<{:?}>", (wit.first_exhausted(), wit.second_exhausted()));
            match (wit.first_exhausted(), wit.second_exhausted()) {
                (true, true) => {
                    println!("{:?}", (wit.current1, wit.current2));
                    let mut wit = SlicesSplitWindowsIter::<_, _, 1, 1>::from11(wit);
                    println!("{:?}", (&mut wit).collect::<Vec<_>>());
                }
                (true, false) => {
                    println!("{:?}", (wit.current1, wit.current2));
                    let mut wit = SlicesSplitWindowsIter::<_, _, 1, 0>::from10(wit);
                    println!("{:?}", (&mut wit).collect::<Vec<_>>());
                }
                (false, true) => {
                    println!("{:?}", (wit.current1, wit.current2));
                    let mut wit = SlicesSplitWindowsIter::<_, _, 0, 1>::from01(wit);
                    println!("{:?}", (&mut wit).collect::<Vec<_>>());
                    println!("<{:?}>", (wit.first_exhausted(), wit.second_exhausted()));
                    match (wit.first_exhausted(), wit.second_exhausted()) {
                        (true, true) => {
                            println!("{:?}", (wit.current1, wit.current2));
                            let mut wit = SlicesSplitWindowsIter::<_, _, 1, 2>::from11(wit);
                            println!("{:?}", (&mut wit).collect::<Vec<_>>());
                        }
                        (true, false) => {
                            println!("{:?}", (wit.current1, wit.current2));
                            let mut wit = SlicesSplitWindowsIter::<_, _, 1, 1>::from10(wit);
                            println!("{:?}", (&mut wit).collect::<Vec<_>>());
                            println!("<{:?}>", (wit.first_exhausted(), wit.second_exhausted()));
                            match (wit.first_exhausted(), wit.second_exhausted()) {
                                (true, true) => {
                                    println!("{:?}", (wit.current1, wit.current2));
                                    let mut wit = SlicesSplitWindowsIter::<_, _, 2, 2>::from11(wit);
                                    println!("{:?}", (&mut wit).collect::<Vec<_>>());
                                }
                                (true, false) => {
                                    println!("{:?}", (wit.current1, wit.current2));
                                    let mut wit = SlicesSplitWindowsIter::<_, _, 2, 1>::from10(wit);
                                    println!("{:?}", (&mut wit).collect::<Vec<_>>());
                                }
                                (false, true) => {
                                    println!("{:?}", (wit.current1, wit.current2));
                                    let mut wit = SlicesSplitWindowsIter::<_, _, 1, 2>::from01(wit);
                                    println!("{:?}", (&mut wit).collect::<Vec<_>>());
                                    println!(
                                        "<{:?}>",
                                        (wit.first_exhausted(), wit.second_exhausted())
                                    );
                                    match (wit.first_exhausted(), wit.second_exhausted()) {
                                        (true, true) => {
                                            println!("{:?}", (wit.current1, wit.current2));
                                        }
                                        (true, false) => {
                                            println!("{:?}", (wit.current1, wit.current2));
                                            let mut wit =
                                                SlicesSplitWindowsIter::<_, _, 2, 2>::from10(wit);
                                            println!("{:?}", (&mut wit).collect::<Vec<_>>());
                                        }
                                        (false, true) => {
                                            println!("{:?}", (wit.current1, wit.current2));
                                        }
                                        (false, false) => unreachable!(),
                                    }
                                }
                                (false, false) => unreachable!(),
                            }
                        }
                        (false, true) => {
                            println!("{:?}", (wit.current1, wit.current2));
                            let mut wit = SlicesSplitWindowsIter::<_, _, 0, 2>::from01(wit);
                            println!("{:?}", (&mut wit).collect::<Vec<_>>());
                        }
                        (false, false) => unreachable!(),
                    }
                }
                (false, false) => unreachable!(),
            }
        }
        Err(cnt) => match SlicesSplitWindowsIter::<_, _, 0, 1>::new(|x| x + 1, slices, cnt) {
            Ok(mut wit) => {
                println!("{:?}", (&mut wit).collect::<Vec<_>>());
                println!("<{:?}>", (wit.first_exhausted(), wit.second_exhausted()));
                match (wit.first_exhausted(), wit.second_exhausted()) {
                    (true, true) => {
                        println!("{:?}", (wit.current1, wit.current2));
                        let mut wit = SlicesSplitWindowsIter::<_, _, 1, 2>::from11(wit);
                        println!("{:?}", (&mut wit).collect::<Vec<_>>());
                    }
                    (true, false) => {
                        println!("{:?}", (wit.current1, wit.current2));
                        let mut wit = SlicesSplitWindowsIter::<_, _, 1, 1>::from10(wit);
                        println!("{:?}", (&mut wit).collect::<Vec<_>>());
                        println!("<{:?}>", (wit.first_exhausted(), wit.second_exhausted()));
                        match (wit.first_exhausted(), wit.second_exhausted()) {
                            (true, true) => {
                                println!("{:?}", (wit.current1, wit.current2));
                                let mut wit = SlicesSplitWindowsIter::<_, _, 2, 2>::from11(wit);
                                println!("{:?}", (&mut wit).collect::<Vec<_>>());
                            }
                            (true, false) => {
                                println!("{:?}", (wit.current1, wit.current2));
                                let mut wit = SlicesSplitWindowsIter::<_, _, 2, 1>::from10(wit);
                                println!("{:?}", (&mut wit).collect::<Vec<_>>());
                            }
                            (false, true) => {
                                println!("{:?}", (wit.current1, wit.current2));
                                let mut wit = SlicesSplitWindowsIter::<_, _, 1, 2>::from01(wit);
                                println!("{:?}", (&mut wit).collect::<Vec<_>>());
                                println!("<{:?}>", (wit.first_exhausted(), wit.second_exhausted()));
                                match (wit.first_exhausted(), wit.second_exhausted()) {
                                    (true, true) => {
                                        println!("{:?}", (wit.current1, wit.current2));
                                    }
                                    (true, false) => {
                                        println!("{:?}", (wit.current1, wit.current2));
                                        let mut wit =
                                            SlicesSplitWindowsIter::<_, _, 2, 2>::from10(wit);
                                        println!("{:?}", (&mut wit).collect::<Vec<_>>());
                                    }
                                    (false, true) => {
                                        println!("{:?}", (wit.current1, wit.current2));
                                    }
                                    (false, false) => unreachable!(),
                                }
                            }
                            (false, false) => unreachable!(),
                        }
                    }
                    (false, true) => {
                        println!("{:?}", (wit.current1, wit.current2));
                        let mut wit = SlicesSplitWindowsIter::<_, _, 0, 2>::from01(wit);
                        println!("{:?}", (&mut wit).collect::<Vec<_>>());
                        println!("<{:?}>", (wit.first_exhausted(), wit.second_exhausted()));
                        match (wit.first_exhausted(), wit.second_exhausted()) {
                            (true, true) => {
                                println!("{:?}", (wit.current1, wit.current2));
                            }
                            (true, false) => {
                                println!("{:?}", (wit.current1, wit.current2));
                                let mut wit = SlicesSplitWindowsIter::<_, _, 1, 2>::from10(wit);
                                println!("{:?}", (&mut wit).collect::<Vec<_>>());
                                println!("<{:?}>", (wit.first_exhausted(), wit.second_exhausted()));
                                match (wit.first_exhausted(), wit.second_exhausted()) {
                                    (true, true) => {
                                        println!("{:?}", (wit.current1, wit.current2));
                                    }
                                    (true, false) => {
                                        println!("{:?}", (wit.current1, wit.current2));
                                        let mut wit =
                                            SlicesSplitWindowsIter::<_, _, 2, 2>::from10(wit);
                                        println!("{:?}", (&mut wit).collect::<Vec<_>>());
                                        match (wit.first_exhausted(), wit.second_exhausted()) {
                                            (true, true) => {
                                                println!("{:?}", (wit.current1, wit.current2));
                                            }
                                            (true, false) => {
                                                println!("{:?}", (wit.current1, wit.current2));
                                                let mut wit =
                                                    SlicesSplitWindowsIter::<_, _, 3, 2>::from10(
                                                        wit,
                                                    );
                                                println!("{:?}", (&mut wit).collect::<Vec<_>>());
                                            }
                                            (false, true) => {
                                                println!("{:?}", (wit.current1, wit.current2));
                                            }
                                            (false, false) => unreachable!(),
                                        }
                                    }
                                    (false, true) => {
                                        println!("{:?}", (wit.current1, wit.current2));
                                    }
                                    (false, false) => unreachable!(),
                                }
                            }
                            (false, true) => {
                                println!("{:?}", (wit.current1, wit.current2));
                            }
                            (false, false) => unreachable!(),
                        }
                    }
                    (false, false) => unreachable!(),
                }
            }
            Err(cnt) => match SlicesSplitWindowsIter::<_, _, 0, 2>::new(|x| x + 1, slices, cnt) {
                Ok(mut wit) => {
                    println!("{:?}", (&mut wit).collect::<Vec<_>>());
                    println!("<{:?}>", (wit.first_exhausted(), wit.second_exhausted()));
                    match (wit.first_exhausted(), wit.second_exhausted()) {
                        (true, true) => {
                            println!("{:?}", (wit.current1, wit.current2));
                        }
                        (true, false) => {
                            println!("{:?}", (wit.current1, wit.current2));
                            let mut wit = SlicesSplitWindowsIter::<_, _, 1, 2>::from10(wit);
                            println!("{:?}", (&mut wit).collect::<Vec<_>>());
                        }
                        (false, true) => {
                            println!("{:?}", (wit.current1, wit.current2));
                        }
                        (false, false) => unreachable!(),
                    }
                }
                _ => {}
            },
        },
    }
}

#[test]
fn cross() {
    let list = LinkedList::<usize>::new();
    let (route_0, route_1) = ((1, 2), (3, 4));
    // route_0 CASE 1
    {
        let route_3 = route_0;
        // route_1 CASE 1
        {
            let route_5 = route_1;
            // Final
            let (route_6, route_7) = (route_5, route_3);
        }
        // route_1 CASE 2
        {
            for split in FuncIter::new(|x| list.successor(x), route_1.0, route_1.1) {
                let (route_4, route_5) = ((route_1.0, split), (list.successor(split), route_1.1));
                // Final
                let (route_6, route_7) = (route_5, [route_3, route_4]);
            }
        }
        // route_1 CASE 3
        {
            let route_4 = route_1;
            // Final
            let route_7 = [route_3, route_4];
        }
    }
    // route_0 CASE 2
    {
        for split in FuncIter::new(|x| list.successor(x), route_0.0, route_0.1) {
            let (route_2, route_3) = ((route_0.0, split), (list.successor(split), route_0.1));
            // route_1 CASE 1
            {
                let route_5 = route_1;
                // Final
                let (route_6, route_7) = ([route_2, route_5], route_3);
            }
            // route_1 CASE 2
            {
                for split in FuncIter::new(|x| list.successor(x), route_1.0, route_1.1) {
                    let (route_4, route_5) =
                        ((route_1.0, split), (list.successor(split), route_1.1));
                    // Final
                    let (route_6, route_7) = ([route_2, route_5], [route_3, route_4]);
                }
            }
            // route_1 CASE 3
            {
                let route_4 = route_1;
                // Final
                let (route_6, route_7) = (route_2, [route_3, route_4]);
            }
        }
    }
    // route_0 CASE 3
    {
        let route_2 = route_0;
        // route_1 CASE 1
        {
            let route_5 = route_1;
            // Final
            let route_6 = [route_2, route_5];
        }
        // route_1 CASE 2
        {
            for split in FuncIter::new(|x| list.successor(x), route_1.0, route_1.1) {
                let (route_4, route_5) = ((route_1.0, split), (list.successor(split), route_1.1));
                // Final
                let (route_6, route_7) = ([route_2, route_5], route_4);
            }
        }
        // route_1 CASE 3
        {
            let route_4 = route_1;
            // Final
            let (route_6, route_7) = (route_2, route_4);
        }
    }
}

#[test]
fn swap() {
    for _ in 0..1 {
        let (m, n) = (3, 3);
        let list = LinkedList::<usize>::new();
        let (route_0, route_1) = ((1, 2), (3, 4));
        // route_0 prepare 1
        let mut cnt = m - 1;
        let mut split2 = route_0.0;
        if split2 == route_0.1 {
            continue;
        }
        while cnt > 0 {
            let succ = list.successor(split2);
            if succ == route_0.1 {
                break;
            }
            split2 = succ;
            cnt -= 1;
        }
        if cnt != 0 {
            continue;
        }
        // route_0 CASE 1
        {
            let (route_3, route_4) = ((route_0.0, split2), (list.successor(split2), route_0.1));

            ////////////////////////////////
            // route_1 prepare 1
            let mut cnt = n - 1;
            let mut split2 = route_1.0;
            if split2 == route_1.1 {
                continue;
            }
            while cnt > 0 {
                let succ = list.successor(split2);
                if succ == route_1.1 {
                    break;
                }
                split2 = succ;
                cnt -= 1;
            }
            if cnt != 0 {
                continue;
            }
            // route_1 CASE 1
            {
                let (route_6, route_7) = ((route_1.0, split2), (list.successor(split2), route_1.1));
                // Final
                let route_9 = [route_6, route_4];
                let route_11 = [route_3, route_7];
            }
            // route_1 CASE 2
            let mut split1 = route_1.0;
            {
                loop {
                    let succ = list.successor(split2);
                    if succ == route_1.1 {
                        break;
                    }
                    split2 = succ;
                    let (route_5, route_6, route_7) = (
                        (route_1.0, split1),
                        (list.successor(split1), split2),
                        (list.successor(split2), route_1.1),
                    );
                    split1 = list.successor(split1);
                    // Final
                    let route_9 = [route_6, route_4];
                    let route_11 = [route_5, route_3, route_7];
                }
            }
            // route_1 CASE 3
            {
                split1 = list.successor(split1);
                let (route_5, route_6) = ((route_0.0, split1), (list.successor(split1), route_0.1));
                // Final
                let route_9 = [route_6, route_4];
                let route_11 = [route_5, route_3];
            }
            ///////////////////////////////
        }
        // route_0 CASE 2
        let mut split1 = route_0.0;
        {
            loop {
                let succ = list.successor(split2);
                if succ == route_0.1 {
                    break;
                }
                split2 = succ;
                let (route_2, route_3, route_4) = (
                    (route_0.0, split1),
                    (list.successor(split1), split2),
                    (list.successor(split2), route_0.1),
                );
                split1 = list.successor(split1);
                ////////////////////////////////
                // route_1 prepare 1
                let mut cnt = n - 1;
                let mut split2 = route_1.0;
                if split2 == route_1.1 {
                    continue;
                }
                while cnt > 0 {
                    let succ = list.successor(split2);
                    if succ == route_1.1 {
                        break;
                    }
                    split2 = succ;
                    cnt -= 1;
                }
                if cnt != 0 {
                    continue;
                }
                // route_1 CASE 1
                {
                    let (route_6, route_7) =
                        ((route_1.0, split2), (list.successor(split2), route_1.1));
                    // Final
                    let route_9 = [route_2, route_6, route_4];
                    let route_11 = [route_3, route_7];
                }
                // route_1 CASE 2
                let mut split1 = route_1.0;
                {
                    loop {
                        let succ = list.successor(split2);
                        if succ == route_1.1 {
                            break;
                        }
                        split2 = succ;
                        let (route_5, route_6, route_7) = (
                            (route_1.0, split1),
                            (list.successor(split1), split2),
                            (list.successor(split2), route_1.1),
                        );
                        split1 = list.successor(split1);
                        // Final
                        let route_9 = [route_2, route_6, route_4];
                        let route_11 = [route_5, route_3, route_7];
                    }
                }
                // route_1 CASE 3
                {
                    split1 = list.successor(split1);
                    let (route_5, route_6) =
                        ((route_0.0, split1), (list.successor(split1), route_0.1));
                    // Final
                    let route_9 = [route_2, route_6, route_4];
                    let route_11 = [route_5, route_3];
                }
                ///////////////////////////////
            }
        }
        // route_0 CASE 3
        {
            split1 = list.successor(split1);
            let (route_2, route_3) = ((route_0.0, split1), (list.successor(split1), route_0.1));
            ////////////////////////////////
            // route_1 prepare 1
            let mut cnt = n - 1;
            let mut split2 = route_1.0;
            if split2 == route_1.1 {
                continue;
            }
            while cnt > 0 {
                let succ = list.successor(split2);
                if succ == route_1.1 {
                    break;
                }
                split2 = succ;
                cnt -= 1;
            }
            if cnt != 0 {
                continue;
            }
            // route_1 CASE 1
            {
                let (route_6, route_7) = ((route_1.0, split2), (list.successor(split2), route_1.1));
                // Final
                let route_9 = [route_2, route_6];
                let route_11 = [route_3, route_7];
            }
            // route_1 CASE 2
            let mut split1 = route_1.0;
            {
                loop {
                    let succ = list.successor(split2);
                    if succ == route_1.1 {
                        break;
                    }
                    split2 = succ;
                    let (route_5, route_6, route_7) = (
                        (route_1.0, split1),
                        (list.successor(split1), split2),
                        (list.successor(split2), route_1.1),
                    );
                    split1 = list.successor(split1);
                    // Final
                    let route_9 = [route_2, route_6];
                    let route_11 = [route_5, route_3, route_7];
                }
            }
            // route_1 CASE 3
            {
                split1 = list.successor(split1);
                let (route_5, route_6) = ((route_0.0, split1), (list.successor(split1), route_0.1));
                // Final
                let route_9 = [route_2, route_6];
                let route_11 = [route_5, route_3];
            }
            ///////////////////////////////
        }
    }
}
