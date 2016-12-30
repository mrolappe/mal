use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use std::fs::File;
use std::io::Read;
use std::convert::From;
use std::string::String;

use itertools;

use reader;
use printer::pr_str;
use common::{MalData, CallableFun, FunContext};

fn mal_core_add(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    let res = number_arg(&args[0]) + number_arg(&args[1]);  // TODO fehlerbehandlung
    Ok(MalData::Number(res))
}

fn mal_core_sub(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    let res = number_arg(&args[0]) - number_arg(&args[1]);  // TODO fehlerbehandlung
    Ok(MalData::Number(res))
}

fn mal_core_mul(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    let res = number_arg(&args[0]) * number_arg(&args[1]);  // TODO fehlerbehandlung
    Ok(MalData::Number(res))
}

fn mal_core_div(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    let res = number_arg(&args[0]).checked_div(number_arg(&args[1])).unwrap();  // TODO fehlerbehandlung
    Ok(MalData::Number(res))
}

fn mal_core_list(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    Ok(MalData::List(Rc::from(args.to_vec())))
}

fn mal_core_list_p(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    if args.is_empty() {
        Err("argument required".to_owned())
    } else {
        match args[0] {
            MalData::List(_) | MalData::Nil =>
                Ok(MalData::True),

            _ =>
                Ok(MalData::False)
        }
    }
}

fn mal_core_empty_p(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    if args.is_empty() {
        Err("argument required".to_owned())
    } else {
        match args[0] {
            MalData::List(ref l) | MalData::Vector(ref l) =>
                Ok(if l.is_empty() { MalData::True } else { MalData::False }),

            _ =>
                Ok(MalData::False)
        }
    }
}

fn mal_core_count(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    if args.is_empty() {
        Err("argument required".to_owned())
    } else {
        match args[0] {
            MalData::Nil =>
                Ok(MalData::Number(0)),

            MalData::List(ref l) | MalData::Vector(ref l) =>
                Ok(MalData::Number(l.len() as i32)),

            _ =>
                Err("list argument required".to_owned())
        }
    }
}

fn mal_core_lt(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    // TODO fehlerbehandlung
    if args.is_empty() {
        Err("2 arguments required".to_owned())
    } else {
        if let ( &MalData::Number(n1), &MalData::Number(n2) ) = ( &args[0], &args[1] ) {
            Ok(if n1 < n2 { MalData::True } else { MalData::False })
        } else {
            Ok(MalData::False)
        }
    }
}

fn mal_core_le(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    // TODO fehlerbehandlung
    if args.is_empty() {
        Err("2 arguments required".to_owned())
    } else {
        if let ( &MalData::Number(n1), &MalData::Number(n2) ) = ( &args[0], &args[1] ) {
            Ok(if n1 <= n2 { MalData::True } else { MalData::False })
        } else {
            Ok(MalData::False)
        }
    }
}

fn mal_core_gt(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    // TODO fehlerbehandlung
    if args.is_empty() {
        Err("2 arguments required".to_owned())
    } else {
        if let ( &MalData::Number(n1), &MalData::Number(n2) ) = ( &args[0], &args[1] ) {
            Ok(if n1 > n2 { MalData::True } else { MalData::False })
        } else {
            Ok(MalData::False)
        }
    }
}

fn mal_core_ge(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    // TODO fehlerbehandlung
    if args.is_empty() {
        Err("2 arguments required".to_owned())
    } else {
        if let ( &MalData::Number(n1), &MalData::Number(n2) ) = ( &args[0], &args[1] ) {
            Ok(if n1 >= n2 { MalData::True } else { MalData::False })
        } else {
            Ok(MalData::False)
        }
    }
}

fn is_list_like(value: &MalData) -> bool {
    match *value {
        MalData::List(_) |
        MalData::Vector(_) =>
            true,

        _ =>
            false
    }
}

fn are_lists_equal(l1: &MalData, l2: &MalData) -> Result<bool, String> {
    if !is_list_like(l1) || !is_list_like(l2) {
        return Err(format!("l1 und l2 muessen listenartig sein (l1: {:?}, l2: {:?})", l1, l2));
    }

    match ( l1, l2 ) {
        ( &MalData::List(ref l1), &MalData::List(ref l2) )
            | ( &MalData::List(ref l1), &MalData::Vector(ref l2) )
            | ( &MalData::Vector(ref l1), &MalData::List(ref l2) )
            | ( &MalData::Vector(ref l1), &MalData::Vector(ref l2) )
            if l1.len() == l2.len() => {
                let res = l1.iter().zip(l2.iter()).all( |( e1, e2 )| {
                    if is_list_like(e1) && is_list_like(e2) {
                        are_lists_equal(e1, e2).unwrap()
                    } else {
                        e1 == e2   
                    }
                });

                Ok(res)
            }

        _ =>
            Ok(false)
    }
}

fn mal_core_equals(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    match ( args[0].clone(), args[1].clone() ) {
        ( MalData::True, MalData::True ) =>
            Ok(MalData::True),

        ( MalData::True, _ ) =>
            Ok(MalData::False),

        ( MalData::Nil, MalData::Nil ) =>
            Ok(MalData::True),

        ( MalData::Nil, _ ) =>
            Ok(MalData::False),

        ( MalData::Keyword(kw1), MalData::Keyword(kw2)) =>
            Ok(if kw1 == kw2 { MalData::True } else { MalData::False }),

        ( MalData::Keyword(_), _) =>
            Ok(MalData::False),

        ( MalData::Number(n1), MalData::Number(n2) ) =>
            Ok(if n1 == n2 { MalData::True } else { MalData::False }),

        ( MalData::Number(_), _) =>
            Ok(MalData::False),

        ( MalData::String(s1), MalData::String(s2) ) =>
            Ok(if s1 == s2 { MalData::True } else { MalData::False }),

        // ( MalData::List(ref l1), MalData::List(ref l2) )
        //     | ( MalData::List(ref l1), MalData::Vector(ref l2) )
        //     | ( MalData::Vector(ref l1), MalData::List(ref l2) )
        //     | ( MalData::Vector(ref l1), MalData::Vector(ref l2) )
        //     if l1.len() == l2.len() => {
        //         let res = l1.iter().zip(l2.iter()).all( |( e1, e2 )| e1 == e2);
        //         Ok(if res { MalData::True } else { MalData::False })
        //     }

        ( ref l1, ref l2 )
            if is_list_like(&l1) && is_list_like(&l2) => {
            // let res = l1.iter().zip(l2.iter()).all( |( e1, e2 )| e1 == e2);
            Ok(if are_lists_equal(&l1, &l2).unwrap() { MalData::True } else { MalData::False })
        }

        _ =>
            Ok(MalData::False)
    }
}

fn mal_core_prn(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    let print_readably = true;
    let res = itertools::join(args.iter().map(|e| pr_str(e, print_readably)), " ");
    println!("{}", res);

    Ok(MalData::Nil)
}

fn mal_core_println(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    let print_readably = false;
    let res = itertools::join(args.iter().map(|e| pr_str(e, print_readably)), " ");
    println!("{}", res);

    Ok(MalData::Nil)
}

fn mal_core_pr_str(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    let print_readably = true;

    // res.push_str(itertools::join(args.iter().map(|e| pr_str(e, print_readably) ), " ").as_str());
    let res = itertools::join(args.iter().map(|e| pr_str(e, print_readably) ), " ");
    // println!("pr-str, res: {}", res);

    Ok(MalData::String(res))
}

fn mal_core_str(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    let print_readably = false;
    let mut res = String::new();
    
    res.push_str(itertools::join(args.iter().map(|e| pr_str(e, print_readably)), "").as_str());

    // println!("res: {}", res);

    Ok(MalData::String(res))
}

fn mal_core_read_string(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    if let Some(&MalData::String(ref string)) = args.get(0) {
        reader::read_str(&string)
    } else {
        Err("string argument required".to_owned())
    }
}

fn mal_core_slurp(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    if let Some(&MalData::String(ref filename)) = args.get(0) {
        let mut file = File::open(filename).unwrap();  // TODO fehlerbehandlung
        let mut buffer = String::new();

        file.read_to_string(&mut buffer).unwrap();  // TODO fehlerbehandlung

        Ok(MalData::String(buffer))
    } else {
        Err("file name argument required".to_owned())
    }
}

fn mal_core_atom(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    let value = args[0].clone();

    debug!("atom, value: {:?}", value);

    Ok(MalData::Atom(Rc::from(RefCell::from(value))))
}

fn mal_core_atom_p(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    if let MalData::Atom(_) = args[0] { Ok(MalData::True) } else { Ok(MalData::False) }
}

fn mal_core_deref(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    if let MalData::Atom(ref atom) = args[0] {
        Ok(atom.borrow().clone())
    } else {
        Ok(MalData::Nil)
    }
}

fn mal_core_reset(ctx: &FunContext, args: & [MalData]) -> Result<MalData, String> {
    if let MalData::Atom(ref atom) = args[0] {
        let ref new_value = args[1];

        *atom.borrow_mut() = new_value.clone();

        Ok(new_value.clone())
    } else {
        Err("atom expected".to_owned())
    }
}

// mro TODO geeigneten platz finden und dorthin verfrachten
// fn apply_fn_closure(fn_closure: &FnClosure, parameters: &[MalData]) -> Result<MalData, String> {
//     debug!("apply_fn_closure, cl: {:?}, parameters: {:?}", fn_closure, parameters);

//     let outer_env = fn_closure.outer_env.clone();
//     let fn_env = Env::new(Some(outer_env), fn_closure.binds.as_slice(), parameters)?;

//     eval(Rc::new(RefCell::new(fn_env)), fn_closure.body.as_ref())
// }

fn mal_core_swap(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    if let MalData::Atom(ref atom) = args[0] {
        let ref atom_fn = args[1];
        let old_value = atom.borrow().clone();

        debug!("swap!, atom: {:?}, fn: {:?}", atom, atom_fn);

        match atom_fn {
            &MalData::FnClosure(_) | &MalData::Function(_) => {
                let mut list = vec!(atom_fn.clone(), old_value.clone());

                // zusaetzliche parameter uebergeben
                if args.len() > 2 {
                    list.extend_from_slice(&args[2..]);
                }

                let new_value_form = MalData::List(Rc::from(list));

                if let Some(ref eval) = ctx.eval.as_ref() {
                    let new_value = eval(ctx, &[new_value_form])?;
                    debug!("swap!, atom_fn({:?}) -> {:?}", old_value.clone(), new_value);

                    *atom.borrow_mut() = new_value.clone();

                    Ok(new_value)
                } else {
                    return Err("evaluator function not set in context".to_owned())
                }
            }

            _ => 
                return Err("atom value update function expected".to_owned())
        }

    } else {
        Err("atom expected".to_owned())
    }
}

fn mal_core_cons(ctx: &FunContext, args: & [MalData]) -> Result<MalData, String> {
    if let ( Some(head), Some(&MalData::List(ref tail)) ) = ( args.get(0), args.get(1) ) {
        let new_len = 1 + tail.len();
        let mut new_vec: Vec<MalData> = Vec::with_capacity(new_len);

        new_vec.push(head.clone());
        new_vec.extend_from_slice(&tail[..]);

        Ok(MalData::List(Rc::from(new_vec)))
    } else {
        Err("head and tail required".to_owned())
    }
}

fn mal_core_concat(ctx: &FunContext, args: & [MalData]) -> Result<MalData, String> {
    if !args.iter().all( |arg| if let &MalData::List(_) = arg { true } else { false }) {
        return Err("only list arguments allowed".to_owned())
    }

    let mut new_vec = Vec::new();

    for arg in args {
        if let &MalData::List(ref list) = arg {
            new_vec.extend_from_slice(&list[..]);
        }
    }

    Ok(MalData::List(Rc::from(new_vec)))
}

// fn mal_core_(ctx: &FunContext, args: & [MalData]) -> Result<MalData, String> {
// }

pub fn init_ns_map() -> HashMap<&'static str, Rc<CallableFun>> {
    let mut ns_map: HashMap<&str, Rc<CallableFun>> = HashMap::new();

    ns_map.insert("+", Rc::new(mal_core_add));
    ns_map.insert("-", Rc::new(mal_core_sub));
    ns_map.insert("*", Rc::new(mal_core_mul));
    ns_map.insert("/", Rc::new(mal_core_div));
    ns_map.insert("list", Rc::new(mal_core_list));
    ns_map.insert("list?", Rc::new(mal_core_list_p));
    ns_map.insert("empty?", Rc::new(mal_core_empty_p));
    ns_map.insert("count", Rc::new(mal_core_count));
    ns_map.insert("=", Rc::new(mal_core_equals));
    ns_map.insert("<", Rc::new(mal_core_lt));
    ns_map.insert("<=", Rc::new(mal_core_le));
    ns_map.insert(">", Rc::new(mal_core_gt));
    ns_map.insert(">=", Rc::new(mal_core_ge));

    ns_map.insert("pr-str", Rc::new(mal_core_pr_str));
    ns_map.insert("str", Rc::new(mal_core_str));
    ns_map.insert("prn", Rc::new(mal_core_prn));
    ns_map.insert("println", Rc::new(mal_core_println));

    ns_map.insert("read-string", Rc::new(mal_core_read_string));
    ns_map.insert("slurp", Rc::new(mal_core_slurp));

    ns_map.insert("atom", Rc::new(mal_core_atom));
    ns_map.insert("atom?", Rc::new(mal_core_atom_p));
    ns_map.insert("deref", Rc::new(mal_core_deref));
    ns_map.insert("reset!", Rc::new(mal_core_reset));
    ns_map.insert("swap!", Rc::new(mal_core_swap));

    ns_map.insert("cons", Rc::new(mal_core_cons));
    ns_map.insert("concat", Rc::new(mal_core_concat));

    ns_map
}


fn number_arg(arg: &MalData) -> i32 {
    debug!("number_arg, arg: {:?}", arg);

    let number = match *arg {
        MalData::Number(num) => num,

        _ => {
            panic!("arg ist keine zahl");
        }// FIXME fehlerbehandlung
    };

    number
}
