use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    sync::Mutex,
    time::Instant,
};

use rustc_hash::FxHashSet;

use rayon::prelude::*;

type Solution = (String, String, String, String, String);

fn main() {
    let start = Instant::now();
    println!("importing words");
    let mut words = Graph::import("words_alpha.txt");
    println!("Importing words took {:.2?}", start.elapsed());

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
    neighbors: FxHashSet<u16>,
}

impl Node {
    fn new(s: String) -> Self {
        Self {
            word: s,
            neighbors: FxHashSet::default(),
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
                    self.0[i].neighbors.insert(j as u16);
                }
            }
        }
    }

    fn search_graph(&mut self) -> Vec<Solution> {
        let solutions: Mutex<Vec<Solution>> = Mutex::default();

        let nodes = &self.0;

        nodes.par_iter().for_each(|i| {
            for &j in &i.neighbors {
                let j = &nodes[j as usize];

                let thirds = i
                    .neighbors
                    .intersection(&j.neighbors)
                    .copied()
                    .collect::<FxHashSet<_>>();

                if thirds.len() < 3 {
                    continue;
                }

                for &k in &thirds {
                    let k = &nodes[k as usize];
                    let fourths = thirds
                        .intersection(&k.neighbors)
                        .copied()
                        .collect::<FxHashSet<_>>();

                    if fourths.len() < 2 {
                        continue;
                    }

                    for &l in &fourths {
                        let l = &nodes[l as usize];
                        let fifths = fourths
                            .intersection(&l.neighbors)
                            .copied()
                            .collect::<Vec<_>>();

                        for m in fifths {
                            let m = &nodes[m as usize];
                            solutions.lock().unwrap().push((
                                i.word.clone(),
                                j.word.clone(),
                                k.word.clone(),
                                l.word.clone(),
                                m.word.clone(),
                            ));
                        }
                    }
                }
            }
        });

        solutions.into_inner().unwrap()
    }
}
