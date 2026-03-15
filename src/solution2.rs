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
        debug_assert!(node < self.node_data.len());
        unsafe { &self.node_data.get_unchecked(node).data }
    }

    pub fn data_mut(&mut self, node: usize) -> &mut T {
        debug_assert!(node < self.node_data.len());
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
        debug_assert!(node < self.node_data.len());
        unsafe { self.node_data.get_unchecked(node).pred }
    }

    pub fn successor(&self, node: usize) -> usize {
        debug_assert!(node < self.node_data.len());
        unsafe { self.node_data.get_unchecked(node).succ }
    }

    pub fn set_predecessor(&mut self, node: usize, predecessor: usize) {
        debug_assert!(node < self.node_data.len());
        unsafe { self.node_data.get_unchecked_mut(node).pred = predecessor };
    }

    pub fn set_successor(&mut self, node: usize, successor: usize) {
        debug_assert!(node < self.node_data.len());
        unsafe { self.node_data.get_unchecked_mut(node).succ = successor };
    }

    pub fn link(&mut self, predecessor: usize, successor: usize) {
        self.set_predecessor(successor, predecessor);
        self.set_successor(predecessor, successor);
    }

    pub fn remove(&mut self, node: usize) {
        debug_assert!(node != 0);
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

pub struct RouteIter<'a, T> {
    data: &'a LinkedList<T>,
    cur: usize,
}

impl<'a, T> Iterator for RouteIter<'a, T> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        let res = self.cur;
        if res == 0 {
            None
        } else {
            self.cur = self.data.successor(self.cur);
            Some(res)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Route<'a, T> {
    context: &'a RouteContext<T>,
    index: usize,
}

impl<'a, T> Route<'a, T> {
    pub fn iter(&self) -> RouteIter<'a, T> {
        RouteIter {
            data: &self.context.data,
            cur: unsafe { self.context.routes.get_unchecked(self.index) }.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RouteContext<T> {
    data: LinkedList<T>,
    routes: Vec<(usize, usize)>,
}

impl<T> RouteContext<T> {
    pub fn iter(&self) -> impl Iterator<Item = Route<'_, T>> + Clone + ExactSizeIterator {
        (0..self.routes.len()).map(|index| Route {
            context: self,
            index,
        })
    }

    pub fn from_linkedlist(data: LinkedList<T>) -> Self {
        let mut routes = Vec::new();
        for &node in data.nodes() {
            if data.predecessor(node) == 0 {
                let mut tail = node;
                while data.successor(tail) != 0 {
                    tail = data.successor(tail);
                }
                routes.push((node, tail));
            }
        }
        Self { data, routes }
    }

    pub fn pred(&self, node: usize) -> Option<usize> {
        self.data
            .node_data
            .get(node)
            .map(|x| x.pred)
            .filter(|&x| x != 0)
    }

    pub fn succ(&self, node: usize) -> Option<usize> {
        self.data
            .node_data
            .get(node)
            .map(|x| x.succ)
            .filter(|&x| x != 0)
    }

    pub fn probe(&self) {}
}

#[test]
fn test() {
    use itertools::Itertools;

    let mut llist = LinkedList::<usize>::default();
    llist.insert(10, 0, 0);
    llist.insert(11, 0, 0);
    let ctx = RouteContext::from_linkedlist(llist);

    for (x, y) in ctx.iter().tuple_combinations() {
        println!("{:?}\n{:?}\n", x, y);
    }
}
