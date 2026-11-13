//! Port of Effect-TS/tsgo `internal/graph/graph.go`: a generic directed
//! graph data structure for modeling typed relationships between nodes.
//!
//! PORT: Go keeps nodes and edges in maps and sorts the keys before every
//! ordered walk. The port keeps them in `BTreeMap`s, whose iteration order
//! is the sorted key order. Go walks of the unsorted maps (`Reverse`,
//! `FilterNodes`, `FilterEdges`, `MapNodes`, `MapEdges`) have no defined
//! order; the port walks them in index order, which is one of Go's orders.
//! Go `iter.Seq2` walks become iterators of `(index, &data)`. The traversals
//! (`DFS`, `DFSPostOrder`, `BFS`, `Topo`) compute the whole order first; they
//! have no side effects, so a caller that stops early sees the same items.

use crate::prelude::*;
use std::collections::BTreeMap;

/// Go `NodeIndex`: identifies a node in the graph. Indices are
/// auto-incremented and never reused.
pub type NodeIndex = i32;

/// Go `EdgeIndex`: identifies an edge in the graph. Indices are
/// auto-incremented and never reused.
pub type EdgeIndex = i32;

/// Go `Direction`: the direction of traversal or query.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    /// Follows edges from source to target.
    Outgoing,
    /// Follows edges from target to source.
    Incoming,
}

impl Direction {
    /// The Go string value.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Direction::Outgoing => "Outgoing",
            Direction::Incoming => "Incoming",
        }
    }
}

/// Go `Edge`: a directed edge between two nodes with associated data.
#[derive(Clone, Debug)]
pub struct Edge<E> {
    pub source: NodeIndex,
    pub target: NodeIndex,
    pub data: E,
}

/// Go `TraversalConfig`: configures traversal algorithms (DFS, BFS, etc.).
/// When `start` is empty, defaults to all source nodes (nodes with no
/// incoming edges). When `direction` is empty (Go `""`, here `None`),
/// defaults to `Outgoing`.
#[derive(Clone, Debug, Default)]
pub struct TraversalConfig {
    pub start: Vec<NodeIndex>,
    pub direction: Option<Direction>,
}

/// Go `MermaidOptions`: configures Mermaid flowchart rendering.
/// A `None` callback is a Go nil func.
pub struct MermaidOptions<'a, N, E> {
    pub node_label: Option<Box<dyn Fn(&N) -> String + 'a>>,
    pub node_shape: Option<Box<dyn Fn(&N) -> (String, String) + 'a>>,
    pub edge_label: Option<Box<dyn Fn(&E) -> String + 'a>>,
    pub edge_shape: Option<Box<dyn Fn(&E) -> (String, String) + 'a>>,
    pub direction: String,
}

impl<N, E> Default for MermaidOptions<'_, N, E> {
    fn default() -> Self {
        MermaidOptions {
            node_label: None,
            node_shape: None,
            edge_label: None,
            edge_shape: None,
            direction: String::new(),
        }
    }
}

/// Go `isAcyclicStackEntry`: used by `IsAcyclic` for iterative DFS cycle
/// detection.
#[derive(Clone, Copy, Debug)]
pub struct IsAcyclicStackEntry {
    pub node: NodeIndex,
    pub neighbor_idx: i32,
}

/// Go `Graph`: a generic directed graph with typed node and edge data.
/// It uses adjacency lists backed by maps for O(1) lookups.
pub struct Graph<N, E> {
    pub nodes: BTreeMap<NodeIndex, N>,
    pub edges: BTreeMap<EdgeIndex, Edge<E>>,
    pub adjacency: FxHashMap<NodeIndex, Vec<EdgeIndex>>,
    pub reverse_adjacency: FxHashMap<NodeIndex, Vec<EdgeIndex>>,
    pub next_node_index: NodeIndex,
    pub next_edge_index: EdgeIndex,
}

impl<N, E> Default for Graph<N, E> {
    fn default() -> Self {
        Self::new()
    }
}

/// Go `removeFromSlice`: removes the first occurrence of `val` from `slice`.
#[must_use]
pub fn remove_from_slice(mut slice: Vec<EdgeIndex>, val: EdgeIndex) -> Vec<EdgeIndex> {
    for (i, v) in slice.iter().enumerate() {
        if *v == val {
            slice.remove(i);
            return slice;
        }
    }
    slice
}

impl<N, E> Graph<N, E> {
    /// Go `New`: creates an empty directed graph.
    #[must_use]
    pub fn new() -> Self {
        Graph {
            nodes: BTreeMap::new(),
            edges: BTreeMap::new(),
            adjacency: FxHashMap::default(),
            reverse_adjacency: FxHashMap::default(),
            next_node_index: 0,
            next_edge_index: 0,
        }
    }

    /// Go `AddNode`: adds a node with the given data and returns its index.
    pub fn add_node(&mut self, data: N) -> NodeIndex {
        let idx = self.next_node_index;
        self.nodes.insert(idx, data);
        self.adjacency.insert(idx, Vec::new());
        self.reverse_adjacency.insert(idx, Vec::new());
        self.next_node_index += 1;
        idx
    }

    /// Go `GetNode`: the data for the node at the given index. `None` is the
    /// Go `false` result: the node does not exist.
    #[must_use]
    pub fn get_node(&self, index: NodeIndex) -> Option<&N> {
        self.nodes.get(&index)
    }

    /// Go `HasNode`: true if a node with the given index exists.
    #[must_use]
    pub fn has_node(&self, index: NodeIndex) -> bool {
        self.nodes.contains_key(&index)
    }

    /// Go `UpdateNode`: applies `f` to the data of the node at the given
    /// index. If the node does not exist, this is a no-op.
    pub fn update_node(&mut self, index: NodeIndex, f: impl FnOnce(N) -> N)
    where
        N: Clone,
    {
        if let Some(data) = self.nodes.get(&index) {
            let data = f(data.clone());
            self.nodes.insert(index, data);
        }
    }

    /// Go `RemoveNode`: removes the node at the given index and all its
    /// incident edges.
    pub fn remove_node(&mut self, index: NodeIndex) {
        if !self.has_node(index) {
            return;
        }

        // Collect all edge indices to remove (deduplicate since self-loops appear in both lists)
        let mut edge_set: FxHashSet<EdgeIndex> = FxHashSet::default();
        for ei in self.adjacency.get(&index).into_iter().flatten() {
            edge_set.insert(*ei);
        }
        for ei in self.reverse_adjacency.get(&index).into_iter().flatten() {
            edge_set.insert(*ei);
        }

        // Remove each edge from the opposite node's adjacency lists and from edges map
        for ei in edge_set {
            // PORT: Go reads a missing edge as the zero `Edge` (source and
            // target 0).
            let (source, target) = match self.edges.get(&ei) {
                Some(edge) => (edge.source, edge.target),
                None => (0, 0),
            };
            if source != index {
                let list = self.adjacency.remove(&source).unwrap_or_default();
                self.adjacency.insert(source, remove_from_slice(list, ei));
            }
            if target != index {
                let list = self.reverse_adjacency.remove(&target).unwrap_or_default();
                self.reverse_adjacency
                    .insert(target, remove_from_slice(list, ei));
            }
            self.edges.remove(&ei);
        }

        self.nodes.remove(&index);
        self.adjacency.remove(&index);
        self.reverse_adjacency.remove(&index);
    }

    /// Go `NodeCount`: the number of nodes in the graph.
    #[must_use]
    pub fn node_count(&self) -> i32 {
        self.nodes.len() as i32
    }

    /// Go `AddEdge`: adds a directed edge from source to target with the
    /// given data. It panics if either source or target does not exist.
    pub fn add_edge(&mut self, source: NodeIndex, target: NodeIndex, data: E) -> EdgeIndex {
        if !self.has_node(source) {
            panic!("graph: source node does not exist");
        }
        if !self.has_node(target) {
            panic!("graph: target node does not exist");
        }
        let idx = self.next_edge_index;
        self.edges.insert(
            idx,
            Edge {
                source,
                target,
                data,
            },
        );
        self.adjacency.entry(source).or_default().push(idx);
        self.reverse_adjacency.entry(target).or_default().push(idx);
        self.next_edge_index += 1;
        idx
    }

    /// Go `GetEdge`: the edge at the given index. `None` is the Go `false`
    /// result: the edge does not exist.
    #[must_use]
    pub fn get_edge(&self, index: EdgeIndex) -> Option<&Edge<E>> {
        self.edges.get(&index)
    }

    /// Go `HasEdge`: true if there is an edge from source to target.
    #[must_use]
    pub fn has_edge(&self, source: NodeIndex, target: NodeIndex) -> bool {
        for ei in self.adjacency.get(&source).into_iter().flatten() {
            // PORT: Go reads a missing edge as the zero `Edge` (target 0).
            let edge_target = self.edges.get(ei).map_or(0, |e| e.target);
            if edge_target == target {
                return true;
            }
        }
        false
    }

    /// Go `OutgoingEdges`: the edge indices for all outgoing edges from the
    /// given node. The returned indices preserve insertion order.
    #[must_use]
    pub fn outgoing_edges(&self, node_index: NodeIndex) -> Vec<EdgeIndex> {
        self.adjacency.get(&node_index).cloned().unwrap_or_default()
    }

    /// Go `IncomingEdges`: the edge indices for all incoming edges to the
    /// given node. The returned indices preserve insertion order.
    #[must_use]
    pub fn incoming_edges(&self, node_index: NodeIndex) -> Vec<EdgeIndex> {
        self.reverse_adjacency
            .get(&node_index)
            .cloned()
            .unwrap_or_default()
    }

    /// Go `UpdateEdge`: applies `f` to the data of the edge at the given
    /// index. If the edge does not exist, this is a no-op.
    pub fn update_edge(&mut self, index: EdgeIndex, f: impl FnOnce(E) -> E)
    where
        E: Clone,
    {
        if let Some(edge) = self.edges.get_mut(&index) {
            edge.data = f(edge.data.clone());
        }
    }

    /// Go `RemoveEdge`: removes the edge at the given index.
    pub fn remove_edge(&mut self, index: EdgeIndex) {
        let Some(edge) = self.edges.get(&index) else {
            return;
        };
        let (source, target) = (edge.source, edge.target);
        let list = self.adjacency.remove(&source).unwrap_or_default();
        self.adjacency
            .insert(source, remove_from_slice(list, index));
        let list = self.reverse_adjacency.remove(&target).unwrap_or_default();
        self.reverse_adjacency
            .insert(target, remove_from_slice(list, index));
        self.edges.remove(&index);
    }

    /// Go `EdgeCount`: the number of edges in the graph.
    #[must_use]
    pub fn edge_count(&self) -> i32 {
        self.edges.len() as i32
    }

    /// Go `Nodes`: iterates all nodes as (index, data) pairs, sorted by index.
    pub fn nodes(&self) -> impl Iterator<Item = (NodeIndex, &N)> + '_ {
        self.nodes.iter().map(|(k, v)| (*k, v))
    }

    /// Go `Edges`: iterates all edges as (index, edge) pairs, sorted by index.
    pub fn edges(&self) -> impl Iterator<Item = (EdgeIndex, &Edge<E>)> + '_ {
        self.edges.iter().map(|(k, v)| (*k, v))
    }

    /// Go `FindNode`: the index of the first node (in index order) matching
    /// the predicate. Returns (0, false) if no node matches.
    pub fn find_node(&self, mut predicate: impl FnMut(&N) -> bool) -> (NodeIndex, bool) {
        for (k, data) in &self.nodes {
            if predicate(data) {
                return (*k, true);
            }
        }
        (0, false)
    }

    /// Go `FindNodes`: the indices of all nodes matching the predicate, in
    /// index order.
    pub fn find_nodes(&self, mut predicate: impl FnMut(&N) -> bool) -> Vec<NodeIndex> {
        let mut result = Vec::new();
        for (k, data) in &self.nodes {
            if predicate(data) {
                result.push(*k);
            }
        }
        result
    }

    /// Go `FindEdge`: the index of the first edge (in index order) matching
    /// the predicate. The predicate receives (edgeData, source, target).
    /// Returns (0, false) if no edge matches.
    pub fn find_edge(
        &self,
        mut predicate: impl FnMut(&E, NodeIndex, NodeIndex) -> bool,
    ) -> (EdgeIndex, bool) {
        for (k, edge) in &self.edges {
            if predicate(&edge.data, edge.source, edge.target) {
                return (*k, true);
            }
        }
        (0, false)
    }

    /// Go `FindEdges`: the indices of all edges matching the predicate, in
    /// index order. The predicate receives (edgeData, source, target).
    pub fn find_edges(
        &self,
        mut predicate: impl FnMut(&E, NodeIndex, NodeIndex) -> bool,
    ) -> Vec<EdgeIndex> {
        let mut result = Vec::new();
        for (k, edge) in &self.edges {
            if predicate(&edge.data, edge.source, edge.target) {
                result.push(*k);
            }
        }
        result
    }



    /// Go `Neighbors`: all neighbor node indices (union of outgoing targets
    /// and incoming sources), deduplicated and sorted by index for
    /// determinism.
    #[must_use]
    pub fn neighbors(&self, node_index: NodeIndex) -> Vec<NodeIndex> {
        let mut seen: FxHashSet<NodeIndex> = FxHashSet::default();
        for ei in self.adjacency.get(&node_index).into_iter().flatten() {
            if let Some(edge) = self.edges.get(ei) {
                seen.insert(edge.target);
            }
        }
        for ei in self
            .reverse_adjacency
            .get(&node_index)
            .into_iter()
            .flatten()
        {
            if let Some(edge) = self.edges.get(ei) {
                seen.insert(edge.source);
            }
        }
        let mut result: Vec<NodeIndex> = seen.into_iter().collect();
        result.sort_unstable();
        result
    }

    /// Go `Reverse`: reverses the direction of all edges in the graph
    /// in-place.
    pub fn reverse(&mut self) {
        for edge in self.edges.values_mut() {
            std::mem::swap(&mut edge.source, &mut edge.target);
        }
        std::mem::swap(&mut self.adjacency, &mut self.reverse_adjacency);
    }

    /// Go `FilterNodes`: removes all nodes that do not satisfy the
    /// predicate. Incident edges of removed nodes are also removed.
    pub fn filter_nodes(&mut self, mut predicate: impl FnMut(&N) -> bool) {
        let mut to_remove: Vec<NodeIndex> = Vec::new();
        for (idx, data) in &self.nodes {
            if !predicate(data) {
                to_remove.push(*idx);
            }
        }
        for idx in to_remove {
            self.remove_node(idx);
        }
    }

    /// Go `FilterEdges`: removes all edges whose data does not satisfy the
    /// predicate. All nodes are preserved.
    pub fn filter_edges(&mut self, mut predicate: impl FnMut(&E) -> bool) {
        let mut to_remove: Vec<EdgeIndex> = Vec::new();
        for (idx, edge) in &self.edges {
            if !predicate(&edge.data) {
                to_remove.push(*idx);
            }
        }
        for idx in to_remove {
            self.remove_edge(idx);
        }
    }

    /// Go `MapNodes`: applies `f` to every node's data in-place.
    pub fn map_nodes(&mut self, mut f: impl FnMut(N) -> N)
    where
        N: Clone,
    {
        for data in self.nodes.values_mut() {
            *data = f(data.clone());
        }
    }

    /// Go `MapEdges`: applies `f` to every edge's data in-place.
    pub fn map_edges(&mut self, mut f: impl FnMut(E) -> E)
    where
        E: Clone,
    {
        for edge in self.edges.values_mut() {
            edge.data = f(edge.data.clone());
        }
    }

    /// Go `sortedNeighborsDirected`: neighbor node indices reachable via
    /// edges in the given direction, sorted by edge index for deterministic
    /// traversal order.
    fn sorted_neighbors_directed(
        &self,
        node_index: NodeIndex,
        direction: Direction,
    ) -> Vec<NodeIndex> {
        let adj_list = if direction == Direction::Incoming {
            self.reverse_adjacency.get(&node_index)
        } else {
            self.adjacency.get(&node_index)
        };
        let mut sorted: Vec<EdgeIndex> = adj_list.cloned().unwrap_or_default();
        sorted.sort_unstable();
        let mut result = Vec::with_capacity(sorted.len());
        for ei in &sorted {
            if let Some(edge) = self.edges.get(ei) {
                if direction == Direction::Incoming {
                    result.push(edge.source);
                } else {
                    result.push(edge.target);
                }
            }
        }
        result
    }


    /// The data of a visited node.
    /// PORT: Go `g.nodes[node]` gives the zero `N` for a start index that is
    /// not in the graph; the port panics there.
    fn node_data(&self, node: NodeIndex) -> &N {
        &self.nodes[&node]
    }





    /// Go `IsAcyclic`: true if the graph contains no cycles. Uses iterative
    /// DFS with recursion stack tracking to detect back edges.
    #[must_use]
    pub fn is_acyclic(&self) -> bool {
        let mut visited: FxHashSet<NodeIndex> = FxHashSet::default();
        let mut recursion_stack: FxHashSet<NodeIndex> = FxHashSet::default();

        for start_node in self.nodes.keys().copied() {
            if visited.contains(&start_node) {
                continue;
            }

            let mut stack = vec![IsAcyclicStackEntry {
                node: start_node,
                neighbor_idx: 0,
            }];
            visited.insert(start_node);
            recursion_stack.insert(start_node);

            while let Some(top) = stack.last_mut() {
                let neighbors = self.sorted_neighbors_directed(top.node, Direction::Outgoing);

                if (top.neighbor_idx as usize) < neighbors.len() {
                    let neighbor = neighbors[top.neighbor_idx as usize];
                    top.neighbor_idx += 1;

                    if recursion_stack.contains(&neighbor) {
                        return false;
                    }
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        recursion_stack.insert(neighbor);
                        stack.push(IsAcyclicStackEntry {
                            node: neighbor,
                            neighbor_idx: 0,
                        });
                    }
                } else {
                    let node = top.node;
                    recursion_stack.remove(&node);
                    stack.pop();
                }
            }
        }

        true
    }

}

/// Go `Clone`: a deep copy of the graph. Node and edge data values are
/// shallow-copied, but all internal maps and slices are independently
/// allocated.
impl<N: Clone, E: Clone> Clone for Graph<N, E> {
    fn clone(&self) -> Self {
        let mut c = Graph {
            nodes: BTreeMap::new(),
            edges: BTreeMap::new(),
            adjacency: FxHashMap::with_capacity_and_hasher(
                self.adjacency.len(),
                Default::default(),
            ),
            reverse_adjacency: FxHashMap::with_capacity_and_hasher(
                self.reverse_adjacency.len(),
                Default::default(),
            ),
            next_node_index: self.next_node_index,
            next_edge_index: self.next_edge_index,
        };
        c.nodes
            .extend(self.nodes.iter().map(|(k, v)| (*k, v.clone())));
        c.edges
            .extend(self.edges.iter().map(|(k, v)| (*k, v.clone())));
        for (k, v) in &self.adjacency {
            c.adjacency.insert(*k, v.clone());
        }
        for (k, v) in &self.reverse_adjacency {
            c.reverse_adjacency.insert(*k, v.clone());
        }
        c
    }
}

/// Go `dfsPostOrderEntry`: used by `DFSPostOrder` for the two-phase stack
/// approach.
#[derive(Clone, Copy, Debug)]
pub struct DfsPostOrderEntry {
    pub node: NodeIndex,
    pub expanded: bool,
}

