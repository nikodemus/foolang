#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Decimal(i64),
    Float(f64),
    Character(char),
    Selector(String),
    String(String),
    Array(Vec<Literal>),
    Record(Vec<String>, Vec<Literal>),
}
