use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::thread;

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
      let lhandle= thread::spawn(move || dnf_sat(*f1, neg_mode));
      let rhandle= thread::spawn(move || dnf_sat(*f2, neg_mode));
      let l = lhandle.join().unwrap();
      let r = rhandle.join().unwrap();
      if neg_mode {
        // Set Cross
        return val_set_cross(l, r)

      }
      val_set_union(l, r)
    }
    FConj(f1, f2) => {
      let lhandle= thread::spawn(move || dnf_sat(*f1, neg_mode));
      let rhandle= thread::spawn(move || dnf_sat(*f2, neg_mode));
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

fn proc_line(line : String) -> Option<Valuation> {
  let vars = line.split(" ");
  let mut ret_val: Valuation = Vec::new();
  for ele in vars {
    let val: i32 = ele.parse().expect("CRITICAL ERROR PARSING LINE");
    if val >= 0 {
      let val_good = val as u32;
      if !val_insertion(&mut ret_val, (val_good, true)) {
        // UNSAT already
        return None
      }
    } else {
      let val_good = -val as u32;
      if !val_insertion(&mut ret_val, (val_good, false)) {
        // UNSAT already
        return None
      }
    }
  }
  Some(ret_val)
}

fn main() {
  let args: Vec<String> = env::args().collect();

  dbg!(&args);

  if args.len() != 2 {
    eprintln!("Usage: {} <input_file>", args[0]);
    exit(-1);
  }

  let file_name = &args[1];

  let file = File::open(file_name).expect("Failed to open file!");

  let reader = BufReader::new(file);

  let thread_list: Arc<Mutex<Vec<Vec<Valuation>>>> = Arc::new(Mutex::new(Vec::new()));
  let mut handles = vec![];

  let mut lines = reader.lines();
  // SKIP THE FIRST info line
  lines.next();

  for line in lines {
    let thread_list_clone = Arc::clone(&thread_list);
    let line = line.expect("Failed to Read Line!");

    let thread_handle = thread::spawn(move || {
      match proc_line(line) {
        None => exit(1),
        Some(val) => {
          let mut queue = thread_list_clone.lock().unwrap();
          queue.push(vec![val]);
          while queue.len() >= 2 {
            // Want to merge together current valuations
            let first = queue.pop().expect("IMPOSSIBLE1");
            let second = queue.pop().expect("IMPOSSIBLE2");
            queue.push(val_set_cross(first, second))
          }
        }
      }
    });

    handles.push(thread_handle);
  }

  for handle in handles {
    handle.join().unwrap();
  } 

  if thread_list.lock().unwrap().len() > 0 {
    print!("SAT");
    return;
  }
  print!("UNSAT");

  // for ele in thread_list.lock() {
  //   for ele2 in ele {
  //     print!("{}: {}, ", ele2.0, ele2.1);
  //   }
  //   print!("\n")
  // }
}
