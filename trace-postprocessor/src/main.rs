#![macro_use]
extern crate argparse;
extern crate badlog;
extern crate csv;
extern crate log;
extern crate pbr;
extern crate petgraph;
extern crate rayon;
extern crate rsmt2;

mod comparison;
mod fileio;
mod instruction;
//mod mc;
mod error;
mod nondet;
mod object;
mod smt;
mod trace;
mod utils;

use argparse::{ArgumentParser, Store, StoreOption, StoreTrue};
use comparison::*;
#[allow(unused_imports)]
use log::*;
use pbr::ProgressBar;
//use mc::*;
use error::SanError;
use fileio::*;
use nondet::is_deterministic;
use rayon::prelude::*;
use std::cmp::min;
use std::env::var_os;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use trace::*;
// Not needed for now...
//fn get_dec<T: Integer + Unsigned>(number: &str) -> T {
//
//}
fn format_time(val: &Duration) -> String {
    let sec = val.as_secs() as u128 * 10u128.pow(9);
    let nano = val.subsec_nanos() as u128;
    format!("{}", (sec + nano) as u128)
}

fn do_comparison(
    golden_run: String,
    faulty_runs: Vec<String>,
    nondet: bool,
    benchmark: &str
) -> Result<(SymbolicTrace, Vec<SymbolicTrace>, Vec<Vec<String>>), Box<Error>> {
    let mut pb = Arc::new(Mutex::new(ProgressBar::new(faulty_runs.len() as u64 + 1)));
    let mut golden_trace = SymbolicTrace::from(&*golden_run)?;
    warn!("Golden trace has been build...");
    if nondet {
        let start = Instant::now();
        let (result, num_deps, g_evs, g_objs, solving_time) = is_deterministic(&mut golden_trace, benchmark);
        let end = start.elapsed();
        // create result record
        // TODO: Add the number of objects involved in a global dependency
        let mut res_record = Vec::new();
        res_record.push(format!("{}", result));
        res_record.push(format!("{}", g_evs));
        res_record.push(format!("{}", g_objs));
        res_record.push(format!("{}", num_deps));
        res_record.push(format!("{}", format_time(&solving_time))); 
        res_record.push(format!("{}", format_time(&end)));
        write_check_results(&*format!("{}.reversibility-check.csv", benchmark), res_record)?;        
        if !result{
            warn!("Golden trace is not deterministic...");
            info!("Stopping the comparison!");
            return Err(Box::new(SanError::new("Goldenrun is not deterministic")));
        } else {
            warn!("golden trace is deterministic");
            return Err(Box::new(SanError::new("Golden run is deterministic")));
        }
    }
    pb.lock().unwrap().inc();
    //let mut faulty_traces = Arc::new(Mutex::new(Vec::new()));
    //let mut faulty_runs = Arc::new(Mutex::new(faulty_runs));

    let faulty_traces = faulty_runs
        .par_iter()
        .filter_map(|_f| {
            pb.lock().unwrap().inc();

            warn!("Dealing with {}", _f);
            if let Ok(tr) = SymbolicTrace::from(_f) {
                return Some(tr);
            } else {
                return None;
            }
        })
        .collect::<Vec<_>>();

    pb.lock().unwrap().finish_print("Done!");

    pb = Arc::new(Mutex::new(ProgressBar::new(faulty_traces.len() as u64)));

    let results = faulty_traces
        .par_iter()
        .map(|_f| {
            warn!("Start comparison...");
            let start = Instant::now();
            let res = compare_naive(&golden_trace, _f);
            let end = start.elapsed();
            let inf = _f.get_id_info();
            let mut rec = Vec::new();
            rec.extend_from_slice(&[
                inf.0,
                inf.1,
                inf.2,
                res.clone(),
                format!("{}", _f.output_hash != golden_trace.output_hash),
                _f.ret_code.clone(),
                (_f.is_injected()).to_string(),
                format_time(&_f.symb_time),
                format_time(&end),
            ]);
            pb.lock().unwrap().inc();
            return rec;
        })
        .collect::<Vec<_>>();

    pb.lock().unwrap().finish_print("Done");

    Ok((golden_trace, faulty_traces, results))
}

fn exec_diff_script(g_file: &str, f_file: &str) -> Option<i32> {
    let script = "../llfi/tools/tracediff.py";
    if let Ok(stat) = Command::new(script)
        .arg("--quick")
        .arg(g_file)
        .arg(f_file)
        .status()
    {
        //warn!("{}", stat);
        return stat.code();
    }
    return None;
}

fn get_result_code(code: Option<i32>) -> String {
    match code {
        Some(0) => String::from("no-dev"),
        Some(2) => String::from("known-bug"),
        Some(23) => String::from("control-dev"),
        Some(24) => String::from("data-dev"),
        _ => String::from("error"),
    }
}

fn get_files(dir: &Vec<String>, fm: i32, is_fi: i32, max_comp: Option<u32>, offset: u32) -> Vec<String> {
    let mut files = Vec::new();
    if is_fi >= 0 {
        let fm = format!("trace.{}-", fm);
        let tmp_fls = dir.iter()
            .filter_map(|f| {
                if f.contains(&*fm) && f.chars().skip(f.len() - 1).take(1).collect::<String>().parse::<u32>().is_ok() {
                    return Some(f.clone());
                } else {
                    return None;
                }
            })
            .collect::<Vec<_>>();
        files.extend_from_slice(&*tmp_fls);
    } else {
        let fm = String::from("trace.0-0");
        let tmp_fls = dir.iter()
            .filter_map(|f| {
                if !f.contains(&*fm) {
                    return Some(f.clone());
                } else {
                    return None;
                }
            })
            .collect::<Vec<_>>();
        files.extend_from_slice(&*tmp_fls);
        // files.extend_from_slice(&*dir);
    }
    let mut m = files.len() - offset as usize;
    if let Some(x) = max_comp {
        m = min(x, m as u32) as usize;
    }
    return files[offset as usize..offset as usize + m].to_vec();
}
fn start(
    benchmark: &str,
    is_fi: i32,
    is_seq: bool,
    is_llfi_comp: bool,
    is_overwrite: bool,
    is_append: bool,
    max_comp: Option<u32>,
    max_span: Option<u32>,
    nondet: bool,
    offset: u32
) {
    let mut dir = String::from("ressources");
    if let Some(v) = var_os("VDATA") {
        dir = v.into_string().unwrap();
    }
    info!("directory is set to {}", dir);
    let seq_or_pth = if is_seq { "serial" } else { "pthread" };
    let mut base_dir = String::from(&*dir);
    base_dir += "/";
    base_dir += benchmark;
    base_dir += "/";
    base_dir += benchmark;
    base_dir += "-";
    base_dir += seq_or_pth;

    let mut g_dir = String::from(&*base_dir);
    g_dir += "/gr/raw-traces";

    let mut f_dir = String::from(&*base_dir);
    f_dir += "/fi/raw-traces";

    let mut golden_run = String::from(&*g_dir);
    golden_run += "/";
    golden_run += benchmark;
    golden_run += "_trace.0-0";

    info!("Golden run: {}", golden_run);
    let mut faulty_runs = Vec::new();

    let paths = fs::read_dir(if is_fi >= 0 { &*f_dir } else { &*g_dir });
    if let Err(_) = &paths {
        warn!("Couldn't load directory {}", f_dir);
    }

    let paths = paths.unwrap();
    let names = paths
        .filter_map(|f| {
            let tmp: String = f.unwrap().path().into_os_string().into_string().unwrap();
            if tmp.ends_with("faultinj")
                || tmp.ends_with("globals")
                || tmp.ends_with("mapping")
                || tmp.ends_with("retc")
            {
                return None;
            } else {
                return Some(tmp);
            }
        })
        .collect::<Vec<_>>();

    if is_fi >= 0 {
        //for i in 0..6 {
        let files = get_files(&names, is_fi, is_fi, max_comp, offset);
        faulty_runs.extend_from_slice(&*files);
    //}
    } else {
        faulty_runs.extend_from_slice(&*get_files(&names, 0, is_fi, max_comp, offset));
    }

    if let Ok((mut golden_trace, mut faulty_traces, result)) =
        do_comparison(golden_run, faulty_runs, nondet, benchmark)
    {
        let mut llfi_dir = String::from(&*base_dir);
        llfi_dir += "/";
        llfi_dir += if is_fi >= 0 { "fi" } else { "gr" };
        let llfi_symb_dir = format!("{}/llfi-symb-traces", llfi_dir);
        llfi_dir += "/llfi-traces";

        let mut tmp_span = 250;
        if let Some(s) = max_span {
            tmp_span = s;
        }

        let tmp_trace = golden_trace.get_llfi_trace(tmp_span);
        golden_trace.drain_content();
        let mut golden_llfi = String::from(&*llfi_dir);
        golden_llfi += "/";
        golden_llfi += benchmark;
        golden_llfi += "_trace.goldenrun-llfi";
        if let Ok(c) = &tmp_trace {
            if let Err(_) = write_to_file(&*c.0, &*golden_llfi) {
                warn!("Couldn't write golden trace...");
            }
        }

        let mut golden_llfi_symb = format!("{}/{}_trace.goldenrun-llfi", llfi_symb_dir, benchmark);
        if let Ok(c) = &tmp_trace {
            if let Err(_) = write_to_file(&*c.1, &*golden_llfi_symb) {
                warn!("Couldn't write golden trace...");
            }
        }

        if is_llfi_comp {
            let mut pb = Arc::new(Mutex::new(ProgressBar::new(faulty_traces.len() as u64)));
            faulty_traces.par_iter_mut().for_each(|_f| {
                warn!("writing llfi...");
                let tmp_trace = _f.get_llfi_trace(tmp_span);
                let mut faulty_llfi = String::from(&*llfi_dir);
                faulty_llfi += "/";
                faulty_llfi += &*_f.id;
                faulty_llfi += "-llfi";
                if let Ok(c) = &tmp_trace {
                    if let Err(_) = write_to_file(&*c.0, &*faulty_llfi) {
                        warn!("Couldn't write a faulty run: {}", faulty_llfi);
                    }
                }
                let faulty_llfi_symb = format!("{}/{}-llfi", llfi_symb_dir, _f.id);
                if let Ok(c) = &tmp_trace {
                    if let Err(_) = write_to_file(&*c.1, &*faulty_llfi_symb) {
                        warn!("Couldn't write a faulty run: {}", faulty_llfi_symb);
                    }
                }
                _f.drain_content();
                pb.lock().unwrap().inc();
            });
            let mut pb = Arc::new(Mutex::new(ProgressBar::new(faulty_traces.len() as u64 * 2)));
            let llfi_results = faulty_traces
                .par_iter()
                .filter_map(|_f| {
                    warn!("diffing...");
                    let mut faulty_llfi = String::from(&*llfi_dir);
                    faulty_llfi += "/";
                    faulty_llfi += &*_f.id;
                    faulty_llfi += "-llfi";
                    let faulty_llfi_symb = format!("{}/{}-llfi", llfi_symb_dir, _f.id);

                    if !Path::new(&*faulty_llfi_symb).exists() {
                        return None;
                    }

                    if !Path::new(&*faulty_llfi).exists() {
                        return None;
                    }

                    let start = Instant::now();
                    let status = exec_diff_script(&*golden_llfi, &*faulty_llfi);
                    let end = start.elapsed();
                    if let None = status {
                        warn!("llfi diff script crashed...");
                        return None;
                    }
                    let inf = _f.get_id_info();
                    let mut llfi_result = Vec::new();
                    llfi_result.extend_from_slice(&[
                        inf.0,
                        inf.1,
                        inf.2,
                        get_result_code(status),
                        format!("{}", _f.output_hash != golden_trace.output_hash),
                        _f.ret_code.clone(),
                        (_f.is_injected()).to_string(),
                        format_time(&_f.symb_time),
                        format_time(&end),
                    ]);
                    pb.lock().unwrap().inc();

                    let start = Instant::now();
                    let status = exec_diff_script(&*golden_llfi_symb, &*faulty_llfi_symb);
                    let end = start.elapsed();
                    if let None = status {
                        warn!("llfi diff script crashed...");
                    }
                    let inf = _f.get_id_info();

                    let mut llfi_symb_result = Vec::new();
                    llfi_symb_result.extend_from_slice(&[
                        inf.0,
                        inf.1,
                        inf.2,
                        get_result_code(status),
                        format!("{}", _f.output_hash != golden_trace.output_hash),
                        _f.ret_code.clone(),
                        (_f.is_injected()).to_string(),
                        format_time(&_f.symb_time),
                        format_time(&end),
                    ]);
                    pb.lock().unwrap().inc();
                    return Some((llfi_result, llfi_symb_result));
                })
                .collect::<Vec<_>>();
            let mut ll_result_f = format!(
                "{}/{}/llfi-results/{}-llfi-results",
                base_dir,
                if is_fi >= 0 { "fi" } else { "gr" },
                benchmark
            );
            let mut ll_symb_result_f = format!(
                "{}/{}/llfi-symb-results/{}-llfi-symb-results",
                base_dir,
                if is_fi >= 0 { "fi" } else { "gr" },
                benchmark
            );
            let mut tmp1 = Vec::new();
            let mut tmp2 = Vec::new();
            for (r1, r2) in llfi_results.iter() {
                tmp1.push(r1.clone());
                tmp2.push(r2.clone());
            }
            if let Err(e) = write_results(&*ll_result_f, tmp1, is_overwrite, is_append) {
                warn!("Couldn't write the llfi results {}: {}", e, ll_result_f);
            }
            if let Err(e) = write_results(&*ll_symb_result_f, tmp2, is_overwrite, is_append) {
                warn!(
                    "Couldn't write the llfi resylts {}: {}",
                    e, ll_symb_result_f
                );
            }
        }

        let mut result_f = String::from(&*base_dir);
        result_f += "/";
        result_f += if is_fi >= 0 { "fi" } else { "gr" };
        result_f += "/results/";
        result_f += benchmark;
        result_f += "-results";

        if let Err(e) = write_results(&*result_f, result, is_overwrite, is_append) {
            warn!("Couldn't write the results to {}: {}", e, result_f);
        }
    } else {
        warn!("Comparison was unsuccessful...");
    }
}
fn main() {
    badlog::init(Some("Warn"));

    info!("Starting the refactored version...");
    let mut benchmark = String::new();
    //let mut num: Option<u32> = Some(1000);
    //let mut f_num: Option<u32> = Some(6);
    let mut max_comp: Option<u32> = None;
    let mut max_span: Option<u32> = None;
    let mut is_fi = -1;
    let mut is_seq = false;
    let mut is_llfi_comp = false;
    let mut is_overwrite = false;
    let mut is_append = false;
    let mut nondet = false;
    let mut offset = 0;
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Trace postprocessor.");
        ap.refer(&mut benchmark)
            .add_argument("benchmark", Store, "benchmark to process");
        ap.refer(&mut max_comp).add_argument(
            "repetitions",
            StoreOption,
            "maximal number of repetitions",
        );
        ap.refer(&mut max_span)
            .add_argument("span", StoreOption, "maximal diff span");
        ap.refer(&mut is_fi)
            .add_option(&["-f", "--fi"], Store, "compare with fault injection");
        ap.refer(&mut is_seq)
            .add_option(&["-s", "--seq"], StoreTrue, "compare sequential traces");
        ap.refer(&mut is_llfi_comp)
            .add_option(&["-l", "--llfi"], StoreTrue, "compare with llfi");
        ap.refer(&mut is_overwrite).add_option(
            &["-o", "--ov"],
            StoreTrue,
            "overwrite the result files",
        );
        ap.refer(&mut is_append)
            .add_option(&["-a", "--app"], StoreTrue, "append to result files");
        ap.refer(&mut nondet).add_option(&["-c", "--check"], StoreTrue, "perform a the non-deterministic check");
        ap.refer(&mut offset).add_option(&["-i", "--interv"], Store, "select a subset of of trace files");
        ap.parse_args_or_exit();
    }

    start(
        &*benchmark,
        is_fi,
        is_seq,
        is_llfi_comp,
        is_overwrite,
        is_append,
        max_comp,
        max_span,
        nondet,
        offset
    );

    /*
    let mut file1 = String::new();
    let mut file2 = String::new();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Trace postprocessor.");
        ap.refer(&mut file1)
            .add_argument("file1", Store, "Trace file to process");
        ap.refer(&mut file2)
            .add_argument("file2", Store, "Trace file to process");
        ap.parse_args_or_exit();
    }

    let mut files = Vec::new();
    files.push(file1);
    files.push(file2);

    let traces = Arc::new(Mutex::new(Vec::new()));
    info!("Starting...");
    files.par_iter().for_each(|_f| {
        if let Ok(tr) = SymbolicTrace::from(_f) {
            traces.lock().unwrap().push(tr);
        } else {
            info!("Problem!!");
        }
    });
   
    traces.lock().unwrap().sort_by(|a, b| a.id.cmp(&b.id));
    let _tr2 = traces.lock().unwrap().pop().unwrap();
    let _tr1 = traces.lock().unwrap().pop().unwrap();
    info!("Trace 1 is {}", _tr1.id);
    info!("Trace 2 is {}", _tr2.id);
    info!("Trace loading successfully");
    info!("start comparison");
    //res = compare_mc(&_tr1, &_tr2);
    let res = compare_naive(&_tr1, &_tr2);
    //res = compare_sequential(&_tr1, &_tr2);

    
    /*
    if res {
        info!("There is no deviation...");
    } else {
        info!("There is a deviation...");
    }
    info!("Finished...");
    */
        */
}
