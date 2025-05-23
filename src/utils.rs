use crate::*;
use std::process::exit;

/// ### public function to set global variable optimized (optimized extraction flag)
/// #### Argument
/// * `optimized` - optimized extraction flag
/// #### Return
/// * `None`
pub unsafe fn set_optimized_flag(optimized: bool) {
    OPTIMIZED = optimized;
    return;
}

/// ### public function to set global variable num_equiv_exprs
/// #### Argument
/// * `token_limit` - token limit
/// #### Return
/// * `None`
pub unsafe fn set_n_equiv_exprs(n_equiv_exprs: u8) {
    N_EQUIV_EXPRS = n_equiv_exprs;
    return;
}

/// ### public function to set global variable token_limit
/// #### Argument
/// * `token_limit` - token_limit
/// #### Return
/// * `None`
pub unsafe fn set_token_limit(token_limit: u8) {
    TOKEN_LIMIT = token_limit;
    return;
}

/// ### public function to set global variable max_token_limit
/// #### Argument
/// * `max_token_limit` - maximum token limit
/// #### Return
/// * `None`
pub unsafe fn set_max_token_limit(max_token_limit: u8) {
    MAX_TOKEN_LIMIT = max_token_limit;
    return;
}

/// ### public function to set global variable time limit
/// #### Argument
/// * `time_limit` - token limit
/// #### Return
/// * `None`
pub unsafe fn set_time_limit(time_limit: u16) {
    TIME_LIMIT = time_limit;
    return;
}

/// ### public function to set global variable start_time
/// #### Argument
/// * `start_time` - start time
/// #### Return
/// * `None`
pub unsafe fn set_start_time(start_time: Instant) {
    START_TIME = Some(start_time);
    return;
}

/// ### public function to get global variable start_time
/// #### Argument
/// * `None`
/// #### Return
/// * `Instant` - current time
pub unsafe fn get_start_time() -> Instant {
    let start_time = match START_TIME {
        Some(start_time) => { start_time },
        _ => {
            log_error(&format!("Failed to unwrap global variable START_TIME '{:?}'.\n", START_TIME));
            exit(1);
        },
    };

    return start_time;
}

/// ### public function to remove permutations from the final results
/// ### of equivalent expression
/// #### Argument
/// * `equiv_exprs` - deduplicate results of equivalent expressions
/// #### Return
/// * `HashSet<String>` - distinct equivalent expressions
pub fn rm_permu(equiv_exprs: &HashSet<String>) -> HashSet<String> {
    let mut expr_mapping = HashMap::default();

    for expr in equiv_exprs.clone().into_iter() {
        let mut tokens: Vec<&str> = expr.split_whitespace().collect();
        tokens.sort();
        let expr_sort: String = tokens.join(" ");

        if !expr_mapping.contains_key(&expr_sort) {
            expr_mapping.insert(expr_sort, expr);
        }
    }

    let equiv_exprs_distinct = expr_mapping.into_values().collect();

    return equiv_exprs_distinct;
}

/// ### public function to print the type of a variable
/// #### Argument
/// * `_` - reference of any variable
/// #### Return
/// * `None`
pub fn pt_type_of<T>(_: &T) {
    log_debug(&format!("Variable Type {}", std::any::type_name::<T>()));
    return;
}

/// ### public function to print egraph information
/// ### total # of e-class(es) & total # of e-node(s)
/// #### Argument
/// * `egraph` - egraph
/// #### Return
/// * `None`
pub fn pt_egraph_info(egraph: &MathEGraph) {
    log_info(&format!("Total # of e-class(es):  {}\n", &egraph.number_of_classes()));
    log_info(&format!("Total # of e-node(s):    {}\n", &egraph.total_number_of_nodes()));

    return;
}

/// ### public function to print egraph information
/// #### Argument
/// * `egraph` - egraph
/// #### Return
/// * `None`
pub fn pt_egraph(egraph: &MathEGraph) {
    log_debug_raw("\n");
    log_debug("------- EGraph Information -------\n");
    log_debug("------------- EClass -------------\n");
    for eclass in egraph.classes() {
        log_debug(&format!("------------ EClass {} ------------\n", eclass.id));
        for i in 0..eclass.nodes.len() {
            if eclass.nodes[i].to_string().parse::<f64>().is_ok() {
                log_debug(&format!("<f64> {}", eclass.nodes[i]));
            } else {
                log_debug(&format!("enode {}", eclass.nodes[i]));
                for k in 0..eclass.nodes[i].children().len() {
                    log_debug_raw(&format!(" {}", eclass.nodes[i].children()[k]));
                }
            }
            log_debug_raw("\n");
        }
        log_debug("parents");
        for k in 0..eclass.parents().len() {
            log_debug_raw(&format!(" {:?}", eclass.parents().nth(k).unwrap()));
        }
        log_debug_raw("\n");
        log_debug(&format!("data {:?}\n", eclass.data));
    }
    log_debug("----------------------------------\n");
    return;
}

/// ### public function to print root eclasses
/// #### Argument
/// * `root_eclasses` - root eclass vec<Id>
/// #### Return
/// * `None`
pub fn pt_root_ecls_info(root_ecls: &Vec<Id>) {
    log_debug_raw("\n");
    log_debug("---------- Root EClasses ----------\n");
    log_debug(&format!("{:?}\n", root_ecls));
    log_debug("-----------------------------------\n");
    return;
}

/// ### public function to print eclass(es) to skip during extraction
/// #### Argument
/// * `skip_ecls` - vec<String> to skip during extraction
/// #### Return
/// * `None`
pub fn pt_skip_ecls(skip_ecls: &HashMap<String, f64>) {
    log_debug_raw("\n");
    log_debug("---------- Skip EClasses ----------\n");
    log_debug(&format!("{:?}\n", skip_ecls));
    log_debug("-----------------------------------\n");
    return;
}

/// ### public function to print grammar
/// #### Argument
/// * `grammar` - grammar HashMap
/// #### Return
/// * `None`
pub fn pt_grammar(grammar: &HashMap<String, Vec<String>>){
    log_debug_raw("\n");
    log_debug("------------- Grammar -------------\n");
    for (eclass, rewrite) in grammar {
        log_debug(&format!("{} -> {:?}\n", eclass, rewrite));
    }
    log_debug("-----------------------------------\n");
    return;
}

/// ### public function to print initial rewrites
/// #### Argument
/// * `init_rw` - init_rw Vec<String>
/// #### Return
/// * `None`
pub fn pt_init_rw(init_rw: &Vec<String>) {
    log_debug_raw("\n");
    log_debug("--------- Initial Rewrite ---------\n");
    log_debug(&format!("{:?}\n", init_rw));
    log_debug("-----------------------------------\n");
    return;
}
