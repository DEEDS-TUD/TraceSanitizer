#![macro_use]
use csv::*;
use error::*;
use instruction::*;
use log::*;
use object::*;
use petgraph::dot::{Config, Dot};
use petgraph::graphmap::DiGraphMap;
use petgraph::visit::Bfs;
use petgraph::Direction;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::fs::{File, write, OpenOptions, read_to_string};
use std::hash::Hash;
use std::io::prelude::*;
use std::path::Path;
use std::process;
use std::result::Result;
use std::sync::{Arc, Mutex};
use trace::*;
use utils::*;

pub fn read_instructions(f_name: &str) -> Result<Vec<Instruction>, Box<Error>> {
    info!("Loading instructions...");
    let mut res = Vec::new();
    let file = File::open(&*f_name)?;
    let mut rdr = Reader::from_reader(file);
    for result in rdr.records() {
        let record = result?;
        if let Some(inst) = Instruction::from(&record) {
            res.push(inst);
        }
    }
    Ok(res)
}

pub fn read_output_hash(fname: &str) -> Result<String, Box<Error>> {
    info!("Loading output hash...");
    let mut out_hash = read_to_string(fname)?;

    return Ok(out_hash);
}
pub fn read_ret_code(f_name: &str) -> Result<String, Box<Error>> {
    info!("Loading return code...");
    let mut ret_code = String::new();
    let file = File::open(&*f_name)?;
    let mut rdr = Reader::from_reader(file);
    let mut flag = true;
    for result in rdr.records() {
        let record = result?;
        if let Some(c) = record.get(0) {
            flag = false;
            ret_code = String::from(c);
        }
    }
    if flag {
        return Err(Box::new(SanError::new("couldn't load return code...")));
    }
    Ok(ret_code)
}
pub fn read_injections(f_name: &str) -> Result<Vec<(u64, u32)>, Box<Error>> {
    info!("Loading injections...");
    let mut res = Vec::new();
    let file = File::open(&*f_name)?;
    let mut rdr = Reader::from_reader(file);
    for result in rdr.records() {
        let record = result?;
        if let Some(inj) = record.get(1) {
            if let Some(ts) = record.get(0) {
                res.push((get_long(ts), get_int(inj)));
                //                res.push(get_int(inj));
            }
        }
    }
    Ok(res)
}

pub fn write_check_results(fname: &str, res: Vec<String>) -> Result<(), Box<Error>>{
    //let file = Some(File::create(fname)?);
    let file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(fname)?;
    let mut wtr = Writer::from_writer(file);
    
    wtr.write_record(&["Result", "Global-events", "Global-objects", "Global-dependencies", "Solving-time", "Total-time"])?;
    wtr.write_record(&[&*res[0], &*res[1], &*res[2], &*res[3], &*res[4], &*res[5]])?;
    Ok(())
}
pub fn write_results(
    fname: &str,
    results: Vec<Vec<String>>,
    is_override: bool,
    is_append: bool,
) -> Result<(), Box<Error>> {
    let mut tmp_name = String::from(fname);
    let mut i = 0;
    while !is_override && Path::new(&*tmp_name).exists() {
        i += 1;
        tmp_name = format!("{}-{}", fname, i);
    }
    tmp_name += ".csv";
    warn!("Writing {} results items to {}", results.len(), tmp_name);

    let mut file = None;
    if !is_append {
        file = Some(File::create(tmp_name)?);
    } else {
        file = Some(OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(tmp_name)?);
    }
    let mut wtr = Writer::from_writer(file.unwrap());
    if !is_append {
        wtr.write_record(&[
            "Benchmark",
            "Fault-model",
            "#",
            "Deviation",
            "Output-Deviation",
            "Exit-code",
            "Fault-active",
            "Symbolification",
            "Comparison",
        ])?;
    }
    for rec in results.iter() {
        wtr.write_record(&[
            &*rec[0], &*rec[1], &*rec[2], &*rec[3], &*rec[4], &*rec[5], &*rec[6], &*rec[7], &*rec[8]
        ])?;
    }
    Ok(())
}

pub fn read_logical_mapping(f_name: &str) -> Result<HashMap<u64, String>, Box<Error>> {
    info!("loading logical naming...");
    let mut res = HashMap::new();
    let _file = File::open(f_name)?;

    let mut _rdr = Reader::from_reader(_file);
    for result in _rdr.records() {
        let _record = result?;
        let el0 = _record.get(0).ok_or("0")?;
        let el1 = _record.get(1).ok_or("1")?;

        res.insert(String::from(el0).parse()?, String::from(el1));
    }
    Ok(res)
}
pub fn write_to_file(content: &str, f_name: &str) -> Result<(), Box<Error>> {
    let path = Path::new(f_name);
    let mut file = File::create(&path)?;

    if let Err(_err) = file.write_all(content.as_bytes()) {
        warn!("Couldn't write to file: {}", _err);
    }
    Ok(())
}

pub fn write_dot(graph: &DiGraphMap<&Event, &str>, f_name: &str) -> Result<(), Box<Error>> {
    let content = Dot::with_config(graph, &[]).to_string();
    let mut n = String::from(f_name);
    n.push_str(".dot");
    write_to_file(&content[..], &*n)?;
    Ok(())
}

pub fn read_thread_graph(
    m_name: &str,
    root: u64,
) -> Result<(HashMap<String, Vec<String>>, HashMap<u64, String>), Box<Error>> {
    info!("Loading thread mapping...");
    let _file = File::open(m_name)?;
    let mut _rdr = Reader::from_reader(_file);
    let mut thread_graph: DiGraphMap<u64, u64> = DiGraphMap::new();
    let mut cnt = 0;
    for result in _rdr.records() {
        let _record = result?;

        let a = _record.get(1).ok_or("0")?.to_string().parse()?;
        let b = _record.get(2).ok_or("0")?.to_string().parse()?;
        //thread_graph.add_edge(a, b, _record.get(0).unwrap().to_string().parse().unwrap());
        thread_graph.add_edge(a, b, cnt);
        cnt += 1;
    }

    let mut idx = 0;
    let mut res_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut thread_map: HashMap<u64, String> = HashMap::new();
    let mut idx_map: HashMap<u64, u64> = HashMap::new();
    let mut bfs = Bfs::new(&thread_graph, root);
    while let Some(n) = bfs.next(&thread_graph) {
        if let Some(parent) = thread_graph
            .neighbors_directed(n, Direction::Incoming)
            .collect::<Vec<_>>()
            .get(0)
        {
            let mut idx = *idx_map.get_mut(parent).ok_or("0")?;
            let mut pre = String::from(&*thread_map.get(parent).ok_or("0")?.clone());
            pre.push_str("_");
            pre.push_str(&*long_to_string(idx));
            thread_map.insert(n, pre);
            idx = idx + 1;
            idx_map.insert(*parent, idx);
            idx_map.insert(n, 0);
        } else {
            thread_map.insert(n, String::from("T_0"));
            idx_map.insert(n, 0);
        }
    }

    /*
    // might need a map for idx to count separately for every parent node
    let mut tmp = String::from("T");
    tmp.push_str(&*idx.to_string());
    thread_map.insert(root, tmp);
    idx += 1;
    let mut bfs = Bfs::new(&thread_graph, root);
    while let Some(n) = bfs.next(&thread_graph) {
        println!("N: {:#?}", n);
        let mut neigh = thread_graph
            .neighbors_directed(n, Direction::Incoming)
            .collect::<Vec<_>>();
        neigh.sort_by_key(|k| thread_graph.edge_weight(n, *k));
        for gh in neigh.iter() {
            let mut pre = String::from("");
            pre.push_str(thread_map.get(&gh).unwrap());
            pre.push_str("_");
            pre.push_str(&*idx.to_string());
            thread_map.insert(n, pre);
            idx += 1;
        }
    }
*/
    let mut flag = Arc::new(Mutex::new(Vec::new()));
    for n in thread_graph.nodes() {
        let mut neigh = thread_graph
            .neighbors_directed(n, Direction::Outgoing)
            .collect::<Vec<_>>();
        neigh.sort_by_key(|k| thread_graph.edge_weight(n, *k));
        let lst = neigh
            .iter()
            .map(|x| {
                if !thread_map.contains_key(x) {
                    flag.lock().unwrap().push("0");
                    return String::new();
                }

                return thread_map.get(x).unwrap().to_owned();
            })
            .collect::<Vec<_>>();
        res_map.insert(
            thread_map
                .get(&n)
                .ok_or(SanError::new("Couldn't load mapping"))?
                .clone(),
            lst,
        );
    }
    if !flag.lock().unwrap().is_empty() {
        return Err(Box::new(SanError::new("Coudln't load thread mapping")));
    }
    //let content = Dot::with_config(&thread_graph, &[]).to_string();
    //let mut n = String::from("file");
    //n.push_str(split_at(m_name, 3));
    //n.push_str(".dot");
    //write_to_file(&content[..], &*n)?;
    return Ok((res_map, thread_map));
}

pub fn read_globals(
    g_name: &str,
    dest: usize,
    objects: &mut Vec<Arc<Object>>,
) -> Result<(), Box<Error>> {
    info!("Loading globals...");
    objects.push(Arc::new(Object::get_null()));
    let _file = File::open(g_name)?;
    let mut _rdr = Reader::from_reader(_file);
    for result in _rdr.records() {
        let _record = result?;
        let mut obj = Object::from_global(&_record);
        obj.update_validity(dest);
        objects.push(Arc::new(obj));
    }

    Ok(())
}
