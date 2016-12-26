use std::collections::HashMap;
use std::rc::Rc;

use itertools;

use printer::pr_str;
use common::{MalData, CallableFun};

fn mal_core_add(args: &[MalData]) -> Result<MalData, String> {
    let res = number_arg(&args[0]) + number_arg(&args[1]);  // TODO fehlerbehandlung
    Ok(MalData::Number(res))
}

fn mal_core_sub(args: &[MalData]) -> Result<MalData, String> {
    let res = number_arg(&args[0]) - number_arg(&args[1]);  // TODO fehlerbehandlung
    Ok(MalData::Number(res))
}

fn mal_core_mul(args: &[MalData]) -> Result<MalData, String> {
    let res = number_arg(&args[0]) * number_arg(&args[1]);  // TODO fehlerbehandlung
    Ok(MalData::Number(res))
}

fn mal_core_div(args: &[MalData]) -> Result<MalData, String> {
    let res = number_arg(&args[0]).checked_div(number_arg(&args[1])).unwrap();  // TODO fehlerbehandlung
    Ok(MalData::Number(res))
}

fn mal_core_list(args: &[MalData]) -> Result<MalData, String> {
    Ok(MalData::List(Rc::from(args.to_vec())))
}

fn mal_core_list_p(args: &[MalData]) -> Result<MalData, String> {
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

fn mal_core_empty_p(args: &[MalData]) -> Result<MalData, String> {
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

fn mal_core_count(args: &[MalData]) -> Result<MalData, String> {
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

fn mal_core_lt(args: &[MalData]) -> Result<MalData, String> {
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

fn mal_core_le(args: &[MalData]) -> Result<MalData, String> {
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

fn mal_core_gt(args: &[MalData]) -> Result<MalData, String> {
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

fn mal_core_ge(args: &[MalData]) -> Result<MalData, String> {
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

fn mal_core_equals(args: &[MalData]) -> Result<MalData, String> {
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

fn mal_core_prn(args: &[MalData]) -> Result<MalData, String> {
    let print_readably = true;
    let res = itertools::join(args.iter().map(|e| pr_str(e, print_readably)), " ");
    println!("{}", res);

    Ok(MalData::Nil)
}

fn mal_core_println(args: &[MalData]) -> Result<MalData, String> {
    let print_readably = false;
    let res = itertools::join(args.iter().map(|e| pr_str(e, print_readably)), " ");
    println!("{}", res);

    Ok(MalData::Nil)
}

fn mal_core_pr_str(args: &[MalData]) -> Result<MalData, String> {
    let print_readably = true;

    // res.push_str(itertools::join(args.iter().map(|e| pr_str(e, print_readably) ), " ").as_str());
    let res = itertools::join(args.iter().map(|e| pr_str(e, print_readably) ), " ");
    // println!("pr-str, res: {}", res);

    Ok(MalData::String(res))
}

fn mal_core_str(args: &[MalData]) -> Result<MalData, String> {
    let print_readably = false;
    let mut res = String::new();
    
    res.push_str(itertools::join(args.iter().map(|e| pr_str(e, print_readably)), "").as_str());

    // println!("res: {}", res);

    Ok(MalData::String(res))
}


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
