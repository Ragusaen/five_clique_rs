use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    sync:: mpsc::channel,
    time::Instant, cmp::min, collections::{HashSet, HashMap},
};


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
            let mut q = HashSet::with_capacity(5);

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
        // Sort words by frequency of character usage. The idea here is that if a word uses many frequently used characters, it is a worse contester, since this leaves fewer of them for the rest of the words
        // We want to consider the worst words first, because these have fewer neighbours and therefore we need to consider less combinations over all. I.e. if badword was last, then it would be checked in all other words as well, when it could be excluded early
        let char_frequency: HashMap<char, i32> = HashMap::from([
            ('a', 82), ('b', 15), ('c', 28), ('d', 43), ('e', 130), ('f', 22), ('g', 20), ('h', 61), ('i', 70), ('j', 2), ('k', 8), ('l', 40), ('m', 24), ('n', 67), ('o', 75), ('p', 19), ('q', 1), ('r', 60), ('s', 63), ('t', 91), ('u', 28), ('v', 10), ('w', 24), ('x', 2), ('y', 20), ('z', 1)
        ]);

        self.0.sort_by_cached_key(|n| -n.word.chars().map(|c| char_frequency[&c]).sum::<i32>());

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

        nodes.par_iter().for_each_with(
            // preinitialize vectors to avoid extra allocations
            (send,  Vec::<u16>::with_capacity(4096),  Vec::<u16>::with_capacity(4096),  Vec::<u16>::with_capacity(4096)) ,
        |(s,  thirds, fourths, fifths),i| {


            for &j in &i.neighbors {
                let j = &nodes[j as usize];

                intersection_sorted_inplace(&i.neighbors, &j.neighbors, thirds);

                if thirds.len() < 3 {
                    continue;
                }

                for k in thirds.iter() {
                    let k = &nodes[*k as usize];
                    intersection_sorted_inplace(thirds, &k.neighbors, fourths);

                    if fourths.len() < 2 {
                        continue;
                    }

                    for l in fourths.iter() {
                        let l = &nodes[*l as usize];
                        intersection_sorted_inplace(fourths, &l.neighbors,  fifths);

                        for m in fifths.iter() {
                            let m = &nodes[*m as usize];
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

fn intersection_sorted_inplace<T: PartialOrd + Clone>(a: &Vec<T>, b: &Vec<T>, output: &mut Vec<T>) {
    output.clear();

    let mut b_iter = b.iter();

    if let Some(mut current_b) = b_iter.next() {
        for current_a in a {
            while *current_b < *current_a {
                current_b = match b_iter.next() {
                    Some(current_b) => current_b,
                    None => return,
                }
            }

            if *current_a == *current_b {
                output.push(current_a.clone())
            }
        }
    }
}