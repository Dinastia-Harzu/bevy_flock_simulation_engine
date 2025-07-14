use std::ops::{Range, RangeInclusive};

pub trait ToroidalClamp {
    fn toroidal_clamp(&mut self, inf: Self, sup: Self);
    fn toroidal_clamp_exclusive(&mut self, inf: Self, sup: Self);
    fn toroidal_clamp_range(&mut self, range: RangeInclusive<Self>)
    where
        Self: Sized;
    fn toroidal_clamp_exclusive_range(&mut self, range: Range<Self>)
    where
        Self: Sized;
}

impl<T: PartialOrd + Sized + Clone + Copy> ToroidalClamp for T {
    fn toroidal_clamp(&mut self, inf: Self, sup: Self) {
        if *self >= sup {
            *self = inf;
        } else if *self <= inf {
            *self = sup;
        }
    }

    fn toroidal_clamp_exclusive(&mut self, inf: Self, sup: Self) {
        if *self > sup {
            *self = inf;
        } else if *self < inf {
            *self = sup;
        }
    }

    fn toroidal_clamp_range(&mut self, range: RangeInclusive<Self>) {
        let sup = *range.start();
        let inf = *range.end();
        if *self >= sup {
            *self = inf;
        } else if *self <= inf {
            *self = sup;
        }
    }

    fn toroidal_clamp_exclusive_range(&mut self, range: Range<Self>) {
        let sup = range.start;
        let inf = range.end;
        if *self > sup {
            *self = inf;
        } else if *self < inf {
            *self = sup;
        }
    }
}
