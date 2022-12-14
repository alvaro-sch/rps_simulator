use std::{f32::consts as f32, ops::Mul};

#[derive(Debug, Clone, Copy)]
pub struct Rad(pub f32);

#[derive(Debug, Clone, Copy)]
pub struct Deg(pub f32);

impl From<Deg> for Rad {
    fn from(deg: Deg) -> Rad {
        Rad(deg.0 * f32::PI / 180.0)
    }
}

impl From<Rad> for Deg {
    fn from(rad: Rad) -> Deg {
        Deg(rad.0 * 180.0 / f32::PI)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform(pub [f32; 9]);

impl Transform {
    pub const fn col_major(t: [f32; 9]) -> Self {
        Self(t)
    }

    pub const fn row_major(t: [f32; 9]) -> Self {
        #[rustfmt::skip]
        let t = [
            t[0], t[3], t[6],
            t[1], t[4], t[7],
            t[2], t[5], t[8],
        ];

        Self(t)
    }

    pub const fn identity() -> Self {
        #[rustfmt::skip]
        let id3 = [
            1.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
            0.0, 0.0, 1.0,
        ];

        Self(id3)
    }

    pub fn translate(self, offset: [f32; 2]) -> Self {
        #[rustfmt::skip]
        let translation = [
            1.0,       0.0,       0.0,
            0.0,       1.0,       0.0,
            offset[0], offset[1], 1.0,
        ];

        Self(translation) * self
    }

    pub fn scale(self, scalars: [f32; 2]) -> Self {
        #[rustfmt::skip]
        let scale = [
            scalars[0], 0.0,        0.0,
            0.0,        scalars[1], 0.0,
            0.0,        0.0,        1.0,
        ];

        Self(scale) * self
    }

    pub fn rotate<A>(self, theta: A) -> Self
    where
        A: Into<Rad>,
    {
        let (sin_theta, cos_theta) = f32::sin_cos(theta.into().0);

        #[rustfmt::skip]
        let rotation = [
            cos_theta,  sin_theta, 0.0,
            -sin_theta, cos_theta, 0.0,
            0.0,        0.0,       1.0,
        ];

        Self(rotation) * self
    }
}

impl Into<[f32; 9]> for Transform {
    fn into(self) -> [f32; 9] {
        self.0
    }
}

impl Mul for Transform {
    type Output = Transform;

    fn mul(self, rhs: Self) -> Self::Output {
        let x = self.0;
        let y = rhs.0;

        let mut z = [0.0; 9];
        z[0] = x[0] * y[0] + x[3] * y[1] + x[6] * y[2];
        z[1] = x[1] * y[0] + x[4] * y[1] + x[7] * y[2];
        z[2] = x[2] * y[0] + x[5] * y[1] + x[8] * y[2];

        z[3] = x[0] * y[3] + x[3] * y[4] + x[6] * y[5];
        z[4] = x[1] * y[3] + x[4] * y[4] + x[7] * y[5];
        z[5] = x[2] * y[3] + x[5] * y[4] + x[8] * y[5];

        z[6] = x[0] * y[6] + x[3] * y[7] + x[6] * y[8];
        z[7] = x[1] * y[6] + x[4] * y[7] + x[7] * y[8];
        z[8] = x[2] * y[6] + x[5] * y[7] + x[8] * y[8];

        Self(z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn epsilon_eq(epsilon: f32, x: f32, y: f32) -> bool {
        (x - y).abs() <= epsilon
    }

    fn transform_epsilon_eq(epsilon: f32, x: Transform, y: Transform) -> bool {
        let (x, y) = (x.0, y.0);
        (0..x.len()).all(|i| epsilon_eq(epsilon, x[i], y[i]))
    }

    #[test]
    fn deg_to_rad() {
        // good enough just to check if the angle is correct
        const EPSILON: f32 = f32::PI / 180.0;

        fn rad(deg: f32) -> f32 {
            Rad::from(Deg(deg)).0
        }

        assert!(epsilon_eq(EPSILON, rad(180.0), f32::PI));
        assert!(epsilon_eq(EPSILON, rad(90.0), f32::FRAC_PI_2));
        assert!(epsilon_eq(EPSILON, rad(45.0), f32::FRAC_PI_4));
        assert!(epsilon_eq(EPSILON, rad(30.0), f32::FRAC_PI_6));
    }

    #[test]
    fn rad_to_deg() {
        // good enough just to check if the angle is correct
        const EPSILON: f32 = 1.0;

        fn deg(rad: f32) -> f32 {
            Deg::from(Rad(rad)).0
        }

        assert!(epsilon_eq(EPSILON, deg(f32::PI), 180.0));
        assert!(epsilon_eq(EPSILON, deg(f32::FRAC_PI_2), 90.0));
        assert!(epsilon_eq(EPSILON, deg(f32::FRAC_PI_4), 45.0));
        assert!(epsilon_eq(EPSILON, deg(f32::FRAC_PI_6), 30.0));
    }

    #[test]
    fn transform_mat3x3_multiplication() {
        const EPSILON: f32 = 1e-2;

        let identity = Transform::identity();

        #[rustfmt::skip]
        let x = Transform::row_major([
            1.0, 2.0, 3.0,
            4.0, 5.0, 6.0,
            7.0, 8.0, 9.0,
        ]);

        #[rustfmt::skip]
        let y = Transform::row_major([
            1.1, 8.3, 1.0,
            6.4, 5.2, 6.0,
            3.9, 4.3, 7.1,
        ]);

        #[rustfmt::skip]
        let z = Transform::row_major([
            25.6, 31.6,  34.3,
            59.8, 85.0,  76.6,
            94.0, 138.4, 118.9,
        ]);

        assert_eq!(x * identity, x);
        assert_eq!(y * identity, y);
        assert_eq!(z * identity, z);
        assert!(transform_epsilon_eq(EPSILON, x * y, z));
    }

    #[test]
    fn affine_transformations() {
        const EPSILON: f32 = f32::EPSILON;

        let t0 = Transform::identity();
        let t1 = t0.scale([2.3, 1.7]);
        let t2 = t1.rotate(Deg(45.0));
        let t3 = t2.translate([33.8, 4.9]);
        let t4 = t3.rotate(Rad(-2.23));

        #[rustfmt::skip]
        let t1_expected = Transform::row_major([
            2.3, 0.0, 0.0,
            0.0, 1.7, 0.0,
            0.0, 0.0, 1.0,
        ]);

        #[rustfmt::skip]
        let t2_expected = Transform::row_major([
            1.6263455, -1.2020816, 0.0,
            1.6263455,  1.2020816, 0.0,
            0.0,        0.0,       1.0,
        ]);

        #[rustfmt::skip]
        let t3_expected = Transform::row_major([
            1.6263455, -1.2020816, 33.8,
            1.6263455,  1.2020816, 4.9,
            0.0,        0.0,       1.0,
        ]);

        #[rustfmt::skip]
        let t4_expected = Transform::row_major([
            0.28947747, 1.6864817,  -16.828728,
            -2.2817101, 0.21396166, -29.719418,
            0.0,        0.0,        1.0,
        ]);

        assert!(transform_epsilon_eq(EPSILON, t1, t1_expected));
        assert!(transform_epsilon_eq(EPSILON, t2, t2_expected));
        assert!(transform_epsilon_eq(EPSILON, t3, t3_expected));
        assert!(transform_epsilon_eq(EPSILON, t4, t4_expected));
    }
}
