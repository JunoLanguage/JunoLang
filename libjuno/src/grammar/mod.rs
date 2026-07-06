use pest_derive::Parser;
#[derive(Parser)]
#[grammar = "src/grammar/juno.pest"]
pub struct JunoParser;

pub use self::Rule as JunoParserRule;