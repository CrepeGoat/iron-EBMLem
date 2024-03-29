pub use core::ops::Bound;

pub enum Range<T> {
    IsExactly(T),
    Excludes(T),
    IsWithin(Bound<T>, Bound<T>),
}

pub trait ElementDef {
    // name
    const ID: u32;
    const PATH: &'static str;

    const MIN_OCCURS: usize; // defaults to 0
    const MAX_OCCURS: Option<usize>; // defaults to usize::MAX
    const LENGTH: Range<usize>; // defaults to type-defined length limits
    const RECURRING: bool; // defaults to false
    const MIN_VERSION: u64; // defaults to 1
    const MAX_VERSION: Option<u64>; // defaults to "EBMLSchema"'s "version" attribute
}

pub trait MasterElementDef: ElementDef {
    const UNKNOWN_SIZE_ALLOWED: bool; // defaults to false
    const RECURSIVE: bool; // defaults to false
}

pub trait UIntElementDef: ElementDef {
    const RANGE: Range<u64>; // defaults to (Unbounded, Unbounded)
    const DEFAULT: Option<u64>;
}

pub trait IntElementDef: ElementDef {
    const RANGE: Range<i64>; // defaults to (Unbounded, Unbounded)
    const DEFAULT: Option<i64>;
}

pub trait FloatElementDef: ElementDef {
    const RANGE: Range<f64>; // defaults to (Unbounded, Unbounded)
    const DEFAULT: Option<f64>;
}

pub trait DateElementDef: ElementDef {
    const RANGE: Range<i64>; // defaults to (Unbounded, Unbounded)
    const DEFAULT: Option<i64>;
}

pub trait StringElementDef: ElementDef {
    const DEFAULT: Option<&'static str>;
}

pub trait Utf8ElementDef: ElementDef {
    const DEFAULT: Option<&'static str>;
}

pub trait BinaryElementDef: ElementDef {
    const DEFAULT: Option<&'static [u8]>;
}
