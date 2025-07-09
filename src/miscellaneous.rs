pub trait ToroidalClamp {
    fn toroidal_clamp(&mut self, inf: Self, sup: Self);
    fn toroidal_clamp_exclusive(&mut self, inf: Self, sup: Self);
}

impl<T: PartialOrd> ToroidalClamp for T {
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
}
