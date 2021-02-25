use std::ops::Bound;

pub enum RangeDef<T> {
    IsExactly(T),
    Excludes(T),
    IsWithin(Bound<T>, Bound<T>),
}

pub trait Element {
    // name
    // path
    const ID: u32;

    const MIN_OCCURS: Option<usize>;
    const MAX_OCCURS: Option<usize>;
    const LENGTH: Option<RangeDef<usize>>;
    const RECURRING: Option<bool>;
    const MIN_VERSION: Option<u64>;
    const MAX_VERSION: Option<u64>;
}

pub trait MasterElement: Element {
    const UNKNOWN_SIZE_ALLOWED: Option<bool>;
    const RECURSIVE: Option<bool>;
}

pub trait UIntElement: Element {
    const RANGE: Option<RangeDef<u64>>;
    const DEFAULT: Option<u64>;
}

pub trait IntElement: Element {
    const RANGE: Option<RangeDef<i64>>;
    const DEFAULT: Option<i64>;
}

pub trait FloatElement: Element {
    const RANGE: Option<RangeDef<f64>>;
    const DEFAULT: Option<f64>;
}

pub trait DateElement: Element {
    const RANGE: Option<RangeDef<i64>>;
    const DEFAULT: Option<i64>;
}

pub trait StringElement: Element {
    const DEFAULT: Option<&'static str>;
}

pub trait UTF8Element: Element {
    const DEFAULT: Option<&'static str>;
}

pub trait BinaryElement: Element {
    const DEFAULT: Option<&'static [u8]>;
}