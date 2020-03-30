pub fn get_hex(number: &str) -> u64 {
    if let Ok(v) = u64::from_str_radix(&*number.to_string(), 16) {
        return v;
    }
    return 133;
}

pub fn get_func_name(func: &str) -> &str {
    let tmp = func.split('-').collect::<Vec<&str>>();
    return tmp[1];
}
pub fn get_int(number: &str) -> u32 {
    return number.to_string().parse().unwrap();
}

pub fn get_long(number: &str) -> u64 {
    return number.to_string().parse().unwrap();
}
pub fn to_hex(number: u64) -> String {
    return format!("{:x}", number);
}

pub fn sign_int_to_string(number: i32) -> String {
    format!("{}", number)
}
pub fn int_to_string(number: u32) -> String {
    format!("{}", number)
}
pub fn long_to_string(number: u64) -> String {
    format!("{}", number)
}
pub fn split_f_name(fname: &str) -> &str {
    let res = fname.split('/').collect::<Vec<&str>>();
    return res[res.len() - 1];
}

pub fn split_at(fname: &str, i: usize) -> &str {
    let res = fname.split('_').collect::<Vec<&str>>();
    return res[i];
}
