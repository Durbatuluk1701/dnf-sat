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
use rayon::iter::{
    IntoParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelBridge,
    ParallelIterator,
};
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

// // Invariant: All formulas in fvec vector are in DNF form
// fn formula_cross<'a>(fvec: &Vec<Formula<'a>>) -> &'a Formula<'a> {
//     let ret_vec: Vec<Vec<Formula>> = Vec::new();
//     for f in fvec {
//         match f {
//             FVar(_) => {
//                 for mut vec in ret_vec {
//                     vec.push(*f)
//                 }
//             }
//             FNeg(_) => {
//                 for mut vec in ret_vec {
//                     vec.push(*f)
//                 }
//             }
//             FDisj(fvec_rec) => {}
//             FConj(fvec_rec) => {}
//         }
//     }
//     let mut final_vec: Vec<Formula> = Vec::new();
//     for vec in ret_vec {
//         final_vec.push(FConj(&vec));
//     }
//     &FDisj(&final_vec)
// }

fn to_dnf(f: Formula) -> Formula {
    match f {
        FVar(_) => f,
        FDisj(fvec) => {
            let ret_vec = fvec.into_par_iter().map(|ele| to_dnf(ele)).collect();
            // let (f1, f2) = rayon::join(|| to_dnf(f1), || to_dnf(f2));
            flatten(FDisj(ret_vec))
            // Box::new(FDisj(f1, f2))
        }
        FNeg(v) => match *v {
            FVar(_) => *v,
            FNeg(frr) => to_dnf(*frr),
            FConj(fvec) => {
                let ret_vec = fvec
                    .into_par_iter()
                    .map(|ele| FNeg(Box::new(to_dnf(ele))))
                    .collect();
                // fvec.par_iter_mut().for_each(|mut ele| {
                //     ele = &mut FNeg(Box::new(to_dnf(*ele)));
                // });
                to_dnf(FDisj(ret_vec))
                // flatten(&mut ret_f);
                // ret_f
            }
            FDisj(fvec) => {
                let ret_vec = fvec
                    .into_par_iter()
                    .map(|ele| FNeg(Box::new(to_dnf(ele))))
                    .collect();
                // fvec.par_iter_mut()
                //     .for_each(|mut ele| ele = &mut FNeg(Box::new(to_dnf(*ele))));
                to_dnf(FConj(ret_vec))
                // let (f1, f2) = rayon::join(|| to_dnf(f1), || to_dnf(f2));
                // to_dnf(Box::new(FConj(Box::new(FNeg(f1)), Box::new(FNeg(f2)))))
            }
        },
        FConj(_) => {
            to_dnf(FNeg(Box::new(to_dnf(FNeg(Box::new(f))))))

            // fvec.par_iter_mut()
            //     .for_each(|mut ele| ele = &mut to_dnf(ele));
            // to_dnf(&FConj(fvec))
        } // match *to_dnf(f1) {
    }
}

// fn to_dnf(f: Formula) -> Formula {
//     match f {
//         FVar(_) => f,
//         FDisj(f1, f2) => FDisj(Box::new(to_dnf(*f1)), Box::new(to_dnf(*f2))),
//         FNeg(v) => match *v {
//             FVar(_) => FNeg(v),
//             FNeg(frr) => to_dnf(*frr),
//             FConj(f1, f2) => FDisj(Box::new(to_dnf(FNeg(f1))), Box::new(to_dnf(FNeg(f2)))),
//             FDisj(f1, f2) => to_dnf(FConj(Box::new(FNeg(f1)), Box::new(FNeg(f2)))),
//         },
//         FConj(f1, f2) => match to_dnf(*f1) {
//             FDisj(f11, f12) => match to_dnf(*f2) {
//                 FDisj(f21, f22) => {
//                     let f11_c = f11.clone();
//                     let f12_c = f12.clone();
//                     let f21_c = f21.clone();
//                     let f22_c = f22.clone();
//                     FDisj(
//                         Box::new(FDisj(
//                             Box::new(to_dnf(FConj(f11, f21))),
//                             Box::new(to_dnf(FConj(f11_c, f22))),
//                         )),
//                         Box::new(FDisj(
//                             Box::new(to_dnf(FConj(f12, f21_c))),
//                             Box::new(to_dnf(FConj(f12_c, f22_c))),
//                         )),
//                     )
//                 }
//                 v2 => {
//                     let v2_c = v2.clone();
//                     FDisj(
//                         Box::new(to_dnf(FConj(f11, Box::new(v2)))),
//                         Box::new(to_dnf(FConj(f12, Box::new(v2_c)))),
//                     )
//                 }
//             },
//             v1 => match to_dnf(*f2) {
//                 FDisj(f21, f22) => {
//                     let v1_c = v1.clone();
//                     FDisj(
//                         Box::new(to_dnf(FConj(Box::new(v1), f21))),
//                         Box::new(to_dnf(FConj(Box::new(v1_c), f22))),
//                     )
//                 }
//                 v2 => FConj(Box::new(v1), Box::new(v2)),
//             },
//         },
//     }
// }
// fn form_vec_to_formula<'a>(v: &'a [Box<Formula<'a>>]) -> Box<Formula<'a>> {
//     let v_len = v.len();
//     if v_len == 1 {
//         let v_0 = &v[0];
//         return v_0.clone();
//     }
//     let (vf, vb) = v.split_at(v_len / 2);
//     return Box::new(FDisj(form_vec_to_formula(vf), form_vec_to_formula(vb)));
// }

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
    FConj(ret_val)
}

// fn line_vec_to_formula(v: &[Result<String, std::io::Error>]) -> Box<Formula> {
//     let v_len = v.len();
//     if v_len == 1 {
//         // Process this line
//         let v_line = v[0].as_ref().expect("Needed a string value");
//         return proc_line(v_line);
//     }
//     let (v1, v2) = v.split_at(v_len / 2);
//     let (v1_r, v2_r) = rayon::join(|| line_vec_to_formula(&v1), || line_vec_to_formula(&v2));
//     Box::new(FConj(v1_r, v2_r))
// }

// (1 \/ 2) /\ (-1 \/ 2)
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
    // let _first_line = lines
    //     .next()
    //     .expect("Need at least a first line")
    //     .expect("Shouldnt be an error on line 1");

    // NOTE: There is some nuance to this, would we rather have extra threads or exactly as many threads as cores?
    let ret_form_vec: Vec<Formula> = lines.par_bridge().map(|line| proc_line(line)).collect();
    // lines.take(num_lines.try_into().unwrap()).collect();
    println!("Welcome to Converter");
    println!(
        "{:?}",
        to_dnf(FDisj(ret_form_vec)) // to_dnf(FConj(
                                    //     Box::new(FDisj(Box::new(FVar(1)), Box::new(FVar(2)))),
                                    //     Box::new(FDisj(Box::new(FNeg(Box::new(FVar(1)))), Box::new(FVar(2))))
                                    // ))
    )
}
