use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::convert::From;
use std::string::String;

use std::time;
use std::time::SystemTime;

use itertools;

use reader;
use printer::pr_str;

use common::{MalData, CallableFun, FunContext};
use common::{make_mal_list_from_vec, get_wrapped_list, make_mal_keyword, mal_bool_value, is_mal_keyword, is_mal_vector, is_mal_nil, is_mal_true, is_mal_false, make_mal_vector_from_slice, make_mal_map_from_kv_list, is_mal_map, make_mal_list_from_vec_with_meta};
use common::{MapKey, mapkey_for, make_mal_list_from_iter, is_list_like, are_lists_equal, is_mal_string, make_mal_string};
use common::{make_mal_vector_from_vec_with_meta, make_mal_map_from_map_with_meta};

use env::{Env, wrapped_env_type};

type MalCoreFunResult = Result<MalData, String>;

#[allow(unused_variables)]
fn mal_core_add(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    match ( args.get(0).map_or(None, |arg| number_arg(arg)), args.get(1).map_or(None, |arg| number_arg(arg)) ) {
        ( Some(num1), Some(num2) ) =>
            Ok(MalData::Number(num1 + num2)),

        _ =>
            Err("add: two number arguments required".to_string())
    }
}

#[allow(unused_variables)]
fn mal_core_sub(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    match ( args.get(0).map_or(None, |arg| number_arg(arg)), args.get(1).map_or(None, |arg| number_arg(arg)) ) {
        ( Some(num1), Some(num2) ) =>
            Ok(MalData::Number(num1 - num2)),

        _ =>
            Err("sub: two number arguments required".to_string())
    }
}

#[allow(unused_variables)]
fn mal_core_mul(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    match ( args.get(0).map_or(None, |arg| number_arg(arg)), args.get(1).map_or(None, |arg| number_arg(arg)) ) {
        ( Some(num1), Some(num2) ) =>
            Ok(MalData::Number(num1 * num2)),

        _ =>
            Err("mul: two number arguments required".to_string())
    }
}

#[allow(unused_variables)]
fn mal_core_div(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    match ( args.get(0).map_or(None, |arg| number_arg(arg)), args.get(1).map_or(None, |arg| number_arg(arg)) ) {
        ( Some(num1), Some(num2) ) =>
            Ok(MalData::Number(num1.checked_div(num2).ok_or("division failed")?)),

        _ =>
            Err("div: two number arguments required".to_string())
    }
}

#[allow(unused_variables)]
fn mal_core_list(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    Ok(make_mal_list_from_vec(args.to_vec()))
}

#[allow(unused_variables)]
fn mal_core_list_p(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    if args.is_empty() {
        Err("argument required".to_owned())
    } else {
        match args[0] {
            MalData::List(_, _) | MalData::Nil =>
                Ok(MalData::True),

            _ =>
                Ok(MalData::False)
        }
    }
}

#[allow(unused_variables)]
fn mal_core_empty_p(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    if args.is_empty() {
        Err("argument required".to_owned())
    } else {
        match args[0] {
            MalData::List(ref l, _) | MalData::Vector(ref l, _) =>
                Ok(if l.is_empty() { MalData::True } else { MalData::False }),

            MalData::Nil =>
                Ok(MalData::True),

            _ =>
                Ok(MalData::False)
        }
    }
}

#[allow(unused_variables)]
fn mal_core_count(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    if args.is_empty() {
        Err("argument required".to_owned())
    } else {
        match args[0] {
            MalData::Nil =>
                Ok(MalData::Number(0)),

            MalData::List(ref l, _) | MalData::Vector(ref l, _) =>
                Ok(MalData::Number(l.len() as i32)),

            _ =>
                Err("list argument required".to_owned())
        }
    }
}

#[allow(unused_variables)]
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

#[allow(unused_variables)]
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

#[allow(unused_variables)]
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

#[allow(unused_variables)]
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

#[allow(unused_variables)]
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

        ( MalData::Keyword(kw1), MalData::Keyword(kw2) ) =>
            Ok(mal_bool_value(kw1 == kw2)),

        ( MalData::Symbol(sym1), MalData::Symbol(sym2) ) =>
            Ok(mal_bool_value(sym1 == sym2)),

        ( MalData::Number(n1), MalData::Number(n2) ) =>
            Ok(mal_bool_value(n1 == n2)),

        ( MalData::Number(_), _) =>
            Ok(MalData::False),

        ( MalData::String(s1), MalData::String(s2) ) =>
            Ok(mal_bool_value(s1 == s2)),

        ( ref l1, ref l2 ) if is_list_like(&l1) && is_list_like(&l2) => {
            let res = mal_bool_value(are_lists_equal(&l1, &l2)?);
            debug!("equals, l1: {:?}, l2: {:?} -> {:?}", l1, l2, res);
            Ok(res)
        }

        ( MalData::Map(m1, _), MalData::Map(m2, _) ) => {
            let res = mal_bool_value(m1 == m2);
            debug!("equals, m1: {:?}, m2: {:?} -> {:?}", m1, m2, res);
            Ok(res)
        }

        ( l, r ) => {
            debug!("equals, default -> false; l: {:?}, r: {:?}", l, r);
            Ok(MalData::False)
        }
    }
}

#[allow(unused_variables)]
fn mal_core_prn(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    let print_readably = true;
    let res = itertools::join(args.iter().map(|e| pr_str(e, print_readably)), " ");
    println!("{}", res);

    Ok(MalData::Nil)
}

#[allow(unused_variables)]
fn mal_core_println(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    let print_readably = false;
    let res = itertools::join(args.iter().map(|e| pr_str(e, print_readably)), " ");
    println!("{}", res);

    Ok(MalData::Nil)
}

#[allow(unused_variables)]
fn mal_core_pr_str(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    let print_readably = true;

    let res = itertools::join(args.iter().map(|e| pr_str(e, print_readably) ), " ");

    Ok(MalData::String(res))
}

#[allow(unused_variables)]
fn mal_core_str(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    let print_readably = false;
    let mut res = String::new();
    
    res.push_str(itertools::join(args.iter().map(|e| pr_str(e, print_readably)), "").as_str());

    Ok(MalData::String(res))
}

#[allow(unused_variables)]
fn mal_core_read_string(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    if let Some(&MalData::String(ref string)) = args.get(0) {
        reader::read_str(&string)
    } else {
        Err("string argument required".to_owned())
    }
}

#[allow(unused_variables)]
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

#[allow(unused_variables)]
fn mal_core_atom(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    let value = args[0].clone();

    debug!("atom, value: {:?}", value);

    Ok(MalData::Atom(Rc::from(RefCell::from(value))))
}

#[allow(unused_variables)]
fn mal_core_atom_p(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    if let MalData::Atom(_) = args[0] { Ok(MalData::True) } else { Ok(MalData::False) }
}

#[allow(unused_variables)]
fn mal_core_deref(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    if let MalData::Atom(ref atom) = args[0] {
        Ok(atom.borrow().clone())
    } else {
        Ok(MalData::Nil)
    }
}

#[allow(unused_variables)]
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

#[allow(unused_variables)]
fn mal_core_swap(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    if let MalData::Atom(ref atom) = args[0] {
        let ref atom_fn = args[1];
        let old_value = atom.borrow().clone();

        debug!("swap!, atom: {:?}, fn: {:?}", atom, atom_fn);

        // match atom_fn {
        //     &MalData::FnClosure(_) | &MalData::Function(_) => {
                // let mut list = vec!(atom_fn.clone(), old_value.clone());

        let mut list = vec!(old_value.clone());
                // zusaetzliche parameter uebergeben
                if args.len() > 2 {
                    list.extend_from_slice(&args[2..]);
                }

        let new_value = apply_fun(ctx, &atom_fn, &list)?;
        //         let new_value_form = make_mal_list_from_vec(list);
        //         debug!("swap!, new_value_form: {:?}", new_value_form);

        //         if let Some(ref eval) = ctx.eval.as_ref() {
        //             let new_value = eval(ctx, &[new_value_form])?;
                    debug!("swap!, atom_fn({:?}) -> {:?}", old_value.clone(), new_value);

                    *atom.borrow_mut() = new_value.clone();

                    Ok(new_value)
        //         } else {
        //             return Err("evaluator function not set in context".to_owned())
        //         }
        //     }

        //     _ => 
        //         return Err("atom value update function expected".to_owned())
        // }

    } else {
        Err("atom expected".to_owned())
    }
}

#[allow(unused_variables)]
fn mal_core_cons(ctx: &FunContext, args: & [MalData]) -> Result<MalData, String> {
    match ( args.get(0), args.get(1) ) {
        ( Some(head), Some(&MalData::List(ref tail, _)) ) |
        ( Some(head), Some(&MalData::Vector(ref tail, _)) ) => {
            let new_len = 1 + tail.len();
            let mut new_vec: Vec<MalData> = Vec::with_capacity(new_len);

            new_vec.push(head.clone());
            new_vec.extend_from_slice(&tail[..]);

            Ok(make_mal_list_from_vec(new_vec))
        }

        _ =>
            Err("head and tail required".to_owned())
    }
}

fn is_mal_list_or_vector(ast: &MalData) -> bool {
    if let &MalData::List(_, _) = ast {
        true
    } else if let &MalData::Vector(_, _) = ast {
        true
    } else {
        false
    }
}

#[allow(unused_variables)]
fn mal_core_concat(ctx: &FunContext, args: & [MalData]) -> Result<MalData, String> {
    if !args.iter().all( |arg| is_mal_list_or_vector(arg)) {
        return Err("only list and vector arguments allowed".to_owned())
    }

    let mut new_vec = Vec::new();

    for arg in args {
        if let &MalData::List(ref list, _) = arg {
            new_vec.extend_from_slice(&list[..]);
        } else if let &MalData::Vector(ref list, _) = arg {
            new_vec.extend_from_slice(&list[..]);
        }
    }

    Ok(make_mal_list_from_vec(new_vec))
}

fn mal_number_value(mal_num: &MalData) -> Option<i32> {
    if let &MalData::Number(num) = mal_num {
        Some(num)
    } else {
        None
    }
}

fn mal_core_nth(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    let list = args.get(0).map( |l| get_wrapped_list(l) ).ok_or("list argument required")?.unwrap();
    let index = args.get(1).map( |n| mal_number_value(n) ).ok_or("index argument required")?.unwrap();

    if index < 0 || index >= list.len() as i32 {
        Err(format!("index {} out of range for list of size {}", index, list.len()))
    } else {
        Ok(list[index as usize].clone())
    }
}

fn mal_core_first(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    // first von nil -> nil
    if args.get(0).map(|l| is_mal_nil(l) ).unwrap_or(false) {
        return Ok(MalData::Nil)
    }

    let list = args.get(0).map( |l| get_wrapped_list(l) ).ok_or("list argument required")?.unwrap();

    if list.is_empty() {
        Ok(MalData::Nil)
    } else {
        Ok(list.first().unwrap().clone())
    }
}

fn mal_empty_list() -> MalData {
    make_mal_list_from_vec(vec![])
}

fn mal_core_rest(ctx: &FunContext, args: &[MalData]) -> Result<MalData, String> {
    // rest(nil) -> ()
    if args.get(0).map(|l| is_mal_nil(l) ).unwrap_or(false) {
        return Ok(mal_empty_list());
    }

    let list = args.get(0).map_or(None, |l| get_wrapped_list(l) ).ok_or("list argument required")?;

    if list.is_empty() {
        Ok(mal_empty_list())
    } else {
        let rest: Vec<MalData> = list.iter().skip(1).cloned().collect();
        Ok(make_mal_list_from_vec(rest))
    }
}

fn mal_core_throw(ctx: &FunContext, args: & [MalData]) -> MalCoreFunResult {
    if args.len() != 1 {
        Err("exception argument required".to_owned())
    } else {
        Ok(MalData::Exception(Box::from(args[0].clone())))
    }
}

fn apply_fun(ctx: &FunContext, fun: &MalData, args: &[MalData]) -> MalCoreFunResult {
    match fun {
        &MalData::Function(ref fun) => {
            let callable = fun.callable.clone();
            let result = callable(ctx, args);

            debug!("apply_fun, fun: {:?}, args: {:?}\n-> {:?}", fun, args, result);

            result
        }

        &MalData::FnClosure(ref fnc) => {
            let ref eval = ctx.eval2;  // FIXME eval in ctx

            let outer_env = fnc.outer_env.clone();
            let fn_env = Env::new(Some(outer_env), fnc.binds.as_slice(), args)?;

            let res = eval(wrapped_env_type(fn_env), fnc.body.as_ref()).map_err( |e| e.to_string())?;

            debug!("apply_fun, fnc: {:?}, args: {:?}\n-> {:?}", fnc, args, res);

            Ok(res)
        }

        _ => {
            Err(format!("cannot apply {:?}", fun))
        }
    }
}

fn mal_core_apply(ctx: &FunContext, args: &[MalData]) -> MalCoreFunResult {
    if args.len() < 2 {
        return Err("apply: function and argument vector required".to_owned());
    }

    let ref fun_arg = args[0];
    let ref args_arg = args[args.len() - 1];
    let args_arg_list = get_wrapped_list(args_arg).ok_or("apply: invalid argument vector")?;

    let prepend_args = if args.len() > 2 { &args[1..args.len() - 1] } else { &[] };

    debug!("apply, fun: {:?}, args: {:?}, prepend: {:?}", fun_arg, args_arg, prepend_args);

    // TODO erstellung der parameterliste optimieren
    let mut eff_args: Vec<MalData> = Vec::with_capacity(prepend_args.len() + args_arg_list.len());
    eff_args.extend(prepend_args.iter().cloned());
    eff_args.extend(args_arg_list.iter().cloned());

    let res = apply_fun(ctx, fun_arg, eff_args.as_slice())?;

    Ok(res)
}

fn mal_core_map(ctx: &FunContext, args: &[MalData]) -> MalCoreFunResult {
    let fun_arg = args.get(0).ok_or("map: function argument required")?;
    let seq_arg = args.get(1).ok_or("map: sequence argument required")?;

    let seq = get_wrapped_list(seq_arg).ok_or("map: invalid sequence argument")?;

    let mut mapped = Vec::with_capacity(seq.len());

    for el in seq {
        let map_res = apply_fun(ctx, fun_arg, vec![el.clone()].as_slice())?;

        if let MalData::Exception(_) = map_res {
            return Ok(map_res.clone())
        } else {
            mapped.push(map_res);
        }
    }

    Ok(make_mal_list_from_vec(mapped))
}

fn mal_core_nil_p(ctx: &FunContext, args: & [MalData]) -> Result<MalData, String> {
    args.get(0).map( |arg| mal_bool_value(is_mal_nil(arg)) ).ok_or("nil?: argument required".to_owned())
}

fn mal_core_true_p(ctx: &FunContext, args: & [MalData]) -> Result<MalData, String> {
    args.get(0).map( |arg| mal_bool_value(is_mal_true(arg)) ).ok_or("true?: argument required".to_owned())
}

fn mal_core_false_p(ctx: &FunContext, args: & [MalData]) -> Result<MalData, String> {
    args.get(0).map( |arg| mal_bool_value(is_mal_false(arg)) ).ok_or("false?: argument required".to_owned())
}
fn is_mal_symbol(value: &MalData) -> bool {
    if let &MalData::Symbol(_) = value { true } else { false }
}


fn mal_core_symbol_p(ctx: &FunContext, args: & [MalData]) -> MalCoreFunResult {
    args.get(0).map( |arg| mal_bool_value(is_mal_symbol(arg)) ).ok_or("symbol?: argument required".to_owned())
}

fn mal_string_as_string(value: &MalData) -> Option<String> {
    if let &MalData::String(ref string) = value {
        Some(string.clone())
    } else {
        None
    }
}

fn make_mal_symbol(string: &str) -> MalData {
    MalData::Symbol(string.to_string())
}

fn mal_core_symbol(ctx: &FunContext, args: & [MalData]) -> MalCoreFunResult {
    let string = args.get(0).ok_or("symbol: argument required".to_owned())?;
    mal_string_as_string(string).map( |s| make_mal_symbol(&s)).ok_or("symbol: name must be string".to_owned())
}

fn mal_core_keyword(ctx: &FunContext, args: & [MalData]) -> MalCoreFunResult {
    let string = args.get(0).ok_or("keyword: argument required".to_owned())?;
    mal_string_as_string(string).map( |s| make_mal_keyword(&s)).ok_or("keyword: name must be string".to_owned())
}

fn mal_core_keyword_p(ctx: &FunContext, args: & [MalData]) -> MalCoreFunResult {
    args.get(0).map( |arg| mal_bool_value(is_mal_keyword(arg)) ).ok_or("keyword?: argument required".to_owned())
}

fn mal_core_vector(ctx: &FunContext, args: & [MalData]) -> MalCoreFunResult {
    Ok(make_mal_vector_from_slice(args))
}

fn mal_core_vector_p(ctx: &FunContext, args: & [MalData]) -> MalCoreFunResult {
    args.get(0).map( |arg| mal_bool_value(is_mal_vector(arg)) ).ok_or("vector?: argument required".to_owned())
}

fn mal_core_hashmap(ctx: &FunContext, args: & [MalData]) -> MalCoreFunResult {
    if args.len() % 2 != 0 {
        return Err("hash-map: even number of arguments required".to_owned());
    }

    Ok(make_mal_map_from_kv_list(&mut args.iter())?)
}

fn mal_core_map_p(ctx: &FunContext, args: & [MalData]) -> MalCoreFunResult {
    args.get(0).map( |arg| mal_bool_value(is_mal_map(arg)) ).ok_or("map?: argument required".to_owned())
}

fn mal_core_contains_p(ctx: &FunContext, args: & [MalData]) -> MalCoreFunResult {
    if args.len() != 2 {
        return Err("map and key arguments required".to_owned());
    }

    if let ( &MalData::Map(ref map, _), ref key ) = ( &args[0], &args[1] ) {
        Ok(mal_bool_value(map.contains_key(&mapkey_for(&key)?)))
    } else {
        Err("invalid arguments".to_owned())
    }
}

fn mal_core_sequential_p(ctx: &FunContext, args: & [MalData]) -> MalCoreFunResult {
    args.get(0).map( |arg| mal_bool_value(is_list_like(arg)) ).ok_or("sequential?: argument required".to_owned())
}

fn mal_map_key_as_mal_value(key: &MapKey) -> MalData {
    match key {
        &MapKey::String(ref string) => 
            MalData::String(string.clone()),

        &MapKey::Symbol(ref symbol) => 
            MalData::Symbol(symbol.clone()),

        &MapKey::Keyword(ref kw) => 
            MalData::Keyword(kw.clone()),

        &MapKey::Number(number) => 
            MalData::Number(number),

        &MapKey::True => 
            MalData::True,

        &MapKey::False => 
            MalData::False,

        // _ =>
        //     panic!("unhandled map key type: {:?}", key)
    }
}

fn mal_core_keys(ctx: &FunContext, args: &[MalData]) -> MalCoreFunResult {
    if let &MalData::Map(ref map, _) = &args[0] {
        let keys = map.keys().map( |k| mal_map_key_as_mal_value(k) ).collect::<Vec<MalData>>();

        Ok(make_mal_list_from_vec(keys))
    } else {
        Err("keys: map argument required".to_owned())
    }
}

fn mal_core_vals(ctx: &FunContext, args: &[MalData]) -> MalCoreFunResult {
    if let &MalData::Map(ref map, _) = &args[0] {
        let iter: &mut Iterator<Item=&MalData> = &mut map.values();

        Ok(make_mal_list_from_iter(iter))
    } else {
        Err("keys: map argument required".to_owned())
    }
}

fn mal_core_assoc(ctx: &FunContext, args: &[MalData]) -> MalCoreFunResult {
    if args.len() < 2 {
        return Err("map and key/value arguments required".to_owned());
    } else if args.len() % 2 != 1 {
        return Err("key/value pairs required".to_owned());
    }

    if let &MalData::Map(ref map, ref meta) = &args[0] {
        let mut new_map = map.clone();

        let kvs = &args[1..].to_vec();
        let mut kv_iter = kvs.iter();

        while let ( Some(key_arg), Some(value_arg) ) = ( kv_iter.next(), kv_iter.next() ) {
            new_map.insert(mapkey_for(key_arg)?, value_arg.clone());
        }

        Ok(MalData::Map(new_map, meta.clone()))
    } else {
        Err("invalid arguments".to_owned())
    }
    
}

fn mal_core_dissoc(ctx: &FunContext, args: & [MalData]) -> MalCoreFunResult {
    if args.len() < 2 {
        return Err("map and keys arguments required".to_owned());
    }

    if let &MalData::Map(ref map, ref meta) = &args[0] {
        let mut new_map = map.clone();

        for key_arg in &args[1..] {
            new_map.remove(&mapkey_for(key_arg)?);
        }

        Ok(MalData::Map(new_map, meta.clone()))
    } else {
        Err("invalid arguments".to_owned())
    }
}

fn mal_core_get(ctx: &FunContext, args: & [MalData]) -> MalCoreFunResult {
    if args.len() != 2 {
        return Err("map and key arguments required".to_owned());
    }

    if let &MalData::Nil = &args[0] {
        Ok(MalData::Nil)
    } else if let ( &MalData::Map(ref map, _), ref key ) = ( &args[0], &args[1] ) {
        Ok(map.get(&mapkey_for(&key)?).map_or(MalData::Nil, |v| v.clone()))
    } else {
        Err("invalid arguments".to_owned())
    }
}

fn mal_core_readline(ctx: &FunContext, args: & [MalData]) -> MalCoreFunResult {
    let prompt = args.get(0).ok_or("prompt argument required")?;

    print!("{}", mal_string_as_string(prompt).ok_or("prompt must be a string")?);
    io::stdout().flush();

    let mut line = String::new();

    io::stdin().read_line(&mut line)
        .map( |c| if c > 0 { MalData::String(line[0..c - 1].to_string()) } else { MalData::String("".to_string()) })
        .map_err( |e| format!("{}", e))
} 

fn mal_core_string_p(ctx: &FunContext, args: & [MalData]) -> MalCoreFunResult {
    args.get(0).ok_or("argument required".to_string()).map( |arg| mal_bool_value(is_mal_string(arg)) )
}

fn mal_core_seq(ctx: &FunContext, args: & [MalData]) -> MalCoreFunResult {
    match args.get(0) {
        Some(&MalData::List(ref lst, _)) | Some(&MalData::Vector(ref lst, _)) if lst.is_empty() =>
            Ok(MalData::Nil),

        Some(&MalData::List(ref lst, _)) | Some(&MalData::Vector(ref lst, _)) =>
            Ok(make_mal_list_from_iter(&mut lst.iter())),

        Some(&MalData::String(ref string)) if string.is_empty() =>
            Ok(MalData::Nil),

        Some(&MalData::String(ref string)) => {
            let mut vec = string.chars().map( |c| make_mal_string(&c.to_string())).collect();

            Ok(make_mal_list_from_vec(vec))
        }

        Some(&MalData::Nil) =>
            Ok(MalData::Nil),

        Some(arg) =>
            Err(format!("seq: argument of illegal type: {:?}", arg)),

        None =>
            Err("seq: argument required".to_string())
    }
}

fn mal_core_conj(ctx: &FunContext, args: & [MalData]) -> MalCoreFunResult {
    if args.len() < 2 {
        return Err("conj: seq arguments required".to_string());
    }

    match ( &args[0], &args[1..] ) {
        ( &MalData::List(ref lst, _), xs) => {
            let mut res = Vec::new();
            res.extend_from_slice(lst);

            for x in xs {
                res.insert(0, x.clone());
            }

            Ok(make_mal_list_from_vec(res))
        }

        ( &MalData::Vector(ref vec, _), xs ) => {
            let mut res = Vec::new();
            res.extend_from_slice(vec);

            for x in xs {
                res.push(x.clone());
            }

            Ok(make_mal_vector_from_slice(&res))
        }

        ( seq, _ ) =>
            Err(format!("illegal type for argument: {:?}", seq))
    }
}

fn make_mal_number(number: i32) -> MalData {
    MalData::Number(number)
}

fn mal_core_time_ms(ctx: &FunContext, args: & [MalData]) -> MalCoreFunResult {
    let de = SystemTime::now().duration_since(time::UNIX_EPOCH);
    debug!("time-ms, de: {:?}", de);
    let msecs_since_epoch = SystemTime::now().duration_since(time::UNIX_EPOCH)
        .map( |dur| dur.as_secs() * 1_000 + (dur.subsec_nanos() / 1_000_000) as u64)
        .map_err( |err| format!("{}", err))?;

    let res = make_mal_number(msecs_since_epoch as i32);
    debug!("time-ms, res: {:?}", res);
    Ok(res)  // FIXME datentyp fuer zahlen
}

fn mal_core_with_meta(ctx: &FunContext, args: & [MalData]) -> MalCoreFunResult {
    if args.len() < 2 {
        return Err("value and metadate arguments required".to_string());
    }

    let with_meta = match &args[0] {
        &MalData::FnClosure(ref fnc) =>
            MalData::FnClosure(fnc.with_meta(&args[1])),

        &MalData::Function(ref fun) =>
            MalData::Function(fun.with_meta(&args[1])),

        &MalData::List(ref lst, _) => {
            make_mal_list_from_vec_with_meta(lst, &args[1])
        }

        &MalData::Vector(ref lst, _) => {
            make_mal_vector_from_vec_with_meta(lst, &args[1])
        }

        &MalData::Map(ref map, _) => {
            make_mal_map_from_map_with_meta(map, &args[1])?
        }

        _ => {
            warn!("with_meta, default; value: {:?}, metadata: {:?}", &args[0], &args[1]);
            args[0].clone()
        }
    };

    Ok(with_meta)
}

fn mal_core_meta(ctx: &FunContext, args: & [MalData]) -> MalCoreFunResult {
    if args.len() != 1 {
        return Err("value argument required".to_string());
    }

    match &args[0] {
        &MalData::FnClosure(ref fnc) => {
            Ok(fnc.get_meta().map_or(MalData::Nil, |m| *m.clone()))
        }

        &MalData::Function(ref fun ) => {
            Ok(fun.get_meta().map_or(MalData::Nil, |m| *m.clone()))
        }

        &MalData::List(_, Some(ref meta) ) => {
            Ok(*meta.clone())
        }

        &MalData::Vector(_, Some(ref meta) ) => {
            Ok(*meta.clone())
        }

        &MalData::Map(_, Some(ref meta) ) => {
            Ok(*meta.clone())
        }

        _ =>
            Ok(MalData::Nil)
    }
}

// fn mal_core_(ctx: &FunContext, args: & [MalData]) -> MalCoreFunResult {
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

    ns_map.insert("nth", Rc::new(mal_core_nth));
    ns_map.insert("first", Rc::new(mal_core_first));
    ns_map.insert("rest", Rc::new(mal_core_rest));

    ns_map.insert("throw", Rc::new(mal_core_throw));
    ns_map.insert("apply", Rc::new(mal_core_apply));
    ns_map.insert("map", Rc::new(mal_core_map));
    ns_map.insert("nil?", Rc::new(mal_core_nil_p));
    ns_map.insert("true?", Rc::new(mal_core_true_p));
    ns_map.insert("false?", Rc::new(mal_core_false_p));
    ns_map.insert("symbol?", Rc::new(mal_core_symbol_p));
    ns_map.insert("symbol", Rc::new(mal_core_symbol));
    ns_map.insert("keyword", Rc::new(mal_core_keyword));
    ns_map.insert("keyword?", Rc::new(mal_core_keyword_p));
    ns_map.insert("vector", Rc::new(mal_core_vector));
    ns_map.insert("vector?", Rc::new(mal_core_vector_p));
    ns_map.insert("hash-map", Rc::new(mal_core_hashmap));
    ns_map.insert("map?", Rc::new(mal_core_map_p));
    ns_map.insert("assoc", Rc::new(mal_core_assoc));
    ns_map.insert("dissoc", Rc::new(mal_core_dissoc));
    ns_map.insert("get", Rc::new(mal_core_get));
    ns_map.insert("contains?", Rc::new(mal_core_contains_p));
    ns_map.insert("keys", Rc::new(mal_core_keys));
    ns_map.insert("vals", Rc::new(mal_core_vals));
    ns_map.insert("sequential?", Rc::new(mal_core_sequential_p));

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
