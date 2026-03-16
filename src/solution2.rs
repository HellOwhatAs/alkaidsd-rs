use crate::Instance;
use itertools::Itertools;
use std::mem::swap;

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

pub struct RouteNodeIter<'a, T> {
    data: &'a LinkedList<T>,
    cur: usize,
}

impl<'a, T> Iterator for RouteNodeIter<'a, T> {
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

pub struct RouteEdgeIter<'a, T> {
    data: &'a LinkedList<T>,
    cur: Option<(usize, usize)>,
}

impl<'a, T> Iterator for RouteEdgeIter<'a, T> {
    type Item = (usize, usize);
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((a, b)) = self.cur {
            self.cur = (b != 0).then(|| (b, self.data.successor(b)));
            Some((a, b))
        } else {
            self.cur
        }
    }
}

impl<'a, T> DoubleEndedIterator for RouteEdgeIter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some((a, b)) = self.cur {
            self.cur = (a != 0).then(|| (self.data.predecessor(a), a));
            Some((a, b))
        } else {
            self.cur
        }
    }
}

#[derive(Debug, Clone)]
pub struct Route<'a, T> {
    context: &'a RouteContext<T>,
    index: usize,
}

impl<'a, T> Route<'a, T> {
    pub fn iter_nodes(&self) -> RouteNodeIter<'a, T> {
        RouteNodeIter {
            data: &self.context.data,
            cur: unsafe { self.context.routes.get_unchecked(self.index) }.0,
        }
    }

    pub fn iter_edges(&self) -> RouteEdgeIter<'a, T> {
        let head = unsafe { self.context.routes.get_unchecked(self.index) }.0;
        RouteEdgeIter {
            data: &self.context.data,
            cur: Some((0, head)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RouteSplitLeft<T> {
    route: T,
    tail: usize,
}

#[derive(Debug, Clone)]
pub struct RouteSplitRight<T> {
    route: T,
    head: usize,
}

#[derive(Debug, Clone)]
pub struct RouteJoin<T1, T2> {
    left: T1,
    right: T2,
}

#[derive(Debug, Clone)]
pub struct RouteReverse<T> {
    route: T,
}

pub trait RouteLike: Sized + Clone {
    fn head(&self) -> usize;
    fn tail(&self) -> usize;
}

impl<'a, T: Clone> RouteLike for Route<'a, T> {
    fn head(&self) -> usize {
        unsafe { self.context.routes.get_unchecked(self.index) }.0
    }

    fn tail(&self) -> usize {
        unsafe { self.context.routes.get_unchecked(self.index) }.1
    }
}
impl<T: RouteLike> RouteLike for RouteSplitLeft<T> {
    fn head(&self) -> usize {
        self.route.head()
    }

    fn tail(&self) -> usize {
        self.tail
    }
}
impl<T: RouteLike> RouteLike for RouteSplitRight<T> {
    fn head(&self) -> usize {
        self.head
    }

    fn tail(&self) -> usize {
        self.route.tail()
    }
}
impl<T1: RouteLike, T2: RouteLike> RouteLike for RouteJoin<T1, T2> {
    fn head(&self) -> usize {
        self.left.head()
    }

    fn tail(&self) -> usize {
        self.right.tail()
    }
}
impl<T: RouteLike> RouteLike for RouteReverse<T> {
    fn head(&self) -> usize {
        self.tail()
    }

    fn tail(&self) -> usize {
        self.head()
    }
}

#[derive(Debug, Clone)]
pub struct RouteContext<T> {
    pub data: LinkedList<T>,
    pub instance: Instance,
    routes: Vec<(usize, usize)>,
}

impl<T: Clone> RouteContext<T> {
    pub fn iter(&self) -> impl Iterator<Item = Route<'_, T>> + Clone + ExactSizeIterator {
        (0..self.routes.len()).map(|index| Route {
            context: self,
            index,
        })
    }

    pub fn new(data: LinkedList<T>, instance: Instance) -> Self {
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
        Self {
            data,
            instance,
            routes,
        }
    }

    pub fn probe<F, const N: usize>(&self, func: F) -> (i32, Vec<(usize, usize)>)
    where
        F: Fn(&mut RouteEditContext<'_, T>, [Route<'_, T>; N]),
    {
        let mut best_delta = 0;
        let (mut actions, mut best_actions) = (Vec::new(), Vec::new());
        for routes in self.iter().array_combinations() {
            actions.clear();
            let mut ctx = RouteEditContext {
                context: self,
                actions: &mut actions,
                delta: routes
                    .iter()
                    .map(|r| {
                        -self.instance.distance(0, r.head() as i16)
                            - self.instance.distance(r.tail() as i16, 0)
                    })
                    .sum(),
            };
            func(&mut ctx, routes);
            if ctx.delta < best_delta {
                best_delta = ctx.delta;
                swap(&mut actions, &mut best_actions);
            }
        }
        (best_delta, best_actions)
    }
}

#[derive(Debug)]
pub struct RouteEditContext<'a, T> {
    context: &'a RouteContext<T>,
    actions: &'a mut Vec<(usize, usize)>,
    delta: i32,
}

impl<'a, T> RouteEditContext<'a, T> {
    pub fn split_at<T1: RouteLike>(
        &mut self,
        route: T1,
        edge: (usize, usize),
    ) -> (RouteSplitLeft<T1>, RouteSplitRight<T1>) {
        self.delta -= self.context.instance.distance(edge.0 as i16, edge.1 as i16);
        (
            RouteSplitLeft {
                route: route.clone(),
                tail: edge.0,
            },
            RouteSplitRight {
                route: route,
                head: edge.1,
            },
        )
    }

    pub fn join<T1: RouteLike, T2: RouteLike>(
        &mut self,
        route1: T1,
        route2: T2,
    ) -> RouteJoin<T1, T2> {
        let (a, b) = (route1.tail(), route2.head());
        self.delta += self.context.instance.distance(a as i16, b as i16);
        self.actions.push((a, b));
        RouteJoin {
            left: route1,
            right: route2,
        }
    }

    pub fn rev<T1: RouteLike>(&mut self, route: T1) -> RouteReverse<T1> {
        RouteReverse { route: route }
    }

    pub fn submit<T1: RouteLike>(&mut self, route: T1) {
        let (a, b) = (route.head(), route.tail());
        self.delta += self.context.instance.distance(0, a as i16)
            + self.context.instance.distance(b as i16, 0);
        self.actions.extend([(0, a), (b, 0)]);
    }
}

#[test]
fn test() {
    let mut llist = LinkedList::<usize>::default();
    llist.insert(10, 0, 0);
    llist.insert(11, 0, 0);
    let ctx = RouteContext::new(llist, Instance::new(10, 10, vec![10], vec![vec![10]]));

    fn func<T: Clone>(op: &mut RouteEditContext<'_, T>, rs: [Route<'_, T>; 2]) {
        for e1 in rs[0].iter_edges() {
            for e2 in rs[1].iter_edges() {
                let (r11, r12) = op.split_at(rs[0].clone(), e1);
                let (r21, r22) = op.split_at(rs[0].clone(), e2);
                let j1 = op.join(r11, r22);
                let j2 = op.join(r21, r12);
                op.submit(j1);
                op.submit(j2);
            }
        }
    }

    ctx.probe(func);
}
