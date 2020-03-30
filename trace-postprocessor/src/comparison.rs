use log::*;
use object::*;
use smt::*;
use trace::*;
use utils::*;

use rayon::prelude::*;
use std::cmp::min;
use std::collections::HashMap;

fn is_same_event(evt1: &Event, evt2: &Event) -> String {
    use trace::EventType::*;
    match (&evt1.data, &evt2.data) {
        (
            &Read {
                value: ref v1,
                object: ref o1,
                offset: ref off1,
                ..
            },
            &Read {
                value: ref v2,
                object: ref o2,
                offset: ref off2,
                ..
            },
        ) => {
            let res = is_same_event_data(v1, v2);
            if res == "Data" {
                return String::from("data-dev");
            } else if res == "Addr" {
                return String::from("addr-dev");
            } else if !(is_same_object(o1, o2) && off1 == off2) {
                return String::from("addr-dev");
            }
        }
        (
            &Write {
                value: ref v1,
                object: ref o1,
                offset: ref off1,
                ..
            },
            &Write {
                value: ref v2,
                object: ref o2,
                offset: ref off2,
                ..
            },
        ) => {
            let res = is_same_event_data(v1, v2);
            if res == "Data" {
                return String::from("data-dev");
            } else if res == "Addr" {
                return String::from("addr-dev");
            } else if !(is_same_object(o1, o2) && off1 == off2) {
                return String::from("addr-dev");
            }
        }
        (&Branch { target: ref t1 }, &Branch { target: ref t2 }) => {
            if t1 != t2 {
                return String::from("control-dev");
            }
        }
        (&Fork { .. }, &Fork { .. }) => {}
        (&Join { .. }, &Join { .. }) => {}
        (&Lock { mutex: ref mtx1 }, &Lock { mutex: ref mtx2 }) => {
            if !is_same_object(mtx1, mtx2) {
                return String::from("addr-dev");
            }
        }
        (&Unlock { mutex: ref mtx1 }, Unlock { mutex: ref mtx2 }) => {
            if !is_same_object(mtx1, mtx2) {
                return String::from("addr-dev");
            }
        }
        (
            &Call {
                name: ref nm1,
                value: ref val1,
                args: ref args1,
            },
            &Call {
                name: ref nm2,
                value: ref val2,
                args: ref args2,
            },
        ) => {
            if nm1 != nm2 {
                return String::from("control-dev");
            }
            let res = is_same_event_data(val1, val2);
            if res == "Data" {
                return String::from("data-dev");
            } else if res == "Addr" {
                return String::from("addr-dev");
            } else {
                if args1.len() != args2.len() {
                    return String::from("control-dev");
                }
                for i in 0..min(args1.len(), args2.len()) {
                    let res = is_same_event_data(&args1[i], &args2[i]);
                    if res == "Data" {
                        return String::from("data-dev");
                    } else if res == "Addr" {
                        return String::from("addr-dev");
                    }
                }
            }
        }
        _ => {
            return String::from("control-dev");
        }
    }
    String::from("no-dev")
}

fn is_same_event_data(data1: &EventData, data2: &EventData) -> String {
    match (data1, data2) {
        (EventData::Value { target: ref t1 }, EventData::Value { target: ref t2 }) => {
            if t1 == t2 {
                return String::from("Ident");
            } else {
                return String::from("Data");
            }
        }
        (
            EventData::Pointer {
                target: ref t1,
                offset: ref o1,
            },
            EventData::Pointer {
                target: ref t2,
                offset: ref o2,
            },
        ) => {
            if is_same_object(t1, t2) && o1 == o2 {
                return String::from("Ident");
            } else {
                return String::from("Addr");
            }
        }
        _ => {
            return String::from("Data");
        }
    }
}

pub fn is_same_object(obj1: &Object, obj2: &Object) -> bool {
    return obj1.id == obj2.id && obj1.size == obj2.size;
}
/*
pub fn compare_sequential(tr1: &SymbolicTrace, tr2: &SymbolicTrace) -> bool {
    let mut i = 0;
    while i != min(tr1.events.len(), tr2.events.len()) {
        let evt1 = &tr1.events[i];
        let evt2 = &tr2.events[i];
        if !is_same_event(evt1, evt2) {
            return false;
        }
        i += 1;
    }

    true
}
*/
pub fn compare_naive(tr1: &SymbolicTrace, tr2: &SymbolicTrace) -> String {
    let mut tmp = Vec::new();
    for th in tr1.thread_naming.values() {
        tmp.push(th);
    }
    //tmp.sort();
    let mut res = tmp.par_iter().filter_map(|th| {
        let test = compare_projection(tr1, tr2, th);
        info!("Thread {} has been compared...", th);
        if test.0 != "no-dev" {
            return Some(test);
        }
        return None;
    }).collect::<Vec<_>>();
    res.sort_by_key(|x| x.1);
    if res.len() == 0 {
        return String::from("no-dev");
    }
    info!("Done with comparison");
    return res[0].0.clone();
/*
    for th in tmp.iter() {
        let mut res = compare_projection(tr1, tr2, th);
        if res != "no-dev" {
            //            res.push_str("-");
            //            res.push_str(th);
            return res;
        }
    }

    String::from("no-dev")
    */
}
pub fn compare_projection(tr1: &SymbolicTrace, tr2: &SymbolicTrace, th: &str) -> (String,u64) {
    let mut i = 0;
    let events1 = tr1.events
        .par_iter()
        .filter_map(|x| {
            if x.thread == th {
                return Some(x.clone());
            } else {
                return None;
            }
        })
        .collect::<Vec<_>>();
    let events2 = &tr2.events
        .par_iter()
        .filter_map(|x| {
            if x.thread == th {
                return Some(x.clone());
            } else {
                return None;
            }
        })
        .collect::<Vec<_>>();
    
    if events1.len() == 0 {
        return (String::from("empty-dev"), 0)
    }
    let mut last_timestamp = events1[0].timestamp.clone().parse().unwrap();
    while i != min(events1.len(), events2.len()) {
        last_timestamp = events1[i].timestamp.clone().parse().unwrap();
        let evt1 = &events1[i];
        let evt2 = &events2[i];
        let mut ignore = true;
        for inj in tr1.injection.iter() {
            if inj.0 < get_long(&*evt1.timestamp) {
                ignore = false;
                break;
            }
        }
        for inj in tr2.injection.iter() {
            if inj.0 < get_long(&*evt2.timestamp) {
                ignore = false;
                break;
            }
        }

        if ignore && !tr1.injection.is_empty() && !tr2.injection.is_empty() {
            i += 1;
            continue;
        }

        let res = is_same_event(evt1, evt2);
        if res != "no-dev" {
            return (res,last_timestamp);
        }
        i += 1;
    }
    if events1.len() != events2.len() {
        return (String::from("data-dev"), last_timestamp);
    }

    (String::from("no-dev"), last_timestamp)
}

/*
pub fn compare_mc(tr1: &SymbolicTrace, tr2: &SymbolicTrace) -> bool {
    let mut _f = Formula::new();
    let graph = build_cfg(tr1);
    let mut tmp = _f.get_cfg_constraint(graph);
    //_f.assert_formula(&tmp);
    //let locks1 = build_lock_pairs(&tr1);
    //tmp = _f.get_lock_constraints(locks1);
    //_f.assert_formula(&tmp);

    //let rw1 = get_all_writes(&tr1);
    
    //tmp = _f.get_read_constraints(rw1);
    //_f.assert_formula(&tmp);

    //tmp = _f.get_total_order_constraint(&tr2.events);
    //_f.assert_formula(&tmp);
    _f.check()
}
*/
