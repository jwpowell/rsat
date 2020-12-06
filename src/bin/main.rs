#![allow(unused)]

use rsat::dimacs::*;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use lzma::LzmaReader;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("USAGE: {} DIMACS_FILE", args[0]);
        return Ok(());
    }

    let file = File::open(&args[1])?;
    let lzma = lzma::LzmaReader::new_decompressor(file).unwrap();
    let io = BufReader::new(lzma);
    let dimacs = Dimacs::new(io);

    let mut clauses: Vec<Vec<i32>> = dimacs.collect();

    for _ in 0..100 {
        let pivot = *clauses.first().unwrap().first().unwrap();
        let before = clauses.len();

        conflicts(&mut clauses, pivot);

        let after = clauses.len();

        let before = before as f32;
        let after = after as f32;

        println!(
            "Before/After: {} {} {:0.02}",
            before,
            after,
            100.0 * (after - before) / before
        );
    }

    Ok(())
}

fn conflicts(clauses: &mut Vec<Vec<i32>>, pivot: i32) {
    let mut conflicts = Vec::new();

    let mut pos: Vec<usize> = clauses
        .iter()
        .enumerate()
        .filter(|(_, clause)| clause.contains(&pivot))
        .map(|e| e.0)
        .collect();

    let mut neg: Vec<usize> = clauses
        .iter()
        .enumerate()
        .filter(|(_, clause)| clause.contains(&-pivot))
        .map(|e| e.0)
        .collect();

    let mut pos_count = 0;
    let mut neg_count = 0;

    for p in &pos {
        for n in &neg {
            if p == n {
                continue;
            }

            let mut conflict: Vec<i32> = clauses[*p]
                .iter()
                .filter(|l| **l != pivot)
                .chain(clauses[*n].iter().filter(|l| **l != -pivot))
                .copied()
                .collect();

            conflict.sort_unstable();
            conflict.dedup();

            conflicts.push(conflict);
        }
    }

    let mut start = 0;

    let mut indexes: Vec<usize> = pos.iter().chain(neg.iter()).copied().collect();

    indexes.sort_unstable();
    indexes.dedup();

    for index in indexes.iter().rev() {
        clauses.swap_remove(*index);
    }

    clauses.extend(conflicts);

    for clause in clauses.iter_mut() {
        clause.sort_unstable();
        clause.dedup();
    }

    clauses.sort_unstable();
    clauses.dedup();
}
