use comparison::is_same_object;
use log::*;
use object::Object;
use petgraph::dot::{Config, Dot};
use rayon::prelude::*;
use smt::*;
use std::sync::{Arc, Mutex};
use trace::*;
//use petgraph::graph::DiGraph;
//use fileio::write_dot;
use petgraph::graphmap::DiGraphMap;
use petgraph::Direction;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use std::iter::FromIterator;

pub fn is_deterministic(_tr: &mut SymbolicTrace, benchmark: &str) -> (bool, u64, u64, u64, Duration) {
    //check_ev(_tr);
    //panic!();
    let mut formula = Formula::new(benchmark);
    warn!("Filtering local events...");
    let (g_evs, g_objs) = add_global_events(_tr);
    warn!("Building CFG...");
    let graph = build_cfg(_tr);
    //write_dot(&graph, "cfg");
    let constraint = formula.get_cfg_constraint(graph);
    formula.assert_formula(constraint);

    let locks = build_lock_pairs(_tr);
    warn!("lock pairs built");
    let constraint = formula.get_lock_constraints(locks);

    formula.assert_formula(constraint);

    let (rw, num_deps) = get_all_writes(_tr);
    //warn!("Dependencies {}", num_deps);
    //warn!("All writes build");
    //let constraint = formula.get_read_constraints(&rw);
    //formula.assert_formula(constraint);

    warn!("Solving");
    let start = Instant::now();
    let result = !formula.check_nondet(&rw, true);
    let end = start.elapsed();

    (result, num_deps, g_evs  as u64, g_objs as u64, end)
}



fn get_all_writes(trace: &SymbolicTrace) -> (HashMap<&Event, HashSet<&Event>>, u64) {
    let mut res = HashMap::new();
    let reads = trace
        .global_events
        .par_iter()
        .filter_map(|x| {
            if let EventType::Read { .. } = x.data {
                return Some(x);
            } else {
                return None;
            }
        })
        .collect::<Vec<_>>();

    let graph: DiGraphMap<&Event, &str> = build_dependencies(&trace);
    //write_dot(&graph, "dependencies");
    // In parallel!!
    let mut num_dep = 0;
    for e1 in reads {
        res.insert(e1, HashSet::new());
        //warn!("EDGES: {}", graph.neighbors_directed(e1, Direction::Incoming).collect::<Vec<_>>().len());
        for e2 in graph.neighbors_directed(e1, Direction::Incoming) {
            num_dep += 1;
            //warn!("{:#?} depends on {:#?}", e1, e2);
            res.get_mut(e1).unwrap().insert(e2);
        }
    }
    (res, num_dep)
}

fn build_dependencies(trace: &SymbolicTrace) -> DiGraphMap<&Event, &str> {
    let mut dep_graph: DiGraphMap<&Event, &str> = DiGraphMap::new();
    for (pos1, _ev1) in trace.global_events.iter().enumerate() {
        let mut nodep = true;
        for (pos2, _ev2) in trace.global_events.iter().enumerate() {
            if pos1 < pos2 {
                nodep = false;
                break;
            }
            match is_dependent(_ev1, _ev2, false) {
                1 => {
                    //warn!("{:#?}", _ev1);
                    //warn!("{:#?}", _ev2);
                    //warn!("##################################");
                    dep_graph.add_edge(_ev2, _ev1, "dep");
                    nodep = false;
                }
                2 => {
                    dep_graph.add_edge(_ev1, _ev2, "dep");
                    nodep = false;
                }
                3 => {
                    dep_graph.add_edge(_ev1, _ev2, "dep");
                    nodep = false;
                }
                _ => {}
            }
        }
        if nodep {
            if let EventType::Read { .. } = _ev1.data {
                warn!("No DEP: {:#?}", _ev1);
            }
        }
    }
    dep_graph
    //write_dot(&dep_graph, &*trace.id);
}

fn build_lock_pairs(trace: &SymbolicTrace) -> Vec<(&Event, &Event, &Event, &Event)> {
    use EventType::*;
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

fn get_lock_pairs(trace: &SymbolicTrace) -> Vec<(&Event, &Event)> {
    use EventType::*;
    let mut pairs = Vec::new();
    for _th in trace.thread_naming.values() {
        let mut _proj = trace
            .global_events
            .par_iter()
            .filter_map(|x| match x.data {
                Lock { .. } | Unlock { .. } => {
                    if _th == &x.thread {
                        return Some(x);
                    }
                    return None;
                }
                _ => {
                    return None;
                }
            })
            .collect::<Vec<_>>();
        let mut mtx_stacks: HashMap<Arc<Object>, Vec<&Event>> = HashMap::new();
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

fn build_cfg(trace: &SymbolicTrace) -> DiGraphMap<&Event, &str> {
    let mut last_ev: HashMap<String, &Event> = HashMap::new();
    let mut forks: HashMap<String, Vec<&Event>> = HashMap::new();
    let mut cfg: DiGraphMap<&Event, &str> = DiGraphMap::new();
    use EventType::*;
    for _ev in trace.global_events.iter() {
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
    //write_dot(&cfg, &*trace.id);
    return cfg;
}

fn get_number_of_objects(_tr: &SymbolicTrace) -> u64 {
    use EventType::*;
    let mut res = _tr.global_events.par_iter().filter_map(|ref x| {
        match x.data {
            Read { object: ref obj, .. } => {return Some(Arc::clone(obj));},
            Write { object: ref obj, .. } => {return Some(Arc::clone(obj));},
            _ => {return None;}
        }
    }).collect::<Vec<_>>();
     return vec_to_set(res).len() as u64;
}

fn get_events_breakdown(_tr: &SymbolicTrace) -> (u64, u64, u64, u64) {
    use EventType::*;
    let mut tmp = _tr.global_events.par_iter().filter_map(|ref _ev| {
        match _ev.data {
            Read { .. } => {return Some(1 as u64);},
            _ => {return None;}
        }
    }).collect::<Vec<_>>();
    let reads: u64 = tmp.par_iter().sum();
    
    tmp = _tr.global_events.par_iter().filter_map(|ref _ev| {
        match _ev.data {
            Write { .. } => {return Some(1 as u64);},
            _ => {return None;}
        }
    }).collect::<Vec<_>>();
    let writes: u64 = tmp.par_iter().sum();

    tmp = _tr.global_events.par_iter().filter_map(|ref _ev| {
        match _ev.data {
            Fork { .. } | Join { .. } => {return Some(1 as u64);},
            _ => {return None;}
        }
    }).collect::<Vec<_>>();
    let spawn: u64 = tmp.par_iter().sum();

    tmp = _tr.global_events.par_iter().filter_map(|ref _ev| {
        match _ev.data {
            Lock { .. } | Unlock { .. } => {return Some(1 as u64);},
            _ => {return None;}
        }
    }).collect::<Vec<_>>();
    let locks: u64 = tmp.par_iter().sum();

    (reads, writes, spawn, locks)
}

fn vec_to_set(vec: Vec<Arc<Object>>) -> HashSet<Arc<Object>> {
    return HashSet::from_iter(vec);
}
fn add_global_events(_tr: &mut SymbolicTrace) -> (u64, u64){
    use EventType::*;
    let res = _tr.events
        .par_iter()
        .filter_map(|_ev| {
            match &_ev.data {
                &Read { .. } | &Write { .. } => {
                    if let Some(_) = _tr.events.par_iter().find_any(|ref x| {
                        let res = is_dependent(_ev, &x, false);
                        return res != 0;
                    }) {
                        return Some(_ev.clone());
                    }
                    return None;
                }
                &Lock { .. } | &Unlock { .. } => {
                    return Some(_ev.clone());
                }
                &Fork { .. } | &Join { .. } => {
                    return Some(_ev.clone());
                }
                &Branch { .. } => {
                    return None;
                }
                _ => {
                    return None;
                }
            };
        })
        .collect::<Vec<_>>();
    _tr.global_events = res;
    /*
    let objs = get_number_of_objects(_tr);
    let (reads, writes, spawn, locks) = get_events_breakdown(_tr);
    warn!("Events: {}", _tr.events.len());
    warn!("Global Events: {}", _tr.global_events.len());
    warn!("Read Events: {}", reads);
    warn!("Write Events: {}", writes);
    warn!("Spawn Events: {}", spawn);
    warn!("Lock Events: {}", locks);
    */
    return (_tr.global_events.len() as u64, 0);
}

fn is_dependent_concrete(_ev1: &Event, _ev2: &Event, is_write: bool) -> u8 {
    use EventType::*;
    if _ev1.thread == _ev2.thread || _ev1 == _ev2 {
        return 0;
    }
    match (&_ev1.data, &_ev2.data) {
        (Read { concrete: con1, .. }, Write { concrete: con2, .. }) => {
            if con1 == con2 {
                return 1;
            }
        }
        (Write { concrete: con1, .. }, Read { concrete: con2, .. }) => {
            if con1 == con2 {
                return 2;
            }
        }
        (Write { concrete: con1, .. }, Write { concrete: con2, .. }) => {
            if con1 == con2 && is_write {
                return 3;
            }
        }
        _ => {}
    }
    return 0;
}

fn is_dependent(_ev1: &Event, _ev2: &Event, is_write: bool) -> u8 {
    use EventType::*;
    if _ev1.thread == _ev2.thread || _ev1 == _ev2 {
        return 0;
    }

    match (&_ev1.data, &_ev2.data) {
        (
            Read {
                offset: off1,
                object: ref obj1,
                concrete: con1,
                ..
            },
            Write {
                offset: off2,
                object: ref obj2,
                concrete: con2,
                ..
            },
        ) => {
            if off1 == off2 && is_conflict(obj1, obj2) && con1 == con2 {
                return 1;
            }
        }
        (
            Write {
                offset: off1,
                object: ref obj1,
                concrete: con1,
                ..
            },
            Read {
                offset: off2,
                object: ref obj2,
                concrete: con2,
                ..
            },
        ) => {
            if off1 == off2 && is_conflict(obj1, obj2) && con1 == con2 {
                return 2;
            }
        }
        (
            Write {
                offset: off1,
                object: ref obj1,
                concrete: con1,
                ..
            },
            Write {
                offset: off2,
                object: ref obj2,
                concrete: con2,
                ..
            },
        ) => {
            if off1 == off2 && is_conflict(obj1, obj2) && con1 == con2 && is_write {
                return 3;
            }
        }
        _ => {}
    }
    return 0;
}

fn is_conflict(obj1: &Object, obj2: &Object) -> bool {
    return is_same_object(obj1, obj2)
        && obj1.construction == obj2.construction
        && obj1.owner == obj2.owner;
}
fn is_dependent_symbolic(_ev1: &Event, _ev2: &Event, is_write: bool) -> u8 {
    if _ev1.thread == _ev2.thread || _ev1 == _ev2 {
        return 0;
    }
    let w_set1 = get_write_set(_ev1);
    let r_set1 = get_read_set(_ev1);
    let w_set2 = get_write_set(_ev2);
    let r_set2 = get_read_set(_ev2);

    for (obj1, off1) in w_set1.iter() {
        for (obj2, off2) in r_set2.iter() {
            //if is_same_object(obj1, obj2) {
            if obj1.hex_addr == obj2.hex_addr {
                if let (Some(_), Some(_)) = (off1, off2) {
                    if off1 == off2 {
                        return 1;
                    }
                } else {
                    return 1;
                }
            }
        }
    }

    for (obj1, off1) in w_set2.iter() {
        for (obj2, off2) in r_set1.iter() {
            //if is_same_object(obj1, obj2) {
            if obj1.hex_addr == obj2.hex_addr {
                if let (Some(_), Some(_)) = (off1, off2) {
                    if off1 == off2 {
                        return 2;
                    }
                } else {
                    return 2;
                }
            }
        }
    }

    if is_write {
        for (obj1, off1) in w_set1.iter() {
            for (obj2, off2) in w_set2.iter() {
                //if is_same_object(obj1, obj2) {
                if obj1.hex_addr == obj2.hex_addr {
                    if let (Some(_), Some(_)) = (off1, off2) {
                        if off1 == off2 {
                            return 3;
                        }
                    } else {
                        return 3;
                    }
                }
            }
        }
    }
    return 0;
}
pub fn get_read_set(_ev: &Event) -> Vec<(Arc<Object>, Option<u64>)> {
    use EventType::*;
    let mut res = Vec::new();

    match &_ev.data {
        Read {
            object: ref obj,
            offset: off,
            ..
        } => {
            res.push((Arc::clone(obj), Some(off.clone())));
        }
        /*
        Call { args: ref args1, ..} => {
            for arg in args1.iter() {
                if let EventData::Pointer {target: tar, ..} = arg {
                    res.push((Arc::clone(tar), None)); 
                }
            }
        }
        */
        _ => {}
    }
    return res;
}

fn get_write_set(_ev: &Event) -> Vec<(Arc<Object>, Option<u64>)> {
    use EventType::*;
    let mut res = Vec::new();

    match &_ev.data {
        Write {
            object: ref obj,
            offset: off,
            ..
        } => {
            res.push((Arc::clone(obj), Some(off.clone())));
        }
        /*
        Call { value: ref val, args: ref args1, ..} => {
            for arg in args1.iter() {
                if let EventData::Pointer {target: tar, ..} = arg {
                    res.push((Arc::clone(tar), None)); 
                }
            }
            if let EventData::Pointer {target: tar, ..} = val {
                res.push((Arc::clone(tar), None));
            }
        },
        */
        _ => {}
    }

    return res;
}
