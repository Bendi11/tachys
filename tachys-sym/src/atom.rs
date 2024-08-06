use symbol::Symbol;

pub mod symbol;

pub struct AtomId();

pub enum Atom {
    Symbol(Symbol),
    Integral {
        upper: AtomId,
        lower: AtomId,
        integrand: AtomId,
    },
    Fraction {
        numerator: AtomId,
        denominator: AtomId,
    },
}
