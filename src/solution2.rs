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

    pub fn iter_range(&self, head: usize, tail: usize) -> ForwardIter<'_, T> {
        ForwardIter {
            list: self,
            current: head,
            end: tail,
            exhausted: false,
        }
    }

    pub fn iter_range_rev(&self, tail: usize, head: usize) -> BackwardIter<'_, T> {
        BackwardIter {
            list: self,
            current: tail,
            end: head,
            exhausted: false,
        }
    }
}

pub struct ForwardIter<'a, T> {
    list: &'a LinkedList<T>,
    current: usize,
    end: usize,
    exhausted: bool,
}

impl<'a, T> Iterator for ForwardIter<'a, T> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }
        let ret = self.current;
        if self.current == self.end {
            self.exhausted = true;
        } else {
            self.current = self.list.successor(self.current);
        }
        Some(ret)
    }
}

pub struct BackwardIter<'a, T> {
    list: &'a LinkedList<T>,
    current: usize,
    end: usize,
    exhausted: bool,
}

impl<'a, T> Iterator for BackwardIter<'a, T> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }
        let ret = self.current;
        if self.current == self.end {
            self.exhausted = true;
        } else {
            self.current = self.list.predecessor(self.current);
        }
        Some(ret)
    }
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

    for edge1 in std::iter::once((0, routes[0].0)).chain(
        list.iter_range(routes[0].0, routes[0].1)
            .map(|node| (node, list.successor(node))),
    ) {
        let r11 = (routes[0].0, edge1.0);
        let r12 = (edge1.1, routes[0].1);

        for edge2 in std::iter::once((0, routes[1].0)).chain(
            list.iter_range(routes[1].0, routes[1].1)
                .map(|node| (node, list.successor(node))),
        ) {
            let r21 = (routes[1].0, edge2.0);
            let r22 = (edge2.1, routes[1].1);
            {
                println!("edge1 = {:?}; edge2 = {:?}", edge1, edge2);
                // println!("{:?}", (0, list.data(r11.0)));
                // println!("{:?}", (list.data(r11.1), list.data(r22.0)));
                // println!("{:?}", (list.data(r22.0), 0));
                // println!("{:?}", (0, list.data(r21.0)));
                // println!("{:?}", (list.data(r21.1), list.data(r12.0)));
                // println!("{:?}", (list.data(r12.0), 0));
                // println!();
            }
        }
    }
}
