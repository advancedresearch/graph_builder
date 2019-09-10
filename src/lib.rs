//! # Graph-Builder
//! An algorithm for generating graphs with post-filtering and edge composition.
//!
//! This algorithm is used by automated theorem provers on typical group/category problems.
//! For example, to study algebra or path semantics.
//!
//! The advantage of this algorithm is that it exploits symmetry of Category Theory.
//! For every morphism `A -> B` and `B -> C`, there exists a morphism `A -> C`.
//! This means that for a large number of objects, one only needs to keep neighbour morphisms.
//!
//! Constructing a graph using small operations makes it possible to
//! minimize the work required to get from one node to another.
//!
//! For information of how use this library, see the documentation on the various functions.

#![deny(missing_docs)]

use std::hash::Hash;
use std::error::Error;

/// A graph is a tuple of nodes and edges between nodes.
pub type Graph<T, U> = (Vec<T>, Vec<([usize; 2], U)>);

/// Stores settings for generating graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenerateSettings {
    /// The maximum number of nodes before terminating.
    pub max_nodes: usize,
    /// The maximum number of edges before terminating.
    pub max_edges: usize,
}

/// Stores a graph generating error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GenerateError {
    /// Hit limit maximum number of nodes.
    MaxNodes,
    /// Hit limit maximum number of edges.
    MaxEdges,
}

impl std::fmt::Display for GenerateError {
    fn fmt(&self, w: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            GenerateError::MaxNodes => write!(w, "Reached limit maximum number of nodes"),
            GenerateError::MaxEdges => write!(w, "Reached limit maximum number of edges"),
        }
    }
}

impl Error for GenerateError {}

impl From<GenerateError> for () {
    fn from(_: GenerateError) -> () {()}
}

/// Generates a graph from:
///
/// - an initial seed state `a`
/// - maximum `n` edges
/// - a function `f` to generate a new node with edge
/// - a filter `g` to post-process nodes
/// - a composer `h` to use for transforming edges when nodes are filtered
/// - settings to control usage of memory
///
/// Returns a list of nodes and a list of edges.
///
/// - `Ok` if generation was successful without hitting memory limits
/// - `Err` if generation hit memory limits
///
/// Waiting with filtering until post-processing makes it possible
/// to create an edge between nodes that require multiple steps.
///
/// The maximum number of edges is usually determined from the length of a list of valid operations.
///
/// ### Error handling
///
/// The algorithm continues to post-processing when hitting memory limits.
/// It is assumed that one wants to make use of the data generated,
/// where memory limits are met often due to combinatorial explosions.
///
/// The algorithm assumes that there are other limits besides those in `GenerateSettings`.
/// To handle errors, one adds a type that implements `From<GenerateError>`.
/// The unit type `()` can be used when error handling is not needed.
///
/// Tip: Since this function requires 6 generic type arguments,
/// it is easier to set the error type on the output type and let Rust infer the rest.
///
/// For example:
///
/// ```ignore
/// let (nodes, edges) = match gen(start, n, f, g, h, &settings) {
///     Ok(x) => x,
///     Err((x, ())) => x,
/// };
/// ```
///
/// When an error happens during composing edges, one can choose whether to
/// report the error with `Err(Some(err))`, or ignore it with `Err(None)`.
/// This is useful because sometimes you want to filter edges without reporting errors.
/// The algorithm assumes that one wishes to continue generating the graph
/// when encountering an error. Only the first error will be reported.
pub fn gen<T, U, F, G, H, E>(
    (mut nodes, mut edges): Graph<T, U>,
    n: usize,
    f: F,
    g: G,
    h: H,
    settings: &GenerateSettings,
) -> Result<Graph<T, U>, (Graph<T, U>, E)>
    where T: Eq + Hash + Clone,
          F: Fn(&T, usize) -> Result<(T, U), E>,
          G: Fn(&T) -> bool,
          H: Fn(&U, &U) -> Result<U, Option<E>>,
          E: From<GenerateError>
{
    use std::collections::{HashMap, HashSet};

    let mut error: Option<E> = None;
    let mut has: HashMap<T, usize> = HashMap::new();
    let mut has_edge: HashSet<[usize; 2]> = HashSet::new();
    for n in &nodes {
        has.insert(n.clone(), 0);
    }
    for edge in &edges {
        has_edge.insert(edge.0);
    }
    let mut i = 0;
    'outer: while i < nodes.len() {
        for j in 0..n {
            match f(&nodes[i], j) {
                Ok((new_node, new_edge)) => {
                    let id = if let Some(&id) = has.get(&new_node) {id}
                    else {
                        let id = nodes.len();
                        has.insert(new_node.clone(), id);
                        nodes.push(new_node);
                        id
                    };
                    has_edge.insert([i, id]);
                    edges.push(([i, id], new_edge));

                    if nodes.len() >= settings.max_nodes {
                        if error.is_none() {
                            error = Some(GenerateError::MaxNodes.into());
                        }
                        break 'outer;
                    } else if edges.len() >= settings.max_edges {
                        if error.is_none() {
                            error = Some(GenerateError::MaxEdges.into());
                        }
                        break 'outer;
                    }
                }
                Err(err) => {
                    error = Some(err);
                }
            }
        }
        i += 1;
    }
    let mut removed: HashSet<usize> = HashSet::new();
    // Hash nodes that do not passes filter.
    for i in 0..nodes.len() {if !g(&nodes[i]) {removed.insert(i);}}
    let edges_count = edges.len();
    let mut removed_edges: Vec<usize> = vec![];
    let mut j = 0;
    // Generate new edges by composing them if they got removed.
    while j < edges.len() {
        let [a, b] = edges[j].0;
        if removed.contains(&b) {
            removed_edges.push(j);
            // Look for all edges that starts with removed node.
            for k in 0..edges_count {
                let [c, d] = edges[k].0;
                if c == b && !has_edge.contains(&[a, d]) {
                    // Compose the two edges into a new one that
                    // no longer refers to the removed node.
                    match h(&edges[j].1, &edges[k].1) {
                        Ok(new_edge) => {
                            edges.push(([a, d], new_edge));
                            has_edge.insert([a, d]);
                        }
                        Err(None) => {}
                        Err(Some(err)) => {
                            if error.is_none() {
                                error = Some(err);
                            }
                        }
                    }
                }
            }
        }
        j += 1;
    }

    let mut new_nodes = vec![];
    let mut map_nodes: Vec<Option<usize>> = vec![];
    for (i, node) in nodes.into_iter().enumerate() {
        if removed.contains(&i) {
            map_nodes.push(None);
        } else {
            let id = new_nodes.len();
            map_nodes.push(Some(id));
            new_nodes.push(node);
        }
    }
    for j in (0..edges.len()).rev() {
        let [a, b] = edges[j].0;
        if let (Some(a), Some(b)) = (map_nodes[a], map_nodes[b]) {
            edges[j].0 = [a, b];
        } else {
            edges.swap_remove(j);
        }
    }

    if let Some(err) = error {
        Err(((new_nodes, edges), err))
    } else {
        Ok((new_nodes, edges))
    }
}

/// Filters edges such that only those who are equal in both directions remains.
///
/// Removes redundant edges and edges which only exist in one direction.
///
/// Does not preserve the order of edges.
/// The order of the edges is unsorted afterwards.
///
/// Assumes that there are maximum two edges between nodes.
pub fn bidir<T: PartialEq + std::fmt::Debug>(edges: &mut Vec<([usize; 2], T)>) {
    if edges.len() == 0 {return};

    // Fix indices such that they pair up.
    for j in 0..edges.len() {
        let [a, b] = edges[j].0;
        edges[j].0 = [a.min(b), a.max(b)];
    }
    edges.sort_by_key(|s| s.0);
    let mut pair = false;
    for j in (0..edges.len()).rev() {
        let k = j + 1;
        if pair {
            if k >= edges.len() {
                edges.swap_remove(j);
            } else {
                if edges[j] == edges[k] {
                    edges.swap_remove(k);
                } else {
                    edges.swap_remove(j);
                }
                pair = false;
            }
        } else {
            pair = true;
        }
    }
}
