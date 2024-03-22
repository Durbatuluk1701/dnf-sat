use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
enum Formula {
    FVar(u32),
    FNeg(Box<Formula>),
    FDisj(Vec<Formula>),
    FConj(Vec<Formula>),
}
// use rayon::iter::{
//     IntoParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelBridge,
//     ParallelIterator,
// };
use Formula::{FConj, FDisj, FNeg, FVar};

// Invariant: Every sub-formula is already flat and DNF
fn flatten(f: Formula) -> Formula {
    match f {
        FVar(_) => f,
        FNeg(fr) => match *fr {
            FVar(x) => FNeg(Box::new(FVar(x))),
            FNeg(f_bot) => *f_bot,
            FConj(_) => panic!("Cannot have Conj below Disj"),
            FDisj(_) => panic!("Cannot have Disj below Conj"),
        },
        FDisj(fvec) => {
            let mut ret_vec = Vec::new();
            for fv in fvec {
                match fv {
                    FVar(_) => ret_vec.push(fv),
                    FNeg(_) => ret_vec.push(fv),
                    FDisj(mut fvec_nested) => ret_vec.append(&mut fvec_nested),
                    FConj(mut fvec_nested) => ret_vec.append(&mut fvec_nested),
                }
            }
            FDisj(ret_vec)
        }
        FConj(fvec) => {
            let mut ret_vec = Vec::new();
            for fv in fvec {
                match fv {
                    FVar(_) => ret_vec.push(fv),
                    FNeg(_) => ret_vec.push(fv),
                    FDisj(_) => panic!("Should not have Disj below Conj"),
                    FConj(mut fvec_nested) => ret_vec.append(&mut fvec_nested),
                }
            }
            FConj(ret_vec)
        }
    }
}

// // Invariant: Every sub-formula is already flat and DNF
// fn flatten(f: &mut Formula) {
//     match f {
//         FVar(_) => (),
//         FNeg(f_new) => match **f_new {
//             FVar(_) => (),
//             FNeg(f_bot) => *f = *f_bot,
//             FConj(_) => panic!("Cannot have Conj below Disj"),
//             FDisj(_) => panic!("Cannot have Disj below Conj"),
//         },
//         FDisj(mut fvec) | FConj(mut fvec) => {
//             let mut new_vec = Vec::new();
//             for fv in fvec.into_iter() {
//                 match fv {
//                     FDisj(mut fvec_nested) | FConj(mut fvec_nested) => {
//                         new_vec.append(&mut fvec_nested);
//                     }
//                     _ => {}
//                 }
//             }
//             fvec = new_vec;
//             // *fvec = &new_vec;
//         }
//     }
// }

// Invariant: All formulas in fvec vector are in DNF form
fn formula_cross(fvec: Vec<Formula>) -> Formula {
    let mut ret_vec: Vec<Vec<Formula>> = vec![vec![]];
    for f in fvec {
        match f {
            FVar(_) | FNeg(_) => {
                for vec in &mut ret_vec {
                    vec.push(f.clone())
                }
            }
            FDisj(fvec_rec) => {
                for form in fvec_rec {
                    for vec in &mut ret_vec {
                        vec.push(form.clone());
                    }
                }
            }
            FConj(mut fvec_rec) => {
                for vec in &mut ret_vec {
                    vec.append(&mut fvec_rec);
                }
            }
        }
    }
    let final_vec: Vec<Formula> = ret_vec.into_iter().map(|vec| FConj(vec)).collect();
    FDisj(final_vec)
}

fn to_dnf(f: Formula) -> Formula {
    match f {
        FVar(_) => f,
        FDisj(fvec) => {
            let mut ret_vec = Vec::new();
            for ele in fvec.into_iter() {
                let rec_call = to_dnf(ele);
                ret_vec.push(rec_call)
            }
            // let ret_vec = fvec.into_par_iter().map(|ele| to_dnf(ele)).collect();
            flatten(FDisj(ret_vec))
        }
        FNeg(v) => match *v {
            FVar(x) => FNeg(Box::new(FVar(x))),
            FNeg(frr) => to_dnf(*frr),
            FConj(fvec) => {
                let mut ret_vec = Vec::new();
                for ele in fvec.into_iter() {
                    let rec_call = to_dnf(FNeg(Box::new(ele)));
                    ret_vec.push(rec_call)
                }
                // let ret_vec = fvec
                //     .into_par_iter()
                //     .map(|ele| FNeg(Box::new(to_dnf(ele))))
                //     .collect();
                to_dnf(flatten(FDisj(ret_vec)))
            }
            FDisj(fvec) => {
                let mut ret_vec = Vec::new();
                for ele in fvec.into_iter() {
                    let rec_call = to_dnf(FNeg(Box::new(ele)));
                    ret_vec.push(rec_call)
                }
                // let ret_vec = fvec
                //     .into_par_iter()
                //     .map(|ele| FNeg(Box::new(to_dnf(ele))))
                //     .collect();
                to_dnf(flatten(FConj(ret_vec)))
            }
        },
        FConj(fvec) => {
            let mut ret_vec = Vec::new();
            for ele in fvec.into_iter() {
                let rec_call = to_dnf(ele);
                ret_vec.push(rec_call);
            }
            formula_cross(ret_vec)
        }
    }
}

fn proc_line(line: Result<String, std::io::Error>) -> Formula {
    // println!("{}", line);
    let good_line = line.expect("String needed").trim().replace("  ", " ");
    let vars = good_line.split(" ");
    let mut ret_val: Vec<Formula> = Vec::new();
    for ele in vars {
        let val: i32 = ele
            .parse()
            .expect(&format!("CRITICAL ERROR PARSING LINE: '{good_line}'"));
        if val >= 0 {
            let val_good = val as u32;
            ret_val.push(FVar(val_good));
        } else {
            let val_good = -val as u32;
            ret_val.push(FNeg(Box::new(FVar(val_good))));
        }
    }
    FDisj(ret_val)
}

fn disj_below_neg(f: Formula, in_neg: bool) -> bool {
    match f {
        FVar(_) => false,
        FNeg(frr) => disj_below_neg(*frr, true),
        FDisj(rec_vec) => {
            if in_neg {
                return true;
            }
            for val in rec_vec {
                if disj_below_neg(val, in_neg) {
                    return true;
                }
            }
            false
        }
        FConj(rec_vec) => {
            for val in rec_vec {
                if disj_below_neg(val, in_neg) {
                    return true;
                }
            }
            false
        }
    }
}

fn disj_below_conj(f: Formula, in_conj: bool) -> bool {
    match f {
        FVar(_) => false,
        FNeg(frr) => disj_below_neg(*frr, in_conj),
        FDisj(rec_vec) => {
            if in_conj {
                return true;
            }
            for val in rec_vec {
                if disj_below_neg(val, in_conj) {
                    return true;
                }
            }
            false
        }
        FConj(rec_vec) => {
            for val in rec_vec {
                if disj_below_neg(val, true) {
                    return true;
                }
            }
            false
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    dbg!(&args);

    if args.len() != 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        std::process::exit(-1);
    }

    let file_name = &args[1];

    let file = File::open(file_name).expect("Failed to open file!");

    let reader = BufReader::new(file);

    let mut lines = reader.lines();
    println!("READ LINES INTO BUFREADER");
    // READ THE FIRST INFO LINE
    lines.next();

    let ret_form_vec: Vec<Formula> = lines.into_iter().map(|line| proc_line(line)).collect();
    // par_bridge().map(|line| proc_line(line)).collect();
    println!("Welcome to Converter");
    let formula = to_dnf(FConj(ret_form_vec));
    println!(
        "DONE: DISJ_BELOW_CONJ = {}; DISJ_BELOW_NEG = {}",
        disj_below_neg(formula.clone(), false),
        disj_below_conj(formula, false)
    );
}
