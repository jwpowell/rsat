#![allow(unused)]

use rsat::dimacs::*;
use rsat::solver::Solver;

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

    let mut solver = Solver::new(dimacs);

    while !solver.finished() {
        solver.step();
    }

    println!("{:?}", solver.status());

    Ok(())
}
