use csv::*;
use instruction::*;
use utils::*;
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Object {
    pub id: String,
    pub is_stack: bool,
    pub size: u64,
    pub address: u64,
    pub owner: String,
    pub construction: usize,
    pub destruction: usize,
    pub identified: bool,
    pub hex_addr: String,
    pub inst_id: u32,
}
impl Object {
    pub fn from_global(record: &StringRecord) -> Object {
        let val = record.get(1).unwrap();
        let tmp: Vec<&str> = val.split('-').collect();

        let address = get_hex(tmp[2]);
        Object {
            id: record.get(0).unwrap().to_string(),
            is_stack: false,
            size: record.get(2).unwrap().to_string().parse().unwrap(),
            address: address,
            owner: String::from("0"),
            construction: 0,
            destruction: 1,
            identified: true,
            hex_addr: String::from(tmp[2]),
            inst_id: 0,
        }
    }
    pub fn get_null() -> Object {
        Object {
            id: String::from("NULL"),
            is_stack: false,
            size: 0,
            address: 0,
            owner: String::from("0"),
            construction: 0,
            destruction: 1,
            identified: true,
            hex_addr: String::from("00000000000"),
            inst_id: 0,
        }
    }
    pub fn from(
        inst: &Instruction,
        idx: u32,
        pos: usize,
        identified: bool,
        owner: &str,
    ) -> Vec<Object> {
        let mut _res = Vec::new();
        let i: &str = &idx.to_string()[..];
        let mut tmp: String = "v".to_string();
        tmp.push_str(i);
        let mut address = get_hex(&*inst.value.value);
        let mut size = inst.value.type_size;
        if (((inst.is_undeclared_call() && !inst.is_allocation())
            || inst.is_load()
            || inst.is_store()) && inst.get_pointers().len() != 0) || inst.is_main()
        {
            let _vals = inst.get_pointers();
            //warn!("Values: {:#?}", _vals);
            let mut j = 0;
            for v in _vals.iter() {
                let i: &str = &(idx + j).to_string()[..];
                tmp = "v".to_string();
                tmp.push_str(i);
                //                warn!("CHECK: {:#?}", inst);
                address = get_hex(&*v.value);

                size = 8;
                _res.push(Object {
                    id: tmp,
                    is_stack: false,
                    size: size,
                    address: address,
                    owner: String::from(owner),
                    construction: pos,
                    destruction: pos,
                    identified: identified,
                    hex_addr: String::from(&*v.value),
                    inst_id: inst.instruction_id,
                });
                j += 1;
            }
            return _res;
        }
        if inst.is_malloc() || inst.is_znam() {
            size = get_hex(&*inst.operands[0].value);
        } else if inst.is_alloca() {
            size = get_hex(&*inst.operands[1].value) * get_hex(&*inst.operands[0].value);
        }
        /*
        else if inst.is_store() {
            address = get_hex(&*inst.operands[0].value);
            size = 1;
        }
        */
        let obj = Object {
            id: tmp,
            is_stack: inst.is_alloca(),
            size: size,
            address: address,
            owner: String::from(owner),
            construction: pos,
            destruction: pos,
            //is pointer?
            identified: identified,
            hex_addr: String::from(&*inst.value.value),
            inst_id: inst.instruction_id,
        };

        _res.push(obj);
        _res
    }

    pub fn update_validity(&mut self, pos: usize) {
        self.destruction = pos;
    }
}
