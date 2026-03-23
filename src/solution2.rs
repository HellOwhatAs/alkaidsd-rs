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
    exhausted: bool,
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
            exhausted: false,
        }
    }
}

impl<F> Iterator for FuncIter<F>
where
    F: Fn(usize) -> usize,
{
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }
        let ret = self.current;
        if self.current == self.end {
            self.exhausted = true;
        } else {
            self.current = (self.func)(self.current);
        }
        Some(ret)
    }
}

pub fn dist(_from: usize, _to: usize) -> i32 {
    0
}

pub fn route_split_iter<'a>(
    route: (usize, usize),
    pred: impl Fn(usize) -> usize + Copy + 'a,
    succ: impl Fn(usize) -> usize + Copy + 'a,
) -> impl Iterator<Item = [(usize, usize); 2]> + 'a {
    let (head, tail) = route;
    (head != 0 && tail != 0)
        .then(|| {
            std::iter::once([(0, pred(head)), (head, tail)]).chain(
                FuncIter::new(succ, head, tail).map(move |node| {
                    [
                        (head, node),
                        (succ(node), if node == tail { 0 } else { tail }),
                    ]
                }),
            )
        })
        .into_iter()
        .flatten()
}

pub fn route_split_windows_iter<'a>(
    route: (usize, usize),
    window: usize,
    pred: impl Fn(usize) -> usize + Copy + 'a,
    succ: impl Fn(usize) -> usize + Copy + 'a,
) -> impl Iterator<Item = [(usize, usize); 3]> + 'a {
    let (head, tail) = route;
    (head != 0 && tail != 0)
        .then(|| {
            (std::iter::once((pred(head), head))
                .chain(FuncIter::new(succ, head, tail).map(move |node| (node, succ(node)))))
            .zip(
                (std::iter::once((pred(head), head))
                    .chain(FuncIter::new(succ, head, tail).map(move |node| (node, succ(node)))))
                .skip(window),
            )
            .map(move |(e1, e2)| {
                [
                    (if e1.1 == head { 0 } else { head }, e1.0),
                    (e1.1, e2.0),
                    (e2.1, if e2.0 == tail { 0 } else { tail }),
                ]
            })
        })
        .into_iter()
        .flatten()
}

pub fn route_join<'a>(
    route1: (usize, usize),
    route2: (usize, usize),
    pred: impl Fn(usize) -> usize + Copy + 'a,
    succ: impl Fn(usize) -> usize + Copy + 'a,
) -> (
    (usize, usize),
    impl Fn(usize) -> usize + Copy + 'a,
    impl Fn(usize) -> usize + Copy + 'a,
) {
    let valid1 = route1.0 != 0 && route1.1 != 0;
    let valid2 = route2.0 != 0 && route2.1 != 0;
    let valid = valid1 && valid2;

    let new_route = match (valid1, valid2) {
        (true, true) => (route1.0, route2.1),
        (true, false) => route1,
        (false, true) => route2,
        (false, false) => (0, 0),
    };

    (
        new_route,
        move |x| {
            (valid && x == route2.0)
                .then_some(route1.1)
                .unwrap_or_else(|| pred(x))
        },
        move |x| {
            (valid && x == route1.1)
                .then_some(route2.0)
                .unwrap_or_else(|| succ(x))
        },
    )
}

#[test]
fn test() {
    let mut list = LinkedList::<usize>::new();
    for arr in [vec![1, 3, 5, 7, 9], vec![2, 4, 6, 8]] {
        let mut pred = 0;
        for i in arr.into_iter() {
            pred = list.insert(i, pred, 0);
        }
    }
    let routes = [(1, 5), (6, 9)];

    let [r1, r2] = routes;

    let delta = 0;
    let pred = |x| list.predecessor(x);
    let succ = |x| list.successor(x);

    // CASE 1
    for [r11, a, r12] in route_split_windows_iter(r1, 1, pred, succ) {
        let (r1_, pred, succ) = route_join(r11, r12, pred, succ);
        for [r21, b, r22] in route_split_windows_iter(r2, 1, pred, succ) {
            let (r2_, pred, succ) = route_join(r21, r22, pred, succ);
            for [r11_, r12_] in route_split_iter(r1_, pred, succ) {
                let (res1, pred, succ) = route_join(r11_, b, pred, succ);
                let (res1, pred, succ) = route_join(res1, r12_, pred, succ);
                for [r21_, r22_] in route_split_iter(r2_, pred, succ) {
                    let (res2, pred, succ) = route_join(r21_, a, pred, succ);
                    let (res2, pred, succ) = route_join(res2, r22_, pred, succ);
                }
            }
        }
    }

    // CASE 2
    for (edge1, edge2) in (std::iter::once((pred(r1.0), r1.0))
        .chain(FuncIter::new(succ, r1.0, r1.1).map(|node| (node, succ(node)))))
    .zip(
        std::iter::once((pred(r1.0), r1.0))
            .chain(FuncIter::new(succ, r1.0, r1.1).map(|node| (node, succ(node))))
            .skip(1),
    ) {
        let delta = delta - dist(edge1.0, edge1.1) - dist(edge2.0, edge2.1);
        let (r11, a, r12) = ((r1.0, edge1.0), (edge1.1, edge2.0), (edge2.1, r1.1));
        let pred = |x| (x == edge2.1).then_some(edge1.0).unwrap_or_else(|| pred(x));
        let succ = |x| (x == edge1.0).then_some(edge2.1).unwrap_or_else(|| succ(x));
        let delta = delta + dist(edge1.0, edge2.1);
        for (edge3, edge4) in std::iter::once((pred(r2.0), r2.0))
            .chain(FuncIter::new(succ, r2.0, r2.1).map(|node| (node, succ(node))))
            .zip(
                std::iter::once((pred(r2.0), r2.0))
                    .chain(FuncIter::new(succ, r2.0, r2.1).map(|node| (node, succ(node))))
                    .skip(1),
            )
        {
            let delta = delta - dist(edge3.0, edge3.1) - dist(edge4.0, edge4.1);
            let (r21, b, r22) = ((r2.0, edge3.0), (edge3.1, edge4.0), (edge4.1, r2.1));
            let pred = |x| (x == edge4.1).then_some(edge3.0).unwrap_or_else(|| pred(x));
            let succ = |x| (x == edge3.0).then_some(edge4.1).unwrap_or_else(|| succ(x));
            let delta = delta + dist(edge3.0, edge4.1);
            for edge5 in std::iter::once((pred(r11.0), r11.0))
                .chain(FuncIter::new(succ, r11.0, r12.1).map(|node| (node, succ(node))))
            {
                let delta = delta - dist(edge5.0, edge5.1);
                let (r11_, r12_) = ((r11.0, edge5.0), (edge5.1, r12.1));
                for edge6 in std::iter::once((pred(r21.0), r21.0))
                    .chain(FuncIter::new(succ, r21.0, r22.1).map(|node| (node, succ(node))))
                {
                    let delta = delta - dist(edge6.0, edge6.1);
                    let (r21_, r22_) = ((r21.0, edge6.0), (edge6.1, r22.1));
                    let delta = delta + dist(r11_.1, b.0) + dist(b.1, r12_.0);
                    let delta = delta + dist(r21_.1, a.0) + dist(a.1, r22_.0);
                }
            }
        }
    }
}
