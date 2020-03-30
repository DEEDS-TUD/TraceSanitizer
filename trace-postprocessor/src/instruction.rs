use csv::*;
use log::*;
#[derive(Debug)]
pub struct Value {
    pub value: String,
    pub type_id: u8,
    pub type_size: u64,
}

#[derive(Debug)]
pub struct Instruction {
    pub timestamp: u64,
    pub thread_id: u64,
    pub instruction_id: u32,
    pub op_name: String,
    pub value: Value,
    pub operands: Vec<Value>,
}

impl Value {
    pub fn from(value: &str) -> Option<Value> {
        let v: Vec<&str> = value.split('-').collect();
        if v.len() < 3 {
            return None;
        }
        for i in 0..3 {
            if v[i].is_empty() {
                return None;
            }
        }
        if v[0].len() < 2 {
            return None;
        }
        let size: u64 = v[1].to_string().parse().unwrap();
        if v[2].len() as u64 != size * 2 {
            return None;
        }

        Some(Value {
            value: v[2].to_string(),
            type_id: v[0].to_string().parse().unwrap(),
            type_size: v[1].to_string().parse().unwrap(),
        })
    }

    pub fn is_pointer(&self) -> bool {
        return self.type_id == 14;
    }
}

impl Instruction {
    pub fn from(record: &StringRecord) -> Option<Instruction> {
        for i in 0..5 {
            let st = record.get(i).unwrap();
            if st.is_empty() {
                return None;
            }
        }

        if let None = Value::from(record.get(4).unwrap()) {
            return None;
        }

        if record.get(0).unwrap().to_string().len() > 16 {
            return None;
        }

        let mut inst = Instruction {
            timestamp: record.get(0).unwrap().to_string().parse().unwrap(),
            thread_id: record.get(1).unwrap().to_string().parse().unwrap(),
            instruction_id: record.get(2).unwrap().to_string().parse().unwrap(),
            op_name: String::from(record.get(3).unwrap()),
            value: Value::from(record.get(4).unwrap()).unwrap(),
            operands: Vec::new(),
        };
        for i in 5..record.len() {
            let st = record.get(i).unwrap();
            if let Some(v) = Value::from(st) {
                inst.operands.push(v);
            }
        }
        if (inst.is_store() || inst.is_alloca() || inst.is_join()) && inst.operands.len() < 2 {
            return None;
        }
        if (inst.is_load() || inst.is_malloc() || inst.is_free()) && inst.operands.len() < 1 {
            return None;
        }
        if inst.is_branch() && (inst.operands.len() != 1 && inst.operands.len() != 3) {
            return None;
        }
        if inst.is_fork() && inst.operands.len() < 4 {
            return None;
        }
        return Some(inst);
    }

    pub fn get_pointers(&self) -> Vec<&Value> {
        //if !self.is_call() {
        //    error!("Called get_pointers on non-call instruction!");
        //    panic!();
        //}

        let mut _res: Vec<&Value> = Vec::new();
        if self.value.is_pointer() {
            _res.push(&self.value);
        }
        // TODO omit the last operand only for call instruction no??
        for val in self.operands[..self.operands.len()].iter() {
            if val.is_pointer() {
                _res.push(&val);
            }
        }
        return _res;
    }

    pub fn is_lock(&self) -> bool {
        return self.op_name.starts_with("call-pthread_mutex_lock");
    }
    pub fn is_unlock(&self) -> bool {
        return self.op_name.starts_with("call-pthread_mutex_unlock");
    }

    pub fn is_join(&self) -> bool {
        return self.op_name.starts_with("call-pthread_join");
    }
    pub fn is_fork(&self) -> bool {
        return self.op_name.starts_with("call-pthread_create");
    }
    pub fn is_restore_stack(&self) -> bool {
        return self.op_name.starts_with("call-llvm.stackrestore") || self.is_return();
    }

    pub fn is_main(&self) -> bool {
        return self.op_name.starts_with("call-main-d");
    }

    pub fn is_new_stack_frame(&self) -> bool {
        return self.op_name.starts_with("call-llvm.stacksave") || self.is_declared_call();
    }

    pub fn is_undeclared_call(&self) -> bool {
        return self.is_call() && self.op_name.ends_with("-u");
    }
    pub fn is_allocation(&self) -> bool {
        return self.is_alloca() || self.is_malloc() || self.is_znam();
    }

    pub fn is_call(&self) -> bool {
        return self.op_name.starts_with("call-");
    }
    pub fn is_declared_call(&self) -> bool {
        return self.is_call() && self.op_name.ends_with("-d");
    }
    pub fn is_alloca(&self) -> bool {
        return self.op_name.starts_with("alloca");
    }

    pub fn is_branch(&self) -> bool {
        return self.op_name.starts_with("br");
    }
    // probably have to implement a similar method for the free call
    pub fn is_malloc(&self) -> bool {
        return self.op_name.starts_with("call-malloc")
            || return self.op_name.starts_with("call-calloc");
    }

    pub fn is_znam(&self) -> bool {
        return self.op_name.starts_with("call-_Znam");
    }

    pub fn is_deallocation(&self) -> bool {
        return self.is_free() || self.is_znam();
    }
    pub fn is_znwm(&self) -> bool {
        return self.op_name.starts_with("call-_Znwm");
    }
    pub fn is_free(&self) -> bool {
        return self.op_name.starts_with("call-free");
    }
    pub fn is_return(&self) -> bool {
        return self.op_name.starts_with("ret");
    }

    pub fn is_load(&self) -> bool {
        return self.op_name.starts_with("load");
    }

    pub fn is_store(&self) -> bool {
        return self.op_name.starts_with("store");
    }
}
