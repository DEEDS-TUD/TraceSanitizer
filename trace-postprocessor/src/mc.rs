use fileio::*;
use object::*;
use trace::*;

use comparison::*;
use log::*;
use petgraph::dot::{Config, Dot};
use petgraph::graph::DiGraph;
use petgraph::graphmap::DiGraphMap;
use petgraph::Direction;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use EventData::*;
use EventType::*;

fn is_dependent(_ev1: &Event, _ev2: &Event) -> u8 {
    if _ev1.thread == _ev2.thread {
        return 0;
    }
    match (&_ev1.data, &_ev2.data) {
        (
            Write {
                object: ref obj1,
                offset: ref off1,
                ..
            },
            Read {
                object: ref obj2,
                offset: ref off2,
                ..
            },
        ) => {
            if obj1 == obj2 && off1 == off2 {
                return 1;
            }
        }
        (
            Read {
                object: ref obj1,
                offset: ref off1,
                ..
            },
            Write {
                object: ref obj2,
                offset: ref off2,
                ..
            },
        ) => {
            if obj1 == obj2 && off1 == off2 {
                return 2;
            }
        }
        _ => {}
    }

    0
}

fn get_lock_pairs(trace: &SymbolicTrace) -> Vec<(Arc<Event>, Arc<Event>)> {
    let mut pairs = Vec::new();
    for _th in trace.thread_naming.values() {
        let mut _proj = trace
            .events
            .par_iter()
            .filter_map(|x| match x.data {
                Lock { .. } | Unlock { .. } => {
                    if _th == &x.thread {
                        return Some(Arc::clone(x));
                    }
                    return None;
                }
                _ => {
                    return None;
                }
            })
            .collect::<Vec<Arc<Event>>>();
        let mut mtx_stacks: HashMap<Arc<Object>, Vec<Arc<Event>>> = HashMap::new();
        for _ev in _proj {
            match &_ev.data {
                Lock { mutex: ref mtx } => {
                    if !mtx_stacks.contains_key(mtx) {
                        mtx_stacks.insert(Arc::clone(mtx), Vec::new());
                    }
                    mtx_stacks.get_mut(mtx).unwrap().push(_ev);
                }
                Unlock { mutex: ref mtx } => {
                    if !mtx_stacks.contains_key(mtx) {
                        mtx_stacks.insert(Arc::clone(mtx), Vec::new());
                    }
                    if let Some(lock) = mtx_stacks.get_mut(mtx).unwrap().pop() {
                        pairs.push((lock, _ev));
                    } else {
                        warn!("Found Unlocking event without matching Lock event");
                    }
                }
                _ => {}
            }
        }
    }
    return pairs;
}

pub fn build_lock_pairs(
    trace: &SymbolicTrace,
) -> Vec<(Arc<Event>, Arc<Event>, Arc<Event>, Arc<Event>)> {
    let mut res = Vec::new();
    let pairs = get_lock_pairs(&trace);
    for (pos1, (_ev11, _ev12)) in pairs.iter().enumerate() {
        for (pos2, (_ev21, _ev22)) in pairs.iter().enumerate() {
            if pos1 <= pos2 {
                break;
            }
            match (&_ev11.data, &_ev21.data) {
                (Lock { mutex: ref mtx1 }, Lock { mutex: ref mtx2 }) => {
                    if mtx1 == mtx2 {
                        res.push((*_ev11, *_ev12, *_ev21, *_ev22));
                    }
                }
                _ => {}
            }
        }
    }

    res
}
pub fn build_locks_dependencies<'a>(
    trace: &'a SymbolicTrace,
    graph: &mut DiGraphMap<&'a Event, &str>,
) {
    let pairs = get_lock_pairs(trace);

    for (pos1, (_ev11, _ev12)) in pairs.iter().enumerate() {
        for (pos2, (_ev21, _ev22)) in pairs.iter().enumerate() {
            if pos1 <= pos2 {
                break;
            }
            match (&_ev11.data, &_ev21.data) {
                (Lock { mutex: ref mtx1 }, Lock { mutex: ref mtx2 }) => {
                    if mtx1 == mtx2 {
                        graph.add_edge(_ev12, _ev21, "lock");
                        graph.add_edge(_ev22, _ev11, "lock");
                    }
                }
                _ => {}
            }
        }
    }
}
/*
pub fn build_dependencies<'a>(
    trace: &'a SymbolicTrace,
    dep_graph: &mut DiGraphMap<Arc<Event>, &str>,
) {
    for (pos1, _ev1) in trace.events.iter().enumerate() {
        for (pos2, _ev2) in trace.events.iter().enumerate() {
            if pos1 < pos2 {
                break;
            }
            match is_dependent(_ev1, _ev2) {
                1 => {
                    dep_graph.add_edge(_ev2, _ev1, "dep");
                }
                2 => {
                    dep_graph.add_edge(_ev1, _ev2, "dep");
                }
                _ => {}
            }
        }
    }
    //write_dot(&dep_graph, &*trace.id);
}
*/
pub fn build_cfg(trace: &SymbolicTrace) -> DiGraphMap<&Event, &str> {
    let mut last_ev: HashMap<String, &Event> = HashMap::new();
    let mut forks: HashMap<String, Vec<&Event>> = HashMap::new();
    let mut cfg: DiGraphMap<&Event, &str> = DiGraphMap::new();
    use EventType::*;
    for _ev in trace.events.iter() {
        let _th = &_ev.thread;
        /*
        match _ev.data {
            Fork { .. } => { forks.insert(_th.clone(), _ev);},
            _ => {}
        }
        */
        if let Fork { .. } = _ev.data {
            if let None = forks.get(_th) {
                forks.insert(_th.clone(), Vec::new());
            }
            forks.get_mut(_th).unwrap().push(_ev);
        }

        if let Some(last) = last_ev.get(_th) {
            cfg.add_edge(last, _ev, "po");
            if let Join { joiner: ref j, .. } = last.data {
                cfg.add_edge(last_ev.get(j).unwrap(), last, "join");
            }
        } else {
            for f in forks.values() {
                for e in f.iter() {
                    if let Fork {
                        createe: ref cr, ..
                    } = e.data
                    {
                        if cr == _th {
                            cfg.add_edge(e, _ev, "fork");
                        }
                    } else {
                        unreachable!();
                    }
                }
            }
        }
        last_ev.insert(_th.clone(), _ev);
    }
    for f in last_ev.values() {
        if let Join { joiner: ref j, .. } = f.data {
            cfg.add_edge(last_ev.get(j).unwrap(), f, "join");
        }
    }

    //debug!("{:#?}", get_lock_pairs(&trace));
    //build_dependencies(trace, &mut cfg);
    //build_locks_dependencies(trace, &mut cfg);
    write_dot(&cfg, &*trace.id);
    return cfg;
}

pub fn get_all_writes(trace: &SymbolicTrace) -> HashMap<&Arc<Event>, Vec<&Arc<Event>>> {
    let mut res = HashMap::new();
    let reads = trace
        .events
        .par_iter()
        .filter_map(|x| {
            if let Read { .. } = x.data {
                return Some(Arc::clone(x));
            } else {
                return None;
            }
        })
        .collect::<Vec<_>>();

    let mut graph: DiGraphMap<&Arc<Event>, &str> = DiGraphMap::new();
    //build_dependencies(&trace, &mut graph);
    write_dot(&graph, "sdfsdf");
    // In parallel!!
    for e1 in reads.iter() {
        res.insert(e1, Vec::new());
        for e2 in graph.neighbors_directed(e1, Direction::Outgoing) {
            res.get_mut(&e1).unwrap().push(e2);
        }
    }
    res
}
