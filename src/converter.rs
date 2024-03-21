pub mod formula;
use formula::formula::Formula;
use Formula::{FConj, FDisj, FNeg, FVar};

fn to_dnf(f: Formula) -> Formula {
    match f {
        FVar(_) => f,
        FDisj(f1, f2) => {
            let (f1, f2) = rayon::join(|| to_dnf(*f1), || to_dnf(*f2));
            FDisj(Box::new(f1), Box::new(f2))
        }
        FNeg(v) => match *v {
            FVar(_) => FNeg(v),
            FNeg(frr) => to_dnf(*frr),
            FConj(f1, f2) => {
                let (f1, f2) = rayon::join(|| to_dnf(*f1), || to_dnf(*f2));
                FDisj(
                    Box::new(to_dnf(FNeg(Box::new(f1)))),
                    Box::new(to_dnf(FNeg(Box::new(f2)))),
                )
            }
            FDisj(f1, f2) => {
                let (f1, f2) = rayon::join(|| to_dnf(*f1), || to_dnf(*f2));
                to_dnf(FConj(
                    Box::new(FNeg(Box::new(f1))),
                    Box::new(FNeg(Box::new(f2))),
                ))
            }
        },
        FConj(f1, f2) => match to_dnf(*f1) {
            FDisj(f11, f12) => match to_dnf(*f2) {
                FDisj(f21, f22) => {
                    let f11_c = f11.clone();
                    let f12_c = f12.clone();
                    let f21_c = f21.clone();
                    let f22_c = f22.clone();
                    let (f11r, f12r) =
                        rayon::join(|| to_dnf(FConj(f11, f21)), || to_dnf(FConj(f11_c, f22)));
                    let (f21r, f22r) =
                        rayon::join(|| to_dnf(FConj(f12, f21_c)), || to_dnf(FConj(f12_c, f22_c)));
                    FDisj(
                        Box::new(FDisj(Box::new(f11r), Box::new(f12r))),
                        Box::new(FDisj(Box::new(f21r), Box::new(f22r))),
                    )
                }
                v2 => {
                    let v2_c = v2.clone();
                    let (f11r, f12r) = rayon::join(
                        || to_dnf(FConj(f11, Box::new(v2))),
                        || to_dnf(FConj(f12, Box::new(v2_c))),
                    );
                    FDisj(Box::new(f11r), Box::new(f12r))
                }
            },
            v1 => match to_dnf(*f2) {
                FDisj(f21, f22) => {
                    let v1_c = v1.clone();
                    let (f11r, f12r) = rayon::join(
                        || to_dnf(FConj(Box::new(v1), f21)),
                        || to_dnf(FConj(Box::new(v1_c), f22)),
                    );
                    FDisj(Box::new(f11r), Box::new(f12r))
                }
                v2 => FConj(Box::new(v1), Box::new(v2)),
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

// (1 \/ 2) /\ (-1 \/ 2)
fn main() {
    println!("Welcome to Converter");
    println!(
        "{:?}",
        to_dnf(FConj(
            Box::new(FDisj(Box::new(FVar(1)), Box::new(FVar(2)))),
            Box::new(FDisj(Box::new(FNeg(Box::new(FVar(1)))), Box::new(FVar(2))))
        ))
    )
}
