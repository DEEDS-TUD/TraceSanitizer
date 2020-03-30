use log::*;
use petgraph::graphmap::DiGraphMap;
use rayon::prelude::*;
use rsmt2::SmtConf;
use rsmt2::Solver;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::sync::atomic::{AtomicBool, Ordering};
use trace::*;
use utils::*;

pub struct Formula {
    pub solvers: Vec<Solver<()>>,
    variables: HashSet<String>,
    constraints: HashSet<String>,
    benchmark: String
}

impl Formula {
    pub fn new(benchmark: &str) -> Formula {
        Formula {
            solvers: Vec::new(),
            variables: HashSet::new(),
            constraints: HashSet::new(),
            benchmark: String::from(benchmark),
        }
    }

    fn get_solver(&self) -> Solver<()> {
        let parser = ();
        let mut conf = SmtConf::z3();
        //conf.cmd("z3 -in -smt2 parallel.enable=true");
        //        conf.option("parallel.enable=true");

        conf.option("-st");
        let mut solver = conf.spawn(parser).unwrap();
        let mut tee_name = String::from("teefile-");
        tee_name += &*format!("{}", self.benchmark);
        tee_name += ".txt";
        let file = File::create(tee_name).unwrap();

        solver.tee(file);
        solver
        //self.solvers.push(solver);
    }

    pub fn get_cfg_constraint(&mut self, graph: DiGraphMap<&Event, &str>) -> String {
        let mut constr = Vec::new();
        for edge in graph.all_edges() {
            self.add_event(edge.0);
            self.add_event(edge.1);
            constr.push(self.get_single_constraint(edge.0, edge.1));
        }

        self.get_and_constraints(constr)
    }

    fn get_timed_constraint(&self, e1: &Event, e2: &Event) -> Option<String> {
        if e1.timestamp < e2.timestamp {
            return Some(self.get_single_constraint(e2, e1));
        } else if e1.timestamp > e2.timestamp {
            return Some(self.get_single_constraint(e1, e2));
        } else {
            return None;
        }
    }
    pub fn check_nondet(&mut self, deps: &HashMap<&Event, HashSet<&Event>>, at_once: bool) -> bool {
        let mut res = Vec::new();
        for (e1, evs) in deps {
            for e2 in evs {
                if let Some(c) = self.get_timed_constraint(e1, e2) {
                    //if res.contains(&c) {
                    //    warn!("{} is already included!!", c);
                    //}
                    res.push(c);
                }
            }
        }

        //warn!("Number of checks: {}", res.len());
        /*
        if !at_once {
            for r in res.iter() {
                warn!("Solving for: {}", r);
                let actlit = self.solver.get_actlit().unwrap();
                self.solver.assert_act(&actlit, r).unwrap();
                if let Ok(chk) = self.solver.check_sat_act(Some(&actlit)) {
                    warn!("Check: {}", chk);
                    if chk {
                        return chk;
                    }
                }
                self.solver.de_actlit(actlit).unwrap();
            }
            return false;
        }
        */

       
        let flag = AtomicBool::new(false);
        if at_once {
            let constr = self.get_or_constraints(&res);
            res.clear();
            res.push(constr);
//            res.insert(0, constr);
        }
        if let Some(race) = res.par_iter().find_any(|ref x| {
            //warn!("Checking: {}", x);
            let mut solver = self.get_solver();
            for var in self.variables.iter() {
                solver.declare_const(var, "Int");
            }
            for constr in self.constraints.iter() {
                solver.assert(constr);
            }
            solver.assert(x);

            if let Ok(chk) = solver.check_sat() {
                if x.starts_with("(or") {
                    flag.store(chk, Ordering::Relaxed);
                    return true;
                } else {
                    return chk;
                }
            }
            return false;
        }) {
            if race.starts_with("(or") {
                //warn!("Result came from the combined formula");
                return flag.load(Ordering::Relaxed);
            } else {
                return true;
            }
        }
        return false;


    }
    pub fn get_read_constraints(&mut self, writes: &HashMap<&Event, HashSet<&Event>>) -> String {
        let mut rw_val1 = HashMap::new();
        for (k, v) in writes.iter() {
            if let EventType::Read {
                value: ref read, ..
            } = k.data
            {
                let w_val = v.par_iter()
                    .filter(|x| {
                        if let EventType::Write {
                            value: ref write, ..
                        } = x.data
                        {
                            return read == write;
                        }
                        return false;
                    })
                    .collect::<Vec<_>>();
                rw_val1.insert(k, w_val);
            }
        }

        let mut constr = Vec::new();
        for k in writes.keys() {
            if let EventType::Write { .. } = k.data {
                continue;
            }
            let allwrites = writes.get(k).unwrap();
            let allwrites_val = rw_val1.get(k).unwrap();
            let mut tmp = Vec::new();
            for wxv in allwrites_val.iter() {
                for wx in allwrites.iter() {
                    if wx.data == wxv.data {
                        continue;
                    }
                    tmp.push(self.get_or_constraint(wx, wxv, k, wx));
                }
                tmp.push(self.get_single_constraint(wxv, k));
            }
            let s = self.get_and_constraints(tmp);
            if &*s != "" {
                constr.push(s);
            }
        }
        self.get_or_constraints(&constr)
    }

    pub fn get_lock_constraints(&mut self, locks: Vec<(&Event, &Event, &Event, &Event)>) -> String {
        let mut res = Vec::new();
        for lock_pair in locks.iter() {
            res.push(self.get_or_constraint(lock_pair.1, lock_pair.2, lock_pair.3, lock_pair.0));
        }
        let tmp = self.get_and_constraints(res);
        tmp
    }

    fn get_or_constraints(&mut self, mut constrs: &Vec<String>) -> String {
        if constrs.is_empty() {
            return String::new();
        } else if constrs.len() == 1 {
            return constrs[0].to_owned();
        }
        let mut res = String::from("(or ");
        for c in constrs.iter() {
            res.push_str(c);
            res.push_str(" ");
        }
        res.push_str(")");

        res
    }

    pub fn get_total_order_constraint(&mut self, evs: &Vec<Event>) -> String {
        let mut res = String::from("(< ");
        for _ev in evs.iter() {
            // Add event symbols??
            let name = self.get_event_name(_ev);
            res.push_str(&*name);
            res.push_str(" ");
        }
        res.push_str(")");
        res
    }

    pub fn check(&self, solver: &mut Solver<()>) -> bool {
        //self.solver.check_sat();
        if let Ok(chk) = solver.check_sat() {
            println!("{}", chk);
            return chk;
        }
        false
    }

    pub fn assert_formula(&mut self, formula: String) {
        if formula.is_empty() {
            return;
        }
        self.constraints.insert(formula);
        //self.solver.assert(formula);
        //        println!("{:#?}", self.solver.check_sat().unwrap());
    }

    fn get_and_constraints(&self, mut constrs: Vec<String>) -> String {
        if constrs.is_empty() {
            return String::new();
        } else if constrs.len() == 1 {
            return constrs.pop().unwrap();
        }
        let mut res = String::from("(and ");
        for c in constrs.iter() {
            res.push_str(c);
            res.push_str(" ");
        }
        res.push_str(")");

        res
    }

    fn get_event_name(&self, _ev: &Event) -> String {
        let mut res = String::from("e");
        res += "_";

        res.push_str(&*_ev.timestamp);
        res += "__";

        res.push_str(&*_ev.thread);
        res.push_str("__");
        res.push_str(&*_ev.id);
        res.push_str("_");
        res.push_str(&*int_to_string(_ev.th_cnt));

        res
    }

    fn add_event(&mut self, _ev: &Event) {
        let name = self.get_event_name(_ev);
        if !self.variables.contains(&name) {
            //self.solver.declare_const(&name, "Int");
            self.variables.insert(name);
        }
    }

    fn get_single_constraint(&self, _ev1: &Event, _ev2: &Event) -> String {
        let name1 = self.get_event_name(_ev1);
        let name2 = self.get_event_name(_ev2);
        format!("(< {} {})", name1, name2)
    }

    fn get_or_constraint(&self, _ev1: &Event, _ev2: &Event, _ev3: &Event, _ev4: &Event) -> String {
        let name1 = self.get_event_name(_ev1);
        let name2 = self.get_event_name(_ev2);
        let name3 = self.get_event_name(_ev3);
        let name4 = self.get_event_name(_ev4);

        format!("(or (< {} {}) (< {} {}) )", name1, name2, name3, name4)
    }
}
