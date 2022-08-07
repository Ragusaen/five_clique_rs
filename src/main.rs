use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    sync:: mpsc::channel,
    time::Instant, cmp::min,
};

use rustc_hash::FxHashSet;

use rayon::prelude::*;

type Solution = (String, String, String, String, String);

fn main() {
    let start = Instant::now();
    println!("importing words");
    let mut words = Graph::import("words_alpha.txt");
    println!("Importing words took {:.2?}", start.elapsed());

    println!("{}", words.0.len());

    let construction_start = Instant::now();
    println!("constructing graph");
    words.construct_graph();
    println!(
        "Graph construction took {:.2?}",
        construction_start.elapsed()
    );

    let search_start = Instant::now();

    println!("searching graph");
    let mut solutions = words.search_graph();
    println!("Graph search took {:.2?}", search_start.elapsed());

    println!(
        "Found {} solutions in {:.2?}",
        solutions.len(),
        start.elapsed()
    );

    solutions.sort();

    let mut output = File::create("output.txt").unwrap();

    for solution in solutions {
        writeln!(
            output,
            "{}, {}, {}, {}, {}",
            solution.0, solution.1, solution.2, solution.3, solution.4
        )
        .unwrap();
    }
}

struct Node {
    word: String,
    neighbors: Vec<u16>,
}

impl Node {
    fn new(s: String) -> Self {
        Self {
            word: s,
            neighbors: Vec::with_capacity(4096),
        }
    }

    fn is_neighbors_with(&self, other: &Node) -> bool {
        self.word.chars().all(|c| !other.word.contains(c))
    }
}

struct Graph(Vec<Node>);

impl Graph {
    fn import(filename: &str) -> Self {
        fn no_duplicate_characters(s: &str) -> bool {
            let mut q = FxHashSet::default();
            q.reserve(5);

            for i in s.chars() {
                q.insert(i);
            }

            q.len() == s.len()
        }

        Graph(
            BufReader::new(File::open(filename).unwrap())
                .lines()
                .filter_map(|x| {
                    let x = x.unwrap();
                    let x = x.trim();
                    if x.len() == 5 && no_duplicate_characters(x) {
                        Some(Node::new(x.to_owned()))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
        )
    }

    fn construct_graph(&mut self) {
        for i in 0..self.0.len() {
            for j in i..self.0.len() {
                if self.0[i].is_neighbors_with(&self.0[j]) {
                    self.0[i].neighbors.push(j as u16)
                }
            }
        }
    }

    fn search_graph(&self) -> Vec<Solution> {
        let (send, recv) = channel();

        let nodes = &self.0;

        nodes.par_iter().for_each_with(send,|s,i| {
            for &j in &i.neighbors {
                let j = &nodes[j as usize];

                let thirds = intersection_sorted(&i.neighbors, &j.neighbors);

                if thirds.len() < 3 {
                    continue;
                }

                for &k in &thirds {
                    let k = &nodes[k as usize];
                    let fourths = intersection_sorted(&thirds, &k.neighbors);

                    if fourths.len() < 2 {
                        continue;
                    }

                    for &l in &fourths {
                        let l = &nodes[l as usize];
                        let fifths = intersection_sorted(&fourths, &l.neighbors);

                        for m in fifths {
                            let m = &nodes[m as usize];
                            s.send((
                                i.word.clone(),
                                j.word.clone(),
                                k.word.clone(),
                                l.word.clone(),
                                m.word.clone(),
                            )).unwrap();
                        }
                    }
                }
            }
        });

        recv.into_iter().collect()
    }
}

fn intersection_sorted<T: PartialOrd + Clone>(a: &Vec<T>, b: &Vec<T>) -> Vec<T> {
    let mut output = Vec::with_capacity(min(a.len(), b.len()));

    let mut b_iter = b.iter();

    if let Some(mut current_b) = b_iter.next() {
        for current_a in a {
            while *current_b < *current_a {
                current_b = match b_iter.next() {
                    Some(current_b) => current_b,
                    None => return output,
                }
            }

            if *current_a == *current_b {
                output.push(current_a.clone())
            }
        }
    }
    output
}