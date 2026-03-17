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

    pub fn apply_actions(&mut self, actions: impl IntoIterator<Item = Action>) {
        for action in actions {
            match action {
                Action::ReverseSegment { left, right } => {
                    let (old_pred, old_succ) = (self.predecessor(left), self.successor(right));
                    self.reversed_link(left, right, old_pred, old_succ);
                }
                Action::Link { pred, succ } => {
                    self.link(pred, succ);
                }
            }
        }
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

#[must_use]
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

#[must_use]
#[derive(Debug, Clone)]
pub struct RouteSplitLeft<T> {
    route: T,
    tail: usize,
    other_head: usize,
}

#[must_use]
#[derive(Debug, Clone)]
pub struct RouteSplitRight<T> {
    route: T,
    head: usize,
}

#[must_use]
#[derive(Debug, Clone)]
pub struct RouteJoin<T1, T2> {
    left: T1,
    right: T2,
}

#[must_use]
#[derive(Debug, Clone)]
pub struct RouteReverse<T> {
    route: T,
}

pub trait RouteLike: Sized + Clone {
    fn head(&self) -> usize;
    fn tail(&self) -> usize;

    fn execute<T: HasPosition>(&self, op: &mut RouteEditContext<'_, T>);
    fn split_at(self, edge: (usize, usize)) -> (RouteSplitLeft<Self>, RouteSplitRight<Self>) {
        (
            RouteSplitLeft {
                route: self.clone(),
                tail: edge.0,
                other_head: edge.1,
            },
            RouteSplitRight {
                route: self,
                head: edge.1,
            },
        )
    }

    fn join<T: RouteLike>(self, other: T) -> RouteJoin<Self, T> {
        RouteJoin {
            left: self,
            right: other,
        }
    }

    fn rev(self) -> RouteReverse<Self> {
        RouteReverse { route: self }
    }
}

impl<'a, T: Clone> RouteLike for Route<'a, T> {
    fn head(&self) -> usize {
        unsafe { self.context.routes.get_unchecked(self.index) }.0
    }

    fn tail(&self) -> usize {
        unsafe { self.context.routes.get_unchecked(self.index) }.1
    }

    fn execute<T1>(&self, _op: &mut RouteEditContext<'_, T1>) {}
}

impl<T: RouteLike> RouteLike for RouteSplitLeft<T> {
    fn head(&self) -> usize {
        if self.tail == 0 {
            0
        } else {
            self.route.head()
        }
    }

    fn tail(&self) -> usize {
        self.tail
    }

    fn execute<T1: HasPosition>(&self, op: &mut RouteEditContext<'_, T1>) {
        self.route.execute(op);
        if self.tail != 0 && self.other_head != 0 {
            let [a, b] = [self.tail, self.other_head].map(|x| op.context.data.data(x));
            op.delta -= op.context.instance.distance(a.position(), b.position());
        }
    }
}

impl<T: RouteLike> RouteLike for RouteSplitRight<T> {
    fn head(&self) -> usize {
        self.head
    }

    fn tail(&self) -> usize {
        if self.head == 0 {
            0
        } else {
            self.route.tail()
        }
    }

    fn execute<T1>(&self, _op: &mut RouteEditContext<'_, T1>) {}
}

impl<T1: RouteLike, T2: RouteLike> RouteLike for RouteJoin<T1, T2> {
    fn head(&self) -> usize {
        let h = self.left.head();
        if h == 0 {
            self.right.head()
        } else {
            h
        }
    }

    fn tail(&self) -> usize {
        let t = self.right.tail();
        if t == 0 {
            self.left.tail()
        } else {
            t
        }
    }

    fn execute<T: HasPosition>(&self, op: &mut RouteEditContext<'_, T>) {
        self.left.execute(op);
        self.right.execute(op);

        let (a, b) = (self.left.tail(), self.right.head());
        if a != 0 && b != 0 {
            op.actions.push(Action::link(a, b));
            let [apos, bpos] = [a, b].map(|x| op.context.data.data(x));
            op.delta += op
                .context
                .instance
                .distance(apos.position(), bpos.position());
        }
    }
}

impl<T: RouteLike> RouteLike for RouteReverse<T> {
    fn head(&self) -> usize {
        self.route.tail()
    }

    fn tail(&self) -> usize {
        self.route.head()
    }

    fn execute<T1: HasPosition>(&self, op: &mut RouteEditContext<'_, T1>) {
        self.route.execute(op);
        op.actions.push(Action::reverse_segment(
            self.route.head(),
            self.route.tail(),
        ));
    }
}

pub trait HasPosition {
    fn position(&self) -> i16;
}

#[derive(Debug, Clone)]
pub struct RouteContext<T> {
    pub data: LinkedList<T>,
    pub instance: Instance,
    routes: Vec<(usize, usize)>,
}

impl<T: Clone + HasPosition> RouteContext<T> {
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

    pub fn update(&mut self, best_move: Move) {
        let Move {
            delta: _,
            actions,
            used_routes,
            new_routes,
        } = best_move;
        self.data.apply_actions(actions);

        for i in 0..new_routes.len().max(used_routes.len()) {
            match (new_routes.get(i), used_routes.get(i)) {
                (Some(&new_route), Some(&j)) => unsafe {
                    *self.routes.get_unchecked_mut(j) = new_route
                },
                (None, Some(&j)) => {
                    // TODO: update cache since route (self.routes.len() - 1) is re-indexed to j.
                    self.routes.swap_remove(j);
                }
                (Some(&new_route), None) => self.routes.push(new_route),
                (None, None) => unreachable!(),
            }
        }
    }

    pub fn probe<const N: usize>(&self, func: ProbeFn<T, N>) -> Option<Move> {
        let mut best_delta = 0;
        let (mut actions, mut best_actions) = (Vec::new(), Vec::new());
        let (mut used_routes, mut new_routes, mut best_new_routes) =
            (Vec::new(), Vec::new(), Vec::new());
        let route_delta_init = |r: &Route<T>| {
            let [depot, head, tail] = [0, r.head(), r.tail()].map(|x| self.data.data(x).position());
            -self.instance.distance(depot, head) - self.instance.distance(tail, depot)
        };
        for routes in self.iter().array_combinations() {
            actions.clear();
            let mut ctx = RouteViewContext {
                context: self,
                actions: &mut actions,
                best_actions: &mut best_actions,
                delta_init: routes.iter().map(route_delta_init).sum(),
                best_delta: &mut best_delta,
                routes: routes.clone(),
                used_routes: &mut used_routes,
                new_routes: &mut new_routes,
                best_new_routes: &mut best_new_routes,
            };
            func(&mut ctx, routes.each_ref());
        }
        (best_delta < 0).then(|| Move {
            delta: best_delta,
            actions: best_actions,
            used_routes,
            new_routes: best_new_routes,
        })
    }
}

#[derive(Debug)]
pub struct Move {
    pub delta: i32,
    pub actions: Vec<Action>,
    pub used_routes: Vec<usize>,
    pub new_routes: Vec<(usize, usize)>,
}

pub type ProbeFn<T, const N: usize> = fn(&mut RouteViewContext<'_, T, N>, [&Route<'_, T>; N]);

#[derive(Debug)]
pub struct RouteViewContext<'a, T, const N: usize> {
    context: &'a RouteContext<T>,
    actions: &'a mut Vec<Action>,
    best_actions: &'a mut Vec<Action>,
    delta_init: i32,
    best_delta: &'a mut i32,
    routes: [Route<'a, T>; N],
    used_routes: &'a mut Vec<usize>,
    new_routes: &'a mut Vec<(usize, usize)>,
    best_new_routes: &'a mut Vec<(usize, usize)>,
}

impl<'a, T: Clone, const N: usize> RouteViewContext<'a, T, N> {
    pub fn simulate<F>(&mut self, func: F) -> bool
    where
        F: Fn(&mut RouteEditContext<'_, T>, [Route<'_, T>; N]),
    {
        self.actions.clear();
        self.new_routes.clear();
        let mut ctx = RouteEditContext {
            context: self.context,
            actions: self.actions,
            new_routes: self.new_routes,
            delta: self.delta_init,
        };
        func(&mut ctx, self.routes.clone());
        if ctx.delta < *self.best_delta {
            *self.best_delta = ctx.delta;
            *self.used_routes = self.routes.iter().map(|x| x.index).collect();
            swap(self.best_new_routes, self.new_routes);
            swap(self.best_actions, self.actions);
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub enum Action {
    Link { pred: usize, succ: usize },
    ReverseSegment { left: usize, right: usize },
}

impl Action {
    pub fn link(pred: usize, succ: usize) -> Self {
        Self::Link { pred, succ }
    }

    pub fn reverse_segment(left: usize, right: usize) -> Self {
        Self::ReverseSegment { left, right }
    }
}

#[derive(Debug)]
pub struct RouteEditContext<'a, T> {
    context: &'a RouteContext<T>,
    actions: &'a mut Vec<Action>,
    new_routes: &'a mut Vec<(usize, usize)>,
    delta: i32,
}

impl<'a, T: HasPosition> RouteEditContext<'a, T> {
    pub fn submit<T1: RouteLike>(&mut self, route: T1) {
        route.execute(self);
        let (a, b) = (route.head(), route.tail());
        let [depot, apos, bpos] = [0, a, b].map(|x| self.context.data.data(x).position());
        self.delta += self.context.instance.distance(depot, apos)
            + self.context.instance.distance(bpos, depot);
        if a != 0 && b != 0 {
            self.actions
                .extend([Action::link(0, a), Action::link(b, 0)]);
            self.new_routes.push((a, b));
        }
    }
}

#[test]
fn test() {
    fn coords_to_distance_matrix(coords: Vec<(i32, i32)>) -> Vec<Vec<i32>> {
        let n = coords.len();
        let mut matrix = vec![vec![0; n]; n];
        for i in 0..n {
            for j in 0..n {
                let dx = coords[i].0 - coords[j].0;
                let dy = coords[i].1 - coords[j].1;
                matrix[i][j] = ((dx * dx + dy * dy) as f64).sqrt() as i32;
            }
        }
        matrix
    }

    #[derive(Debug, Default, Clone)]
    struct NodeData {
        position: i16,
    }

    impl HasPosition for NodeData {
        fn position(&self) -> i16 {
            self.position
        }
    }

    let instance = Instance::new(
        8,
        10,
        vec![1; 8],
        coords_to_distance_matrix(vec![
            (1, 2),
            (2, 1),
            (-1, 2),
            (-2, 1),
            (-2, -1),
            (-1, -2),
            (1, -2),
            (2, -1),
        ]),
    );

    let mut llist = LinkedList::<NodeData>::default();
    for route in [vec![1, 3, 5, 7], vec![2, 4, 6]] {
        let mut depot = 0;
        for pos in route {
            let node = llist.insert(NodeData { position: pos }, depot, 0);
            depot = node;
        }
    }
    let mut ctx = RouteContext::new(llist, instance);
    println!(
        "{:?}",
        ctx.iter()
            .map(|r| r
                .iter_nodes()
                .map(|i| ctx.data.data(i).position())
                .collect::<Vec<_>>())
            .collect::<Vec<_>>()
    );
    let swap: ProbeFn<_, 2> = |ctx, [route1, route2]| {
        for e1 in route1.iter_edges() {
            for e2 in route2.iter_edges() {
                if ctx.simulate(|op, [r1, r2]| {
                    let (r11, r12) = r1.split_at(e1);
                    let (r21, r22) = r2.split_at(e2);
                    op.submit(r11.join(r22));
                    op.submit(r21.join(r12));
                }) {
                    return;
                };
            }
        }
    };
    while let Some(m) = ctx.probe(swap) {
        println!("Found move with delta {:?}", m);
        ctx.update(m);

        println!(
            "{:?}",
            ctx.iter()
                .map(|r| r
                    .iter_nodes()
                    .map(|i| ctx.data.data(i).position())
                    .collect::<Vec<_>>())
                .collect::<Vec<_>>()
        );
    }
}
