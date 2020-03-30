use error::*;
use fileio::*;
use instruction::*;
use log::*;
use object::*;
use petgraph::graphmap::DiGraphMap;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};
use utils::*;

#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum EventData {
    Value { target: String },
    Pointer { target: Arc<Object>, offset: u64 },
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Event {
    pub data: EventType,
    pub timestamp: String,
    pub thread: String,
    pub id: String,
    pub inst: usize,
    pub th_cnt: u32,
    op_code: String,
    // necessary for forks and joins
    value: String,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum EventType {
    DummyEvent,
    Read {
        value: EventData,
        object: Arc<Object>,
        //is_ptr: bool,
        //val_ptr: Option<Arc<Object>>,
        offset: u64,
        concrete: String, //        val_offset: Option<u64>
    },
    Write {
        value: EventData,
        object: Arc<Object>,
        //is_ptr: bool,
        //val_ptr: Option<Arc<Object>>,
        offset: u64,
        concrete: String, //        val_offset: Option<u64>
    },
    Branch {
        target: u32,
    },
    Lock {
        mutex: Arc<Object>,
    },
    Unlock {
        mutex: Arc<Object>,
    },
    Fork {
        creator: String,
        createe: String,
    },
    Join {
        joiner: String,
        joinee: String,
    },
    Call {
        name: String,
        value: EventData,
        args: Vec<EventData>,
    }, //StackAllocation { object: Object, size: u32 },
       //HeapAllocation { object: Object, size: u32 },
       //Call {}
}

#[derive(Debug)]
pub struct SymbolicTrace {
    pub id: String,
    pub events: Vec<Event>,
    pub global_events: Vec<Event>,
    pub objects: Vec<Arc<Object>>,
    pub thread_naming: HashMap<u64, String>,
    pub thread_hiearchy: HashMap<String, Vec<String>>,
    pub thread_mapping: HashMap<u64, String>,
    pub injection: Vec<(u64, u32)>,
    pub symb_time: Duration,
    pub ret_code: String,
    pub output_hash: String,
}

impl Event {
    fn from(e_data: EventType, inst: &Instruction, pos: usize, th: &str, cnt: u32) -> Event {
        Event {
            data: e_data,
            timestamp: inst.timestamp.to_string(),
            thread: String::from(th),
            id: inst.instruction_id.to_string().parse().unwrap(),
            inst: pos,
            th_cnt: cnt,
            op_code: inst.op_name.clone(),
            value: inst.value.value.clone(),
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let res = format!("{:#?}", self);
        return write!(f, "{}", res);
    }
}

impl fmt::Display for EventData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use EventData::*;
        let mut res = String::new();
        match &self {
            Pointer {
                target: ref t,
                offset: ref off,
            } => {
                res.push_str(&*t.id);
                res.push_str("-");
                res.push_str(&*long_to_string(off.clone()));
            }
            Value { target: ref t } => {
                res.push_str(&*t);
            }
        }
        return write!(f, "{}", res);
    }
}

impl SymbolicTrace {
    fn new(id: &str) -> SymbolicTrace {
        SymbolicTrace {
            id: String::from(id),
            events: Vec::new(),
            global_events: Vec::new(),
            objects: Vec::new(),
            thread_naming: HashMap::new(),
            thread_hiearchy: HashMap::new(),
            thread_mapping: HashMap::new(),
            injection: Vec::new(),
            symb_time: Duration::new(0, 0),
            ret_code: String::new(),
            output_hash: String::new(),
        }
    }

    pub fn drain_content(&mut self) {
        self.objects.clear();
        self.events.clear();
        self.thread_naming.clear();
        self.thread_hiearchy.clear();
        self.thread_mapping.clear();
    }
    pub fn from(fname: &str) -> Result<SymbolicTrace, Box<Error>> {
        let start = Instant::now();
        let f_name = split_f_name(&*fname);
        let mut trace = SymbolicTrace::new(f_name);
        //        println!("{:#?}", trace.get_id_info());
        trace.load_injections(fname)?;

        trace.load_ret_code(fname)?;
        trace.load_output_hash(fname)?;

        let instructions = read_instructions(fname)?;

        trace.load_mapping(fname, instructions[0].thread_id)?;

        trace.build_memory(&instructions, fname)?;

        trace.build_events(&instructions)?;

        trace.symb_time = start.elapsed();
        //info!("{}", trace.get_llfi_trace()?);
        //        println!("{:#?}", trace.objects);
        Ok(trace)
    }

    fn get_llfi_line(&self, _ev: &Event) -> Result<(String, String), Box<Error>> {
        use EventType::*;
        let mut content = String::new();
        content.push_str("ID: ");
        content.push_str(&*_ev.id);
        content.push_str("\tOPCode: ");
        content.push_str(&*_ev.op_code);
        content.push_str("\tValue: ");
        let mut c_content = format!("{}{}", &*content, &*_ev.value);
        //content.push_str(&*_ev.value);

        match &_ev.data {
            &Read { value: ref val, .. } | &Write { value: ref val, .. } => {
                content.push_str(&*format!("{}", val));
            }
            &Lock { mutex: ref mtx } | &Unlock { mutex: ref mtx } => {
                content.push_str(&*mtx.id);
            }
            &Fork { .. } | &Join { .. } => {
                content.push_str(&*_ev.value);
            }
            &Call { value: ref val, .. } => {
                content.push_str(&*format!("{}", val));
            }
            &Branch { target: ref t } => content.push_str(&*int_to_string(t.clone())),
            _ => {
                return Err(Box::new(SanError::new(
                    "invalid event in the llfi trace...",
                )));
            }
        };
        content.push_str("\n");
        c_content.push_str("\n");
        return Ok((content, c_content));
    }
    pub fn get_llfi_trace(&self, max_span: u32) -> Result<(String, String), Box<Error>> {
        let mut content = String::new();
        let mut s_content = String::new();
        let mut inst_cnt = 0;
        let mut start = false;
        let mut span_cnt = 0;
        for _ev in self.events.iter() {
            inst_cnt += 1;

            if span_cnt < max_span && (start || !self.is_injected()) {
                let line = self.get_llfi_line(_ev)?;
                content.push_str(&*line.1);
                s_content.push_str(&*line.0);
                span_cnt += 1;
            }
            if self.is_injected() && self.injection[0].0 < get_long(&*_ev.timestamp) && !start {
                let mut tmp = String::from("#TraceStartInstNumber: ");
                tmp.push_str(&*sign_int_to_string(inst_cnt));
                tmp.push_str("\n");
                content.push_str(&*tmp);
                s_content.push_str(&*tmp);
                start = true;
                let line = self.get_llfi_line(_ev)?;
                content.push_str(&*line.1);
                s_content += &*line.0;
                span_cnt += 1;
            }
        }
        return Ok((content, s_content));
    }
    pub fn get_id_info(&self) -> (String, String, String) {
        let res = self.id.clone();
        let bench = res.split("_trace").collect::<Vec<&str>>()[0];
        let tmp = res.split('.').collect::<Vec<&str>>()[1];
        let tmp = tmp.split('-').collect::<Vec<&str>>();
        let fm = tmp[0];
        let cnt = tmp[1];
        return (String::from(bench), String::from(fm), String::from(cnt));
    }

    fn load_output_hash(&mut self, fname: &str) -> Result<(), Box<Error>> {
        let mut oh_name = String::from(&*fname);
        oh_name += "_outhash";
        self.output_hash = read_output_hash(&*oh_name)?;    
        Ok(())
    }
    fn load_ret_code(&mut self, fname: &str) -> Result<(), Box<Error>> {
        let mut rc_name = String::from(&*fname);
        rc_name += "_retc";
        self.ret_code = read_ret_code(&*rc_name)?;
        Ok(())
    }
    fn load_injections(&mut self, fname: &str) -> Result<(), Box<Error>> {
        let mut i_name = String::from(&*fname);
        i_name += "_faultinj";
        self.injection = read_injections(&*i_name)?;
        Ok(())
    }
    fn update_value(&self, val: String, thread_table: &HashMap<String, String>) -> String {
        //TODO: add code for the hashmap of threads!
        if let Some(ret) = thread_table.get(&get_hex(&val).to_string()) {
            return ret.clone();
        }
        return val;
    }

    pub fn is_injected(&self) -> bool {
        !self.injection.is_empty()
    }
    fn build_memory(
        &mut self,
        instructions: &Vec<Instruction>,
        fname: &str,
    ) -> Result<(), Box<Error>> {
        let mut g_name = String::from(&*fname);
        g_name += "_globals";
        read_globals(&*g_name, instructions.len(), &mut self.objects)?;
        self.load_objects(instructions)?;
        Ok(())
    }

    fn load_objects(&mut self, instructions: &Vec<Instruction>) -> Result<(), Box<Error>> {
        let mut _heap: (Vec<Object>, u32) = (Vec::new(), 0);
        let mut _idxs: HashMap<u64, u32> = HashMap::new();
        let mut _stacks: HashMap<u64, Vec<Object>> = HashMap::new();
        let mut _sps: HashMap<u64, Vec<usize>> = HashMap::new();
        let mut last_instr: HashMap<String, usize> = HashMap::new();

        for n in self.thread_naming.keys() {
            _idxs.insert(n.to_owned(), 0);
            _stacks.insert(n.to_owned(), Vec::new());
            _sps.insert(n.to_owned(), Vec::new());
            //last_instr.insert(self.thread_naming.get(n).unwrap().to_owned(), 0);
            last_instr.insert(self.thread_naming.get(n).ok_or(SanError::new("thread mapping problem"))?.to_owned(), 0);
        }
        /*
        if _idxs.len() == 0 && instructions.len() > 0 {
            _idxs.insert(instructions[0].thread_id, 0);
            _stacks.insert(instructions[0].thread_id, Vec::new());
            _sps.insert(instructions[0].thread_id, Vec::new());
        }
        */
        for (pos, _e) in instructions.iter().enumerate() {
            let _thread = _e.thread_id;
//            last_instr.insert(self.thread_naming.get(&_thread).unwrap().to_owned(), pos);
            last_instr.insert(self.thread_naming.get(&_thread).ok_or(SanError::new("thread mapping problem"))?.to_owned(), pos);
            if _e.is_new_stack_frame() {
                let mut _sp = _sps.get_mut(&_thread)
                    .ok_or(SanError::new("stack pointer problem"))?;
                _sp.push(_stacks[&_thread].len());
            }
            if _e.is_allocation() {
                let _idx = _idxs
                    .get(&_thread)
                    .ok_or(SanError::new("object id problem"))? + 1;
                let _obj = Object::from(
                    _e,
                    _idx,
                    pos,
                    true,
                    self.thread_naming
                        .get(&_thread)
                        .ok_or(SanError::new("thread naming problem"))?,
                ).remove(0);
                if _e.is_alloca() {
                    let _stack = _stacks
                        .get_mut(&_thread)
                        .ok_or(SanError::new("stack pointer problem"))?;
                    _stack.push(_obj);
                    if _e.instruction_id == 374 {
                        //println!("CHECK: {:#?}", _stack);
                    }
                    _idxs.insert(_thread, _idx);
                } else {
                    // Heap allocation!
                    _heap.0.push(_obj);
                    _heap.1 += 1;
                }
            }
            if _e.is_free() {
                let mut i = _heap.0.len();
                while i > 0 {
                    if _heap.0[i - 1].address == get_hex(&*_e.operands[0].value) {
                        let mut val = _heap.0.remove(i - 1);
                        val.update_validity(pos);
                        self.objects.push(Arc::new(val));
                        break;
                    }
                    i -= 1;
                }
            }
            if _e.is_restore_stack() {
                let _stack = _stacks
                    .get_mut(&_thread)
                    .ok_or(SanError::new("stack pointer problem"))?;
                let _sp = _sps.get_mut(&_thread)
                    .ok_or(SanError::new("coudln't load trace..."))?
                    .pop()
                    .ok_or(SanError::new("couldn't load trace..."))?;
                let mut i = _stack.len();
                while i > _sp {
                    let mut val = _stack.pop().ok_or(SanError::new("couldn't load trace..."))?;

                    val.update_validity(pos);
                    self.objects.push(Arc::new(val));
                    i -= 1;
                }
            }
        }

        info!("Checking the object stack...");
        for (k, v) in _stacks.iter_mut() {
            let mut i = 0;
            let l = v.len();
            while i < l {
                i += 1;
                let mut obj = v.pop().ok_or(SanError::new("couldn't load trace..."))?;

                let l_instr = last_instr.get(&obj.owner).ok_or(SanError::new("last_instr problem"))?;
                obj.update_validity(l_instr.clone());
                //obj.update_validity(instructions.len());
                if obj.construction == 3050 {
                    warn!("CHeck this {:#?}", obj);
                }

                self.objects.push(Arc::new(obj));
            }
        }

        let mut i = 0;
        let l = _heap.0.len();
        while i < l {
            i += 1;
            let mut obj = _heap
                .0
                .pop()
                .ok_or(SanError::new("couldn't load trace..."))?;
            obj.update_validity(instructions.len());
            self.objects.push(Arc::new(obj));
        }

        info!("Adding un-indentified pointers...");
        /*
        for (pos, _e) in instructions.iter().enumerate() {
            if (((_e.is_undeclared_call() && !_e.is_allocation()) || _e.is_load() || _e.is_store())
                && _e.get_pointers().len() != 0) || _e.is_main()
            {
                let mut _objs: Vec<Object> = Object::from(
                    _e,
                    _heap.1,
                    pos,
                    false,
                    self.thread_naming.get(&_e.thread_id).ok_or(SanError::new("couldn't load trace..."))?,
                );

                let mut i = 0;
                let l = _objs.len();
                while i < l {
                    i += 1;
                    let mut _obj = _objs.pop().ok_or(SanError::new("couldn't load trace..."))?;
                    if let None = self.get_object(_obj.address, pos) {
                        _obj.update_validity(instructions.len());
                        self.objects.push(Arc::new(_obj));
                        _heap.1 += 1;
                    }
                }
            }
        }
        */
        if _heap.0.len() > 0 {
            warn!("heap is not empty!");
        }
        info!("Finished building the memory model...");
        info!("Number of objects {}", self.objects.len());

        return Ok(());
    }

    fn load_mapping(&mut self, fname: &str, root: u64) -> Result<(), Box<Error>> {
        let mut m_mapping = String::from(fname);
        m_mapping += "_mapping";
        let (hiearch, naming) = read_thread_graph(&*m_mapping, root)?;
        self.thread_naming = naming;
        self.thread_hiearchy = hiearch;
        let mut name = String::from(fname);
        name += "_logical_mapping";
        self.thread_mapping = read_logical_mapping(&*name)?;
        Ok(())
    }

    fn get_offset(&self, addr: u64, obj: &Arc<Object>) -> u64 {
        return addr - obj.address;
    }
    fn build_events(&mut self, instructions: &Vec<Instruction>) -> Result<(), Box<Error>> {
        info!("Building symbolic trace...");
        let mut seen = HashSet::new();
        let mut iters = HashMap::new();
        let mut counters = HashMap::new();
        let mut active_map = HashMap::new();
        let mut cache = HashMap::new();
        for th in self.thread_naming.values() {
            counters.insert(th, 0);
        }
        for (k, v) in self.thread_hiearchy.iter() {
            iters.insert(k, v.iter());
        }

        for (pos, inst) in instructions.iter().enumerate() {
            let mut ee = EventType::DummyEvent;
            let th = self.thread_naming
                .get(&inst.thread_id)
                .ok_or(SanError::new("couldn't load trace..."))?;
            let cnt = *counters
                .get_mut(th)
                .ok_or(SanError::new("couldn't load trace..."))?;

            //self.add_object(inst, pos, &mut cache, instructions.len(), cnt.clone());
            {
                if (((inst.is_undeclared_call() && !inst.is_allocation()) && !inst.is_deallocation()
                    || inst.is_load() || inst.is_store())
                    && inst.get_pointers().len() != 0) || inst.is_main()
                {
                    let mut _objs: Vec<Object> = Object::from(
                        &inst,
                        cnt.clone(),
                        pos,
                        false,
                        self.thread_naming
                            .get(&inst.thread_id)
                            .ok_or(SanError::new("couldn't load trace..."))?,
                    );

                    let mut i = 0;
                    let l = _objs.len();
                    while i < l {
                        i += 1;
                        let mut _obj = _objs.pop().ok_or(SanError::new("couldn't load trace..."))?;
                        if let None = self.get_object(_obj.address, pos, &mut cache) {
                            _obj.update_validity(instructions.len());
                            self.objects.push(Arc::new(_obj));
                        }
                    }
                }
            }
            if inst.is_load() {
                let tmp = get_hex(&*inst.operands[0].value);
                //                let tmp = u64::from_str_radix(&*inst.operands[0].value.to_string(), 16).ok_or(SanError::new("couldn't load trace..."))?;
                //                let mut v_tr = None;
                let mut val = EventData::Value {
                    target: self.update_value(inst.value.value.to_string(), &active_map),
                };
                if inst.value.is_pointer() {
                    let tmp = get_hex(&*inst.value.value);
                    if let Some(obj1) = self.get_object(tmp, pos, &mut cache) {
                        let off = self.get_offset(tmp, &obj1);
                        //off_tr = Some(self.get_offset(tmp, &obj1));
                        //                        v_tr = Some(obj1);
                        val = EventData::Pointer {
                            target: obj1,
                            offset: off,
                        };
                    }
                }
                if let Some(obj) = self.get_object(tmp, pos, &mut cache) {
                    let off = self.get_offset(tmp, &obj);
                    ee = EventType::Read {
                        value: val,
                        object: obj,
                        //                        is_ptr: inst.value.is_pointer(),
                        //                        val_ptr: v_tr,
                        offset: off,
                        concrete: String::from(&*inst.operands[0].value)
                        //                        val_offset: off_tr,
                    };
                } else {
                    seen.insert(to_hex(tmp));
                    warn!("Address {} couldn't been identified", tmp);
                    warn!("{:#?}", inst);
                }
            } else if inst.is_store() {
                let tmp = get_hex(&*inst.operands[1].value);
                //let mut v_tr = None;
                //                let mut off_tr = None;
                let mut val = EventData::Value {
                    target: self.update_value(inst.operands[0].value.to_string(), &active_map),
                };
                if inst.operands[0].is_pointer() {
                    let tmp = get_hex(&*inst.operands[0].value);
                    if let Some(obj1) = self.get_object(tmp, pos, &mut cache) {
                        let off = self.get_offset(tmp, &obj1);
                        //                        off_tr = Some(self.get_offset(tmp, &obj1));
                        val = EventData::Pointer {
                            target: obj1,
                            offset: off,
                        };
                        //                        v_tr = Some(obj1);
                    }
                }
                if let Some(obj) = self.get_object(tmp, pos, &mut cache) {
                    let off = self.get_offset(tmp, &obj);
                    ee = EventType::Write {
                        value: val,
                        object: obj,
                        //                        is_ptr: inst.operands[0].is_pointer(),
                        //                        val_ptr: v_tr,
                        offset: off,
                        concrete: String::from(&*inst.operands[1].value)
                        //                        val_offset: off_tr,
                    };
                } else {
                    seen.insert(to_hex(tmp));
                    warn!("Address {} couldn't been identified", tmp);
                }
            } else if inst.is_branch() {
                let mut _trg = 0;
                if inst.operands.len() == 1 {
                    _trg = u32::from_str_radix(&*inst.operands[0].value.to_string(), 16)?;
                } else {
                    let cond = u32::from_str_radix(&*inst.operands[0].value.to_string(), 16)?;
                    if cond == 0 {
                        _trg = u32::from_str_radix(&*inst.operands[2].value.to_string(), 16)?;
                    } else {
                        _trg = u32::from_str_radix(&*inst.operands[1].value.to_string(), 16)?;
                    }
                }
                ee = EventType::Branch { target: _trg };
            } else if inst.is_fork() {
                let n = self.thread_naming
                    .get(&inst.thread_id)
                    .ok_or(SanError::new("couldn't load trace..."))?;
                //TODO sometimes the program crashes before thread creation is logged proprely
                if let Some(ctt) = iters
                    .get_mut(n)
                    .ok_or(SanError::new("couldn't load trace..."))?
                    .next()
                {
                    let mut j = self.thread_naming.len() as u64 + 1;
                    for (k, v) in self.thread_naming.iter() {
                        if v == ctt {
                            j = *k;
                            break;
                        }
                    }

                    let tid = self.thread_mapping
                        .get(&j)
                        .ok_or(SanError::new("couldn't load trace..."))?
                        .clone();
                    active_map.insert(tid, ctt.clone());
                    ee = EventType::Fork {
                        creator: n.clone(),
                        createe: ctt.clone(),
                    };
                } else {
                    break;
                    info!("Thread ID: {}", inst.thread_id);
                }
            //                let neigh = symb_tr.get_ordered_neighbours(inst.thread_id);
            //ee = EventType::Fork {
            //    creator: *symb_tr.threads.get(&inst.thread_id).ok_or(SanError::new("couldn't load trace..."))?,
            //    createe: ,

            //}
            } else if inst.is_join() {
                let tid = get_hex(&*inst.operands[0].value).to_string();
                if let None = active_map.get(&tid) {
                    break;
                }
                ee = EventType::Join {
                    //joiner: self.thread_naming
                    //    .get(&get_hex(&*inst.operands[0].value))
                    //    .ok_or(SanError::new("couldn't load trace..."))?
                    //    .clone(),
                    joiner: active_map
                        .get(&tid)
                        .ok_or(SanError::new("couldn't load trace..."))?
                        .clone(),
                    joinee: self.thread_naming
                        .get(&inst.thread_id)
                        .ok_or(SanError::new("couldn't load trace..."))?
                        .clone(),
                };
            } else if inst.is_lock() {
                if let Some(mtx) =
                    self.get_object(get_hex(&*inst.operands[0].value), pos, &mut cache)
                {
                    ee = EventType::Lock { mutex: mtx };
                } else {
                    warn!("Mutex object coudln't be identified");
                }
            } else if inst.is_unlock() {
                if let Some(mtx) =
                    self.get_object(get_hex(&*inst.operands[0].value), pos, &mut cache)
                {
                    ee = EventType::Unlock { mutex: mtx };
                } else {
                    warn!("Mutex object coudln't be identified");
                }
            } else if inst.is_undeclared_call() {
                let mut arguments = Vec::new();
                let mut val = EventData::Value {
                    target: self.update_value(inst.value.value.to_string(), &active_map),
                };
                if inst.value.is_pointer() {
                    let tmp = get_hex(&*inst.value.value);
                    if let Some(obj1) = self.get_object(tmp, pos, &mut cache) {
                        let off = self.get_offset(tmp, &obj1);
                        val = EventData::Pointer {
                            target: obj1,
                            offset: off,
                        };
                    }
                }
                for arg in inst.operands.iter() {
                    let mut val = EventData::Value {
                        target: self.update_value(arg.value.to_string(), &active_map),
                    };
                    if arg.is_pointer() {
                        let tmp = get_hex(&*arg.value);
                        if let Some(obj1) = self.get_object(tmp, pos, &mut cache) {
                            let off = self.get_offset(tmp, &obj1);
                            val = EventData::Pointer {
                                target: obj1,
                                offset: off,
                            };
                        }
                    }
                    arguments.push(val);
                }

                ee = EventType::Call {
                    name: String::from(get_func_name(&*inst.op_name)),
                    value: val,
                    args: arguments,
                };
            }
            match ee {
                EventType::DummyEvent => {}
                _ => {
                    //let th = self.thread_naming.get(&inst.thread_id).ok_or(SanError::new("couldn't load trace..."))?;
                    //let cnt = *counters.get_mut(th).ok_or(SanError::new("couldn't load trace..."))?;
                    let ev = Event::from(
                        ee,
                        &inst,
                        pos,
                        self.thread_naming
                            .get(&inst.thread_id)
                            .ok_or(SanError::new("couldn't load trace..."))?,
                        cnt as u32,
                    );
                    counters.insert(th, cnt + 1);
                    self.events.push(ev);
                }
            };
        }

        return Ok(());
    }

    pub fn get_object(
        &self,
        addr: u64,
        pos: usize,
        cache: &mut HashMap<u64, Vec<Arc<Object>>>,
    ) -> Option<Arc<Object>> {
        if addr == 0 {
            return Some(Arc::clone(&self.objects[0]));
        }
        if let Some(objs) = cache.get(&addr) {
            if let Some(obj) = objs.par_iter().find_any(|ref x| {
                addr >= x.address
                    && addr < x.address + x.size
                    && pos >= x.construction
                    && pos <= x.destruction
            }) {
                return Some(Arc::clone(obj));
            }
        }
        if let Some(obj) = self.objects.par_iter().find_any(|ref x| {
            addr >= x.address
                && addr < x.address + x.size
                && pos >= x.construction
                && pos <= x.destruction
        }) {
            if let None = cache.get(&addr) {
                cache.insert(addr, Vec::new());
            }
            cache.get_mut(&addr).unwrap().push(Arc::clone(obj));
            return Some(Arc::clone(obj));
        }
        return None;
    }
}
