use clap::Parser;
use gfa::parser::GFAParser;
use handlegraph::{
    handle::Handle,
    conversion::from_gfa,
    packedgraph::PackedGraph,
    pathhandlegraph::{GraphPathNames, IntoNodeOccurrences, GraphPaths, IntoPathIds, PathId}, handlegraph::{IntoNeighbors, IntoHandles},
};
use rayon::prelude::*;
use std::{path::PathBuf, collections::HashSet, hash::Hash, u128};
use std::io::{Error, ErrorKind};
use std::time::Instant;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    path: PathBuf,

    #[arg(short, long)]
    query: String,
}

fn remove_duplicates<T: Eq + Hash>(s: Vec<T>) -> Vec<T> {
    s.into_iter().collect::<HashSet<_>>().into_iter().collect::<Vec<_>>()
}

fn path_name(graph: &PackedGraph, path_id: PathId) -> Option<String> {
    graph.get_path_name(path_id).map(|n| std::str::from_utf8(&n.collect::<Vec<_>>()).unwrap().to_owned())
}

fn bench_paths_full(graph: &PackedGraph) -> Result<u128, Box<dyn std::error::Error>> {
    let now = Instant::now();
    let handle = Handle::new(51273, gfa::gfa::Orientation::Forward);
    let steps = graph.steps_on_handle(handle).ok_or(Error::new(ErrorKind::Other, "Handle should have steps"))?;
    let path_infos: Vec<_> = steps.par_bridge()
        .map(|step| (path_name(graph, step.0), graph.path_len(step.0)))
        .collect();
    let filtered_infos = remove_duplicates(path_infos);
    for info in filtered_infos {
        println!("{:?}", info);
    }
    Ok(now.elapsed().as_millis())
}

fn bench_steps_iolinks(graph: &PackedGraph) -> Result<u128, Box<dyn std::error::Error>> {
    let now = Instant::now();
    let steps = graph.path_ids().par_bridge().map(|id| {
        if graph.path_first_step(id).is_none() {
            return Vec::new().into_iter();
        }
        let mut cstep = graph.path_first_step(id).unwrap();
        let mut idx = 1;
        let handle = graph.path_handle_at_step(id, cstep).unwrap();
        let ilinks = graph.degree(handle, handlegraph::handle::Direction::Left);
        let olinks = graph.degree(handle, handlegraph::handle::Direction::Right);
        let cpath_name = path_name(graph, id);
        let mut v = vec![(cpath_name.clone(), idx, ilinks, olinks)];
        while graph.path_next_step(id, cstep).is_some() {
            cstep = graph.path_next_step(id, cstep).unwrap();
            idx = idx + 1;
            let handle = graph.path_handle_at_step(id, cstep).unwrap();
            let ilinks = graph.degree(handle, handlegraph::handle::Direction::Left);
            let olinks = graph.degree(handle, handlegraph::handle::Direction::Right);
            v.push((cpath_name.clone(), idx, ilinks, olinks));
        }
        v.into_iter()
    }).collect::<Vec<_>>();
    let steps = steps.into_iter().flatten();
    for step in steps {
        println!("{:?}", step);
    }
    Ok(now.elapsed().as_millis())
}

fn bench_path_lengths(graph: &PackedGraph) -> Result<u128, Box<dyn std::error::Error>> {
    let now = Instant::now();
    let paths = graph.path_ids().map(|id| (path_name(graph, id), graph.path_len(id))).collect::<Vec<_>>();
    for path in paths {
        println!("{:?}", path);
    }
    Ok(now.elapsed().as_millis())
}

fn bench_nodes_high_path_count(graph: &PackedGraph) -> Result<u128, Box<dyn std::error::Error>> {
    let now = Instant::now();
    let mut nodes = graph.handles().map(|handle| (handle.unpack_number(), graph.steps_on_handle(handle).map_or(0, |iter| iter.count()))).collect::<Vec<_>>();
    nodes.sort_by_key(|t| t.1);
    for node in nodes {
        println!("{:?}", node);
    }
    Ok(now.elapsed().as_millis())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let now = Instant::now();
    let cli = Cli::parse();
    let gfa_parser = GFAParser::new();
    let gfa = gfa_parser.parse_file(cli.path)?;
    let graph = from_gfa::<PackedGraph, ()>(&gfa);
    let parsing = now.elapsed().as_millis();
    match &cli.query[..] {
        "nodes_high_path_count" => {
            let nodes = bench_nodes_high_path_count(&graph)?;
            eprintln!("nodes_high_path_count: {},\t{}", nodes, nodes + parsing);
        },
        "path_lengths" => {
            let paths = bench_path_lengths(&graph)?;
            eprintln!("paths_lengths: {},\t{}", paths, paths + parsing);
        },
        "path_lengths_through_node" => {
            let paths_full = bench_paths_full(&graph)?;
            eprintln!("paths_full: {},\t{}", paths_full, paths_full + parsing);
        },
        "steps_ionodes" => {
            let steps_iolinks = bench_steps_iolinks(&graph)?;
            eprintln!("steps_iolinks: {},\t{}", steps_iolinks, steps_iolinks + parsing);
        },
        _ => {
            eprintln!("Please provide a valid query")
        }
    }

    Ok(())
}
