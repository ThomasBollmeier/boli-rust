use std::ops::Add;

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

impl Add<IntValue> for IntValue {
    type Output = IntValue;

    fn add(self, other: IntValue) -> IntValue {
        IntValue {
            value: self.value + other.value,
        }
    }
}

impl Add<RationalValue> for IntValue {
    type Output = RationalValue;

    fn add(self, other: RationalValue) -> RationalValue {
        RationalValue::new(
            self.value * other.denominator + other.numerator,
            other.denominator,
        )
    }
}

impl Add<RealValue> for IntValue {
    type Output = RealValue;

    fn add(self, other: RealValue) -> RealValue {
        RealValue {
            value: self.value as f64 + other.value,
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

impl Add<IntValue> for RationalValue {
    type Output = RationalValue;

    fn add(self, other: IntValue) -> Self::Output {
        other + self
    }
}

impl Add<RationalValue> for RationalValue {
    type Output = RationalValue;

    fn add(self, other: RationalValue) -> RationalValue {
        RationalValue::new(
            self.numerator * other.denominator + other.numerator * self.denominator,
            self.denominator * other.denominator,
        )
    }
}

impl Add<RealValue> for RationalValue {
    type Output = RealValue;

    fn add(self, other: RealValue) -> RealValue {
        RealValue {
            value: self.numerator as f64 / self.denominator as f64 + other.value,
        }
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

impl Add<IntValue> for RealValue {
    type Output = RealValue;

    fn add(self, other: IntValue) -> RealValue {
        other + self
    }
}

impl Add<RationalValue> for RealValue {
    type Output = RealValue;

    fn add(self, other: RationalValue) -> RealValue {
        other + self
    }
}

impl Add<RealValue> for RealValue {
    type Output = RealValue;

    fn add(self, other: RealValue) -> RealValue {
        RealValue {
            value: self.value + other.value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_add_int() {
        let value1 = IntValue { value: 42 };
        let value2 = IntValue { value: 43 };

        let result = value1 + value2;
        assert_eq!(result.value, 85);
    }

    #[test]
    fn test_int_add_rational() {
        let value1 = IntValue { value: 2 };
        let value2 = RationalValue::new(1, 2);

        let result = value1 + value2;
        assert_eq!(result.numerator, 5);
        assert_eq!(result.denominator, 2);
    }

    #[test]
    fn test_int_add_real() {
        let value1 = IntValue { value: 2 };
        let value2 = RealValue { value: 1.5 };

        let result = value1 + value2;
        assert_eq!(result.value, 3.5);
    }

    #[test]
    fn test_rational_add_int() {
        let value1 = RationalValue::new(1, 2);
        let value2 = IntValue { value: 2 };

        let result = value1 + value2;
        assert_eq!(result.numerator, 5);
        assert_eq!(result.denominator, 2);
    }

    #[test]
    fn test_rational_add_rational() {
        let value1 = RationalValue::new(1, 3);
        let value2 = RationalValue::new(5, 3);

        let result = value1 + value2;
        assert_eq!(result.numerator, 2);
        assert_eq!(result.denominator, 1);
        assert_eq!(format!("{result}"), "2");
    }

    #[test]
    fn test_rational_add_real() {
        let value1 = RationalValue::new(1, 2);
        let value2 = RealValue { value: 1.5 };

        let result = value1 + value2;
        assert_eq!(result.value, 2.0);
    }

    #[test]
    fn test_real_add_int() {
        let value1 = RealValue { value: 1.5 };
        let value2 = IntValue { value: 2 };

        let result = value1 + value2;
        assert_eq!(result.value, 3.5);
    }

    #[test]
    fn test_real_add_rational() {
        let value1 = RealValue { value: 1.5 };
        let value2 = RationalValue::new(1, 2);

        let result = value1 + value2;
        assert_eq!(result.value, 2.0);
    }

    #[test]
    fn test_real_add_real() {
        let value1 = RealValue { value: 1.5 };
        let value2 = RealValue { value: 2.0 };

        let result = value1 + value2;
        assert_eq!(result.value, 3.5);
    }
}
