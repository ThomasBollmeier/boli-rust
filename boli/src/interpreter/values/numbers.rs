use super::*;

#[derive(Debug)]
pub struct IntValue {
    pub value: i64,
}

impl Value for IntValue {
    fn get_type(&self) -> ValueType {
        ValueType::Int
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Display for IntValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl ComparableEq for IntValue {
    fn is_equal(&self, other: &ValueRef) -> bool {
        if let Some(other) = downcast_value::<IntValue>(&other.borrow()) {
            self.value == other.value
        } else {
            false
        }
    }
}

#[derive(Debug)]
pub struct RationalValue {
    pub numerator: i64,
    pub denominator: i64,
}

impl RationalValue {
    pub fn new(numerator: i64, denominator: i64) -> Self {
        let mut ret = Self {
            numerator,
            denominator,
        };
        ret.simplify();
        ret
    }

    fn simplify(&mut self) {
        let gcd = Self::gcd(self.numerator, self.denominator);
        self.numerator /= gcd;
        self.denominator /= gcd;

        if self.numerator > 0 && self.denominator < 0 {
            self.numerator *= -1;
            self.denominator *= -1;
        } else if self.numerator < 0 && self.denominator < 0 {
            self.numerator *= -1;
            self.denominator *= -1;
        }
    }

    fn gcd(a: i64, b: i64) -> i64 {
        if b == 0 {
            a
        } else {
            Self::gcd(b, a % b)
        }
    }
}

impl Value for RationalValue {
    fn get_type(&self) -> ValueType {
        ValueType::Rational
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Display for RationalValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.denominator == 1 {
            return write!(f, "{}", self.numerator);
        }
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}

#[derive(Debug)]
pub struct RealValue {
    pub value: f64,
}

impl Value for RealValue {
    fn get_type(&self) -> ValueType {
        ValueType::Real
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Display for RealValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value_str = format!("{:?}", self.value).replace(".", ",");
        write!(f, "{value_str}")
    }
}
