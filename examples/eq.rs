/*

This example shows to generate all solutions of an equation of the form:

    x0 + x1 + x2 + ... + xn-2 = xn-1

For example:

    a + b = c
    c - a = b
    c - b = a

Each solution is a node in a generated graph.
An edge tells how to swap sides and sign to get from one node to another.

To get from one solution to another, one only needs to move maximum two terms.

If the right side is negative, automatic inversion is used.
This improves the performance of the graph generation.

The number of nodes from `n` terms and `m` right-side terms is:

    bin(n, m)

The number of edges is the number of pairs between nodes:

    pairs(bin(n, m))

    pairs(n) = n * (n-1) / 2

*/

extern crate graph_builder;

use std::str::FromStr;

use graph_builder::*;

fn main() {
    // Change this to control the number of terms in the equation.
    let n: usize = if let Some(x) = std::env::args_os().skip(1).take(1).next() {
        usize::from_str(&x.into_string().expect("number")).unwrap()
    } else {
        3
    };
    // Change this to control how many
    let solution_terms: usize = if let Some(x) = std::env::args_os().skip(2).take(1).next() {
        usize::from_str(&x.into_string().expect("number")).unwrap()
    } else {
        1
    };

    // Putting all terms except the last one
    let start = Eq {
        side: {
            if solution_terms == 1 && n > 0 {
                let mut res = vec![true; n-1];
                res.push(false);
                res
            } else {
                vec![true; n]
            }
        },
        positive: vec![true; n],
    };

    // Swap side and sign on the chosen term.
    let f = |eq: &Eq, ind: usize| {
        let mut eq = eq.clone();
        eq.side[ind] = !eq.side[ind];
        eq.positive[ind] = !eq.positive[ind];
        Ok((eq, Swap(vec![ind])))
    };
    // Filter nodes to those with the specified number of solutions.
    let g = |eq: &Eq| eq.len_right() == solution_terms;
    // Join operations.
    // Since these swap operations are commutative, require order.
    let h = |a: &Swap, b: &Swap| if a.0 >= b.0 {Err(None)} else {Ok(Swap({
        let mut a = a.0.clone();
        a.extend_from_slice(&b.0);
        a.sort();
        a
    }))};

    let settings = GenerateSettings {
        max_nodes: 1000,
        max_edges: 1000,
    };

    let seed = (vec![start], vec![]);
    // Generate graph.
    let (eqs, mut edges) = match gen(seed, n, f, g, h, &settings) {
        Ok(x) => x,
        Err((x, ())) => x,
    };

    // Remove all edges that are not bidirectional.
    bidir(&mut edges);
    edges.sort();
    for i in 0..eqs.len() {
        println!("{}: {}", i, eqs[i]);
    }
    for i in 0..edges.len() {
        println!("{:?}", edges[i]);
    }

    println!("(nodes, edges): ({}, {})", eqs.len(), edges.len());
}


/// Stores an equation.
#[derive(PartialEq, Eq, Clone, Hash)]
pub struct Eq {
    /// The sides of the terms.
    pub side: Vec<bool>,
    /// The signs of the terms.
    pub positive: Vec<bool>,
}

impl Eq {
    /// Returns the number of terms on the right.
    pub fn len_right(&self) -> usize {
        self.side.iter().filter(|&&n| n).count()
    }

    /// Returns an index if the equation has a unique right side.
    pub fn unique_right(&self) -> Option<usize> {
        let mut found = None;
        for i in 0..self.side.len() {
            if self.side[i] {
                if found.is_some() {return None};
                found = Some(i);
            }
        }
        found
    }

    /// Returns a tuple of signs, one when positive and one when negative.
    pub fn signs(&self) -> (&'static str, &'static str) {
        if let Some(ind) = self.unique_right() {
            if self.positive[ind] {("+", "-")}
            else {("-", "+")}
        } else {("+", "-")}
    }
}

/// Stores swap operations.
#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct Swap(pub Vec<usize>);

impl std::fmt::Display for Eq {
    fn fmt(&self, w: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let mut left: usize = 0;
        let (plus, minus) = self.signs();
        for i in 0..self.side.len() {
            if !self.side[i] {
                if self.positive[i] {write!(w, "{}", plus)?}
                else {write!(w, "{}", minus)?}
                write!(w, "x{} ", i)?;
                left += 1;
            }
        }
        if left == 0 {write!(w, "0 ")?}
        write!(w, "= ")?;
        if self.side.len() - left == 0 {
            write!(w, "0")?;
        } else {
            for i in 0..self.side.len() {
                if self.side[i] {
                    if self.positive[i] {write!(w, "{}", plus)?}
                    else {write!(w, "{}", minus)?}
                    write!(w, "x{} ", i)?
                }
            }
        }
        Ok(())
    }
}
