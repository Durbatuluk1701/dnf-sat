pub mod formula;
use core::fmt;
use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
};

use formula::formula::Formula;
use Formula::{FConj, FDisj, FNeg, FVar};

fn to_dnf(f: Box<Formula>) -> Box<Formula> {
    match *f {
        FVar(_) => f,
        FDisj(f1, f2) => {
            let (f1, f2) = rayon::join(|| to_dnf(f1), || to_dnf(f2));
            Box::new(FDisj(f1, f2))
        }
        FNeg(v) => match *v {
            FVar(_) => Box::new(FNeg(v)),
            FNeg(frr) => to_dnf(frr),
            FConj(f1, f2) => {
                let (f1, f2) = rayon::join(|| to_dnf(f1), || to_dnf(f2));
                Box::new(FDisj(
                    to_dnf(Box::new(FNeg(f1))),
                    to_dnf(Box::new(FNeg(f2))),
                ))
            }
            FDisj(f1, f2) => {
                let (f1, f2) = rayon::join(|| to_dnf(f1), || to_dnf(f2));
                to_dnf(Box::new(FConj(Box::new(FNeg(f1)), Box::new(FNeg(f2)))))
            }
        },
        FConj(f1, f2) => match *to_dnf(f1) {
            FDisj(f11, f12) => match *to_dnf(f2) {
                FDisj(f21, f22) => {
                    let f11_c = f11.clone();
                    let f12_c = f12.clone();
                    let f21_c = f21.clone();
                    let f22_c = f22.clone();
                    let (f11r, f12r) = rayon::join(
                        || to_dnf(Box::new(FConj(f11, f21))),
                        || to_dnf(Box::new(FConj(f11_c, f22))),
                    );
                    let (f21r, f22r) = rayon::join(
                        || to_dnf(Box::new(FConj(f12, f21_c))),
                        || to_dnf(Box::new(FConj(f12_c, f22_c))),
                    );
                    Box::new(FDisj(
                        Box::new(FDisj(f11r, f12r)),
                        Box::new(FDisj(f21r, f22r)),
                    ))
                }
                v2 => {
                    let v2_box = Box::new(v2);
                    let v2_box_c = v2_box.clone();
                    let (f11r, f12r) = rayon::join(
                        || to_dnf(Box::new(FConj(f11, v2_box))),
                        || to_dnf(Box::new(FConj(f12, v2_box_c))),
                    );
                    Box::new(FDisj(f11r, f12r))
                }
            },
            v1 => match *to_dnf(f2) {
                FDisj(f21, f22) => {
                    let v1_box = Box::new(v1);
                    let v1_box_c = v1_box.clone();
                    let (f11r, f12r) = rayon::join(
                        || to_dnf(Box::new(FConj(v1_box, f21))),
                        || to_dnf(Box::new(FConj(v1_box_c, f22))),
                    );
                    Box::new(FDisj(f11r, f12r))
                }
                v2 => Box::new(FConj(Box::new(v1), Box::new(v2))),
            },
        }, // FConj(f1, f2) => {
           //     let (f1, f2) = rayon::join(|| to_dnf(*f1), || to_dnf(*f2));
           //     match (f1, f2) {
           //         (FDisj(f11, f12), FDisj(f21, f22)) => {
           //             let f11_c = f11.clone();
           //             let f12_c = f12.clone();
           //             let f21_c = f21.clone();
           //             let f22_c = f22.clone();
           //             let (f11r, f12r, f21r, f22r) = rayon::join()
           //             FDisj(
           //                 Box::new(FDisj(
           //                     Box::new(to_dnf(FConj(f11, f21))),
           //                     Box::new(to_dnf(FConj(f11_c, f22))),
           //                 )),
           //                 Box::new(FDisj(
           //                     Box::new(to_dnf(FConj(f12, f21_c))),
           //                     Box::new(to_dnf(FConj(f12_c, f22_c))),
           //                 )),
           //             )
           //         }
           //         (v1, v2) => {
           //             let v2_c = v2.clone();
           //             FDisj(
           //                 Box::new(to_dnf(FConj(f1, Box::new(v2)))),
           //                 Box::new(to_dnf(FConj(f2, Box::new(v2_c)))),
           //             )
           //         }
           //     }
           // }
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
fn form_vec_to_formula(v: &[Box<Formula>]) -> Box<Formula> {
    let v_len = v.len();
    if v_len == 1 {
        let v_0 = &v[0];
        return v_0.clone();
    }
    let (vf, vb) = v.split_at(v_len / 2);
    return Box::new(FDisj(form_vec_to_formula(vf), form_vec_to_formula(vb)));
}

fn proc_line(line: &String) -> Box<Formula> {
    // println!("{}", line);
    let good_line = line.trim().replace("  ", " ");
    let vars = good_line.split(" ");
    let mut ret_val: Vec<Box<Formula>> = Vec::new();
    for ele in vars {
        let val: i32 = ele
            .parse()
            .expect(&format!("CRITICAL ERROR PARSING LINE: '{good_line}'"));
        if val >= 0 {
            let val_good = val as u32;
            ret_val.insert(0, Box::new(FVar(val_good)));
        } else {
            let val_good = -val as u32;
            ret_val.insert(0, Box::new(FNeg(Box::new(FVar(val_good)))));
        }
    }
    form_vec_to_formula(&ret_val)
}

fn line_vec_to_formula(v: &[Result<String, std::io::Error>]) -> Box<Formula> {
    let v_len = v.len();
    if v_len == 1 {
        // Process this line
        let v_line = v[0].as_ref().expect("Needed a string value");
        return proc_line(v_line);
    }
    let (v1, v2) = v.split_at(v_len / 2);
    let (v1_r, v2_r) = rayon::join(|| line_vec_to_formula(&v1), || line_vec_to_formula(&v2));
    Box::new(FConj(v1_r, v2_r))
}

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
    let vec_lines: Vec<Result<String, std::io::Error>> = lines.collect();
    let formula = line_vec_to_formula(&vec_lines);
    // lines.take(num_lines.try_into().unwrap()).collect();
    println!("Welcome to Converter");
    println!(
        "{:?}",
        to_dnf(formula) // to_dnf(FConj(
                        //     Box::new(FDisj(Box::new(FVar(1)), Box::new(FVar(2)))),
                        //     Box::new(FDisj(Box::new(FNeg(Box::new(FVar(1)))), Box::new(FVar(2))))
                        // ))
    )
}
