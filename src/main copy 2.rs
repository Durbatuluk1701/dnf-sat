use std::{env, vec};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::exit;
use std::sync::{Arc, Mutex};
// use std::thread;

#[derive(Hash, Eq, PartialEq, Debug)]
enum Formula {
  FVar(u32),
  FNeg(Box<Formula>),
  FDisj(Box<Formula>, Box<Formula>),
  FConj(Box<Formula>, Box<Formula>),
}

// Possible optimization someday, make a search tree-esque structure
type Valuation = Vec<(u32, bool)>;

use Formula::{FVar, FNeg, FConj, FDisj};

// Checks if a valuation has a key already assigned
fn val_key_in (l : &Valuation, k : &u32) -> bool {
  for (key, _val) in l {
    if key == k {
      return true
    }
  }
  false
}

fn val_in (l : &Valuation, (k,v) : &(u32, bool)) -> bool {
  for (key, val) in l {
    if key == k {
      return val == v
    }
  }
  false
}

/**
 * Spec: mutates l in places, returns false if insert is bad
 */
fn val_insertion (l : &mut Valuation, v : (u32, bool)) -> bool {
  if val_key_in(l, &v.0) {
    // The key is already in
    // Returns true if kv pair is in, or false (bad insert) if they disagree
    return val_in(l, &v)
  }
  l.push(v);
  true
}

// If all the keys of l are in r and vice versa
fn valuation_key_eq (l : &Valuation, r : &Valuation) -> bool {
  for (key, _val) in l {
    if !val_key_in(r, key) {
      return false
    }
  }
  for (key, _val) in r {
    if !val_key_in(l, key) {
      return false
    }
  }
  true
}

// If all the keys and values of l are in r and vice versa
fn valuation_eq (l : &Valuation, r : &Valuation) -> bool {
  for kv in l {
    if !val_in(r, kv) {
      return false
    }
  }
  for kv in r {
    if !val_in(l, kv) {
      return false
    }
  }
  true
}

/**
 * Spec: Finds the union, unless they are inconsistent
 */
fn val_union (l : &Valuation, r : &Valuation) -> Option<Valuation> {
  let mut ret_val : Valuation = Vec::new();
  for ele in l {
    ret_val.push(*ele);
  }
  for ele in r {
    if !val_insertion(&mut ret_val, *ele) {
      // Insertion failed, return None
      return None
    }
  }
  Some(ret_val)
}

// Invariant: l and r are valid Vec's of valuations (built properly)
fn val_set_union (l : Vec<Valuation>, r : Vec<Valuation>) -> Vec<Valuation> {
  let mut ret_vec: Vec<Valuation> = Vec::<Valuation>::new();
  // Since l is built properly, we can add all of l to ret_vec
  for ele in l {
    ret_vec.push(ele)
  }
  for ele1 in r {
    let mut ele1_in_ret_vec = false;
    for ele2 in &ret_vec {
      // If at anypoint ele1 = ele2, this changes to true
      ele1_in_ret_vec |= valuation_eq(&ele1, ele2)
    }
    if !ele1_in_ret_vec {
      ret_vec.push(ele1)
    }
  }
  ret_vec
}

/**
 * Spec: { val1 \cup val2 | \forall val1 \in l, val2 \in r }
 * Invariant: l and r are "valid"
 */
fn val_set_cross (l : Vec<Valuation>, r : Vec<Valuation>) -> Vec<Valuation> {
  let mut ret_vec: Vec<Valuation> = Vec::<Valuation>::new();
  for ele1 in &l {
    for ele2 in &r {
      match val_union(ele1, ele2) {
        None => continue,
        Some(v) => {
          ret_vec.push(v)
        }
      }
    }
  }
  ret_vec
}

fn dnf_sat (f : Formula, neg_mode : bool) -> Vec<Valuation> {
  match f {
    FVar(v) => {
      // Return v in a set by itself,
      vec![vec![(v, !neg_mode)]]
    },
    FNeg(f) => {
      dnf_sat(*f, !neg_mode)
    },
    FDisj(f1, f2) => {
      let lhandle= std::thread::spawn(move || dnf_sat(*f1, neg_mode));
      let rhandle= std::thread::spawn(move || dnf_sat(*f2, neg_mode));
      let l = lhandle.join().unwrap();
      let r = rhandle.join().unwrap();
      if neg_mode {
        // Set Cross
        return val_set_cross(l, r)

      }
      val_set_union(l, r)
    }
    FConj(f1, f2) => {
      let lhandle= std::thread::spawn(move || dnf_sat(*f1, neg_mode));
      let rhandle= std::thread::spawn(move || dnf_sat(*f2, neg_mode));
      let l = lhandle.join().unwrap();
      let r = rhandle.join().unwrap();
      if neg_mode {
        // Set Union
        return val_set_union(l, r)
      }
      val_set_cross(l, r)
    }
  }
}

/**
 * Each "line" is a set of X1 \/ ... \/ XN (with no conjs)
 * So, we can just make one big OR union
 */
fn proc_line(line : &String) -> Vec<Valuation> {
  print!("{}", line);
  let vars = line.split(" ");
  let mut ret_val : Vec<Valuation> = Vec::new();
  for ele in vars {
    let val: i32 = ele.parse().expect("CRITICAL ERROR PARSING LINE");
    if val >= 0 {
      let val_good = val as u32;
      // It is safe to add like this because they are all INDEPENDENT
      // AND its is a Vec of Vals
      ret_val.insert(0, vec![(val_good, true)]);
      // if !val_insertion(&mut cur_val, (val_good, true)) {
      //   // UNSAT already
      //   return None
      // }
    } else {
      let val_good = -val as u32;
      ret_val.insert(0, vec![(val_good, false)]);
      // if !val_insertion(&mut cur_val, (val_good, false)) {
      //   // UNSAT already
      //   return None
      // }
    }
  }
  ret_val
}

fn main() {
  let args: Vec<String> = env::args().collect();

  dbg!(&args);

  if args.len() != 2 && args.len() != 4 {
    eprintln!("Usage: {} <input_file> [-c|--cores <number>]", args[0]);
    exit(-1);
  }

  let file_name = &args[1];
  
  let num_cores : i32 = if args.len() == 4 { args[3].parse().expect("Expected number of cores to be an i32") } else { 1 };

  let file = File::open(file_name).expect("Failed to open file!");

  let reader = BufReader::new(file);

  let mut lines = reader.lines();
  // READ THE FIRST INFO LINE
  let first_line = lines.next().expect("Need at least a first line").expect("Shouldnt be an error on line 1");
  let first_line_info : Vec<&str> = first_line.split(" ").take(4).collect();
  let num_lines : i32 = first_line_info[3].parse().expect("Expected number of lines entry to be i32");

  let overall_queue = Arc::new(Mutex::new(Vec::new()));

  let general_partion_size : i32 = num_lines / num_cores;
  let gp_usize : usize = general_partion_size.try_into().unwrap();
  let vec_lines : Vec<_> = lines.take(num_lines.try_into().unwrap()).collect();
  let chunks = vec_lines.chunks(gp_usize);

  crossbeam::thread::scope(|scope| {
    let mut thread_handles = vec![];

    for chunk in chunks {
      print!("STARTING CHUNK");
      thread_handles.push(scope.spawn(move |_| {
        let mut queue : Vec<Vec<Valuation>> = Vec::new();
        for line_res in chunk { 
          match line_res {
            Err(_) => panic!("What happened"),
            Ok(line) => {
              queue.push(proc_line(&line));
              while queue.len() >= 2 {
                // Want to merge together current valuations
                let first = queue.pop().expect("IMPOSSIBLE1");
                let second = queue.pop().expect("IMPOSSIBLE2");
                queue.push(val_set_cross(first, second))
              }
            }
          }
        };
        queue
      }));
    }
    for handle in thread_handles {
      let binding = Arc::clone(&overall_queue);
      let mut cloned_queue = binding.lock().unwrap();
      cloned_queue.append(&mut handle.join().unwrap());
    }
  }).unwrap();

  dbg!(overall_queue);

  // if overall_queue.lock().unwrap().len() > 0 {
  //   print!("SAT");
  //   return;
  // }
  // print!("UNSAT");

  // for ele in thread_list.lock() {
  //   for ele2 in ele {
  //     print!("{}: {}, ", ele2.0, ele2.1);
  //   }
  //   print!("\n")
  // }
}
