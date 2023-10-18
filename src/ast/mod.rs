pub mod ast_def;


use lalrpop_util::lalrpop_mod;

lalrpop_mod! {
    #[allow(clippy::all)]
    pub grammar, "/ast/grammar.rs"
}