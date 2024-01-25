use clap::Parser;
use gfa::parser::GFAParser;
use handlegraph::{
    handle::Handle,
    conversion::from_gfa,
    packedgraph::PackedGraph,
    pathhandlegraph::{GraphPathNames, IntoNodeOccurrences, GraphPaths},
};
use rayon::prelude::*;
use std::path::PathBuf;
use std::io::{Error, ErrorKind};
use std::time::Instant;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    path: PathBuf,
}


fn bench_paths_full(graph: PackedGraph) -> Result<u128, Box<dyn std::error::Error>> {
    let now = Instant::now();
    let handle = Handle::new(51273, gfa::gfa::Orientation::Forward);
    let steps = graph.steps_on_handle(handle).ok_or(Error::new(ErrorKind::Other, "Handle should have steps"))?;
    let path_infos: Vec<_> = steps.par_bridge()
        .map(|step| (graph
                     .get_path_name(step.0)
                     .map(|n| std::str::from_utf8(&n.collect::<Vec<_>>()).unwrap().to_owned()), graph.path_len(step.0)))
        .collect();
    for info in path_infos {
        println!("{:?}", info);
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
    let paths_full = bench_paths_full(graph)?;

    eprintln!("paths_full: {},\t{}", paths_full, paths_full + parsing);
    Ok(())
}
