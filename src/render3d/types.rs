//! Core 3D types for GPU rendering
//!
//! This module provides fundamental 3D types optimized for GPU rendering,
//! including vectors, matrices, and transforms.

use std::ops::{Add, Mul, Neg, Sub};

/// 3D vector type
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    /// Create a new Vec3
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Zero vector
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    /// Unit X vector
    pub const X: Self = Self::new(1.0, 0.0, 0.0);

    /// Unit Y vector
    pub const Y: Self = Self::new(0.0, 1.0, 0.0);

    /// Unit Z vector
    pub const Z: Self = Self::new(0.0, 0.0, 1.0);

    /// Create a vector with all components equal to the given value
    pub const fn splat(v: f32) -> Self {
        Self::new(v, v, v)
    }

    /// Compute the dot product with another vector
    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Compute the cross product with another vector
    pub fn cross(&self, other: &Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    /// Compute the length (magnitude) of the vector
    pub fn length(&self) -> f32 {
        self.dot(self).sqrt()
    }

    /// Compute the squared length of the vector
    pub fn length_squared(&self) -> f32 {
        self.dot(self)
    }

    /// Normalize the vector to unit length
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 1e-10 {
            Self::new(self.x / len, self.y / len, self.z / len)
        } else {
            Self::ZERO
        }
    }

    /// Linearly interpolate between two vectors
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
            self.z + (other.z - self.z) * t,
        )
    }

    /// Convert to array
    pub fn to_array(&self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }

    /// Create from array
    pub fn from_array(arr: [f32; 3]) -> Self {
        Self::new(arr[0], arr[1], arr[2])
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

/// 4D vector type (for homogeneous coordinates)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    /// Create a new Vec4
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// Zero vector
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    /// Create from Vec3 with w component
    pub fn from_vec3(v: Vec3, w: f32) -> Self {
        Self::new(v.x, v.y, v.z, w)
    }

    /// Convert to array
    pub fn to_array(&self) -> [f32; 4] {
        [self.x, self.y, self.z, self.w]
    }
}

/// 4x4 matrix in column-major order
///
/// Memory layout matches GPU expectations:
/// ```text
/// | m[0]  m[4]  m[8]   m[12] |   | col0.x col1.x col2.x col3.x |
/// | m[1]  m[5]  m[9]   m[13] | = | col0.y col1.y col2.y col3.y |
/// | m[2]  m[6]  m[10]  m[14] |   | col0.z col1.z col2.z col3.z |
/// | m[3]  m[7]  m[11]  m[15] |   | col0.w col1.w col2.w col3.w |
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat4 {
    /// Column-major data: [col0, col1, col2, col3]
    pub v: [f32; 16],
}

impl Default for Mat4 {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Mat4 {
    /// Identity matrix
    pub const IDENTITY: Self = Self {
        v: [
            1.0, 0.0, 0.0, 0.0, // column 0
            0.0, 1.0, 0.0, 0.0, // column 1
            0.0, 0.0, 1.0, 0.0, // column 2
            0.0, 0.0, 0.0, 1.0, // column 3
        ],
    };

    /// Create a new matrix from column-major data
    pub const fn from_cols_array(v: [f32; 16]) -> Self {
        Self { v }
    }

    /// Create a matrix from four column vectors
    pub fn from_cols(c0: Vec4, c1: Vec4, c2: Vec4, c3: Vec4) -> Self {
        Self {
            v: [
                c0.x, c0.y, c0.z, c0.w, c1.x, c1.y, c1.z, c1.w, c2.x, c2.y, c2.z, c2.w, c3.x, c3.y,
                c3.z, c3.w,
            ],
        }
    }

    /// Get column as Vec4
    pub fn col(&self, i: usize) -> Vec4 {
        let base = i * 4;
        Vec4::new(
            self.v[base],
            self.v[base + 1],
            self.v[base + 2],
            self.v[base + 3],
        )
    }

    /// Create a translation matrix
    pub fn from_translation(t: Vec3) -> Self {
        Self {
            v: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, t.x, t.y, t.z, 1.0,
            ],
        }
    }

    /// Create a uniform scale matrix
    pub fn from_scale(s: Vec3) -> Self {
        Self {
            v: [
                s.x, 0.0, 0.0, 0.0, 0.0, s.y, 0.0, 0.0, 0.0, 0.0, s.z, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    /// Create a rotation matrix around the X axis
    pub fn from_rotation_x(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self {
            v: [
                1.0, 0.0, 0.0, 0.0, 0.0, c, s, 0.0, 0.0, -s, c, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    /// Create a rotation matrix around the Y axis
    pub fn from_rotation_y(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self {
            v: [
                c, 0.0, -s, 0.0, 0.0, 1.0, 0.0, 0.0, s, 0.0, c, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    /// Create a rotation matrix around the Z axis
    pub fn from_rotation_z(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self {
            v: [
                c, s, 0.0, 0.0, -s, c, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    /// Create a rotation matrix from Euler angles (YXZ order)
    pub fn from_euler_yxz(yaw: f32, pitch: f32, roll: f32) -> Self {
        Self::from_rotation_y(yaw) * Self::from_rotation_x(pitch) * Self::from_rotation_z(roll)
    }

    /// Create a look-at view matrix (right-handed)
    pub fn look_at_rh(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        let f = (target - eye).normalize();
        let r = f.cross(&up).normalize();
        let u = r.cross(&f);

        Self {
            v: [
                r.x,
                u.x,
                -f.x,
                0.0,
                r.y,
                u.y,
                -f.y,
                0.0,
                r.z,
                u.z,
                -f.z,
                0.0,
                -r.dot(&eye),
                -u.dot(&eye),
                f.dot(&eye),
                1.0,
            ],
        }
    }

    /// Create a perspective projection matrix (right-handed, depth 0 to 1)
    pub fn perspective_rh(fov_y: f32, aspect: f32, near: f32, far: f32) -> Self {
        let f = 1.0 / (fov_y / 2.0).tan();
        let nf = 1.0 / (near - far);

        Self {
            v: [
                f / aspect,
                0.0,
                0.0,
                0.0,
                0.0,
                f,
                0.0,
                0.0,
                0.0,
                0.0,
                far * nf,
                -1.0,
                0.0,
                0.0,
                near * far * nf,
                0.0,
            ],
        }
    }

    /// Create an orthographic projection matrix
    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        let rml = right - left;
        let tmb = top - bottom;
        let fmn = far - near;

        Self {
            v: [
                2.0 / rml,
                0.0,
                0.0,
                0.0,
                0.0,
                2.0 / tmb,
                0.0,
                0.0,
                0.0,
                0.0,
                -1.0 / fmn,
                0.0,
                -(right + left) / rml,
                -(top + bottom) / tmb,
                -near / fmn,
                1.0,
            ],
        }
    }

    /// Transform a Vec3 as a point (w=1)
    pub fn transform_point(&self, p: Vec3) -> Vec3 {
        let x = self.v[0] * p.x + self.v[4] * p.y + self.v[8] * p.z + self.v[12];
        let y = self.v[1] * p.x + self.v[5] * p.y + self.v[9] * p.z + self.v[13];
        let z = self.v[2] * p.x + self.v[6] * p.y + self.v[10] * p.z + self.v[14];
        let w = self.v[3] * p.x + self.v[7] * p.y + self.v[11] * p.z + self.v[15];

        if w.abs() > 1e-10 {
            Vec3::new(x / w, y / w, z / w)
        } else {
            Vec3::new(x, y, z)
        }
    }

    /// Transform a Vec3 as a direction (w=0)
    pub fn transform_vector(&self, v: Vec3) -> Vec3 {
        Vec3::new(
            self.v[0] * v.x + self.v[4] * v.y + self.v[8] * v.z,
            self.v[1] * v.x + self.v[5] * v.y + self.v[9] * v.z,
            self.v[2] * v.x + self.v[6] * v.y + self.v[10] * v.z,
        )
    }

    /// Get the translation component
    pub fn translation(&self) -> Vec3 {
        Vec3::new(self.v[12], self.v[13], self.v[14])
    }
}

impl Mul for Mat4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let mut result = [0.0f32; 16];

        for col in 0..4 {
            for row in 0..4 {
                result[col * 4 + row] = self.v[row] * rhs.v[col * 4]
                    + self.v[4 + row] * rhs.v[col * 4 + 1]
                    + self.v[8 + row] * rhs.v[col * 4 + 2]
                    + self.v[12 + row] * rhs.v[col * 4 + 3];
            }
        }

        Self { v: result }
    }
}

/// GPU-friendly transform representation using 4 column vectors (64 bytes)
///
/// This format is optimized for shader upload - instead of re-uploading entire
/// mesh geometry each frame, only this 64-byte transform needs to be updated.
///
/// # Memory Layout
///
/// ```text
/// col0: [m00, m10, m20, m30]  // First column (X axis + scale)
/// col1: [m01, m11, m21, m31]  // Second column (Y axis + scale)
/// col2: [m02, m12, m22, m32]  // Third column (Z axis + scale)
/// col3: [m03, m13, m23, m33]  // Fourth column (translation)
/// ```
///
/// # Example
///
/// ```rust
/// use makepad_d3::render3d::types::{Transform3D, Vec3, Mat4};
///
/// // Create transform from translation and rotation
/// let translation = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
/// let rotation = Mat4::from_rotation_y(0.5);
/// let combined = translation * rotation;
///
/// let transform = Transform3D::from_mat4(&combined);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct Transform3D {
    /// First column of the 4x4 matrix
    pub col0: Vec4,
    /// Second column of the 4x4 matrix
    pub col1: Vec4,
    /// Third column of the 4x4 matrix
    pub col2: Vec4,
    /// Fourth column of the 4x4 matrix (translation)
    pub col3: Vec4,
}

impl Transform3D {
    /// Identity transform
    pub const IDENTITY: Self = Self {
        col0: Vec4::new(1.0, 0.0, 0.0, 0.0),
        col1: Vec4::new(0.0, 1.0, 0.0, 0.0),
        col2: Vec4::new(0.0, 0.0, 1.0, 0.0),
        col3: Vec4::new(0.0, 0.0, 0.0, 1.0),
    };

    /// Create a new transform from four column vectors
    pub const fn new(col0: Vec4, col1: Vec4, col2: Vec4, col3: Vec4) -> Self {
        Self {
            col0,
            col1,
            col2,
            col3,
        }
    }

    /// Create a transform from a Mat4
    pub fn from_mat4(m: &Mat4) -> Self {
        Self {
            col0: m.col(0),
            col1: m.col(1),
            col2: m.col(2),
            col3: m.col(3),
        }
    }

    /// Convert back to Mat4
    pub fn to_mat4(&self) -> Mat4 {
        Mat4::from_cols(self.col0, self.col1, self.col2, self.col3)
    }

    /// Create a translation transform
    pub fn from_translation(t: Vec3) -> Self {
        Self::from_mat4(&Mat4::from_translation(t))
    }

    /// Create a scale transform
    pub fn from_scale(s: Vec3) -> Self {
        Self::from_mat4(&Mat4::from_scale(s))
    }

    /// Create a uniform scale transform
    pub fn from_uniform_scale(s: f32) -> Self {
        Self::from_scale(Vec3::splat(s))
    }

    /// Create a rotation transform around X axis
    pub fn from_rotation_x(angle: f32) -> Self {
        Self::from_mat4(&Mat4::from_rotation_x(angle))
    }

    /// Create a rotation transform around Y axis
    pub fn from_rotation_y(angle: f32) -> Self {
        Self::from_mat4(&Mat4::from_rotation_y(angle))
    }

    /// Create a rotation transform around Z axis
    pub fn from_rotation_z(angle: f32) -> Self {
        Self::from_mat4(&Mat4::from_rotation_z(angle))
    }

    /// Create a rotation transform from Euler angles (YXZ order)
    pub fn from_euler_yxz(yaw: f32, pitch: f32, roll: f32) -> Self {
        Self::from_mat4(&Mat4::from_euler_yxz(yaw, pitch, roll))
    }

    /// Compose this transform with another: self * other
    ///
    /// This applies `other` first, then `self`.
    pub fn then(&self, other: &Self) -> Self {
        Self::from_mat4(&(self.to_mat4() * other.to_mat4()))
    }

    /// Get the columns as an array of Vec4 (for shader upload)
    pub fn to_columns(&self) -> [Vec4; 4] {
        [self.col0, self.col1, self.col2, self.col3]
    }

    /// Get the translation component
    pub fn translation(&self) -> Vec3 {
        Vec3::new(self.col3.x, self.col3.y, self.col3.z)
    }

    /// Transform a point
    pub fn transform_point(&self, p: Vec3) -> Vec3 {
        self.to_mat4().transform_point(p)
    }

    /// Transform a direction vector
    pub fn transform_vector(&self, v: Vec3) -> Vec3 {
        self.to_mat4().transform_vector(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_vec3_operations() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);

        assert_eq!(a + b, Vec3::new(5.0, 7.0, 9.0));
        assert_eq!(a - b, Vec3::new(-3.0, -3.0, -3.0));
        assert_eq!(a * 2.0, Vec3::new(2.0, 4.0, 6.0));
        assert_eq!(a.dot(&b), 32.0);
    }

    #[test]
    fn test_vec3_cross() {
        let x = Vec3::X;
        let y = Vec3::Y;
        let z = x.cross(&y);

        assert!((z.x - 0.0).abs() < 1e-6);
        assert!((z.y - 0.0).abs() < 1e-6);
        assert!((z.z - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_vec3_normalize() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        let n = v.normalize();

        assert!((n.length() - 1.0).abs() < 1e-6);
        assert!((n.x - 0.6).abs() < 1e-6);
        assert!((n.y - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_mat4_identity() {
        let m = Mat4::IDENTITY;
        let p = Vec3::new(1.0, 2.0, 3.0);
        let result = m.transform_point(p);

        assert_eq!(result, p);
    }

    #[test]
    fn test_mat4_translation() {
        let t = Vec3::new(10.0, 20.0, 30.0);
        let m = Mat4::from_translation(t);
        let p = Vec3::new(1.0, 2.0, 3.0);
        let result = m.transform_point(p);

        assert_eq!(result, Vec3::new(11.0, 22.0, 33.0));
    }

    #[test]
    fn test_mat4_scale() {
        let s = Vec3::new(2.0, 3.0, 4.0);
        let m = Mat4::from_scale(s);
        let p = Vec3::new(1.0, 2.0, 3.0);
        let result = m.transform_point(p);

        assert_eq!(result, Vec3::new(2.0, 6.0, 12.0));
    }

    #[test]
    fn test_mat4_rotation_x() {
        let m = Mat4::from_rotation_x(PI / 2.0);
        let p = Vec3::Y;
        let result = m.transform_point(p);

        assert!((result.x - 0.0).abs() < 1e-6);
        assert!((result.y - 0.0).abs() < 1e-6);
        assert!((result.z - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_mat4_multiplication() {
        let t = Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0));
        let s = Mat4::from_scale(Vec3::splat(2.0));

        // Scale then translate
        let combined = t * s;
        let p = Vec3::new(1.0, 0.0, 0.0);
        let result = combined.transform_point(p);

        // 1.0 * 2.0 + 1.0 = 3.0
        assert!((result.x - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_transform3d_identity() {
        let t = Transform3D::IDENTITY;
        let p = Vec3::new(1.0, 2.0, 3.0);
        let result = t.transform_point(p);

        assert_eq!(result, p);
    }

    #[test]
    fn test_transform3d_composition() {
        let t1 = Transform3D::from_translation(Vec3::new(1.0, 0.0, 0.0));
        let t2 = Transform3D::from_scale(Vec3::splat(2.0));

        // Apply scale first, then translation
        let combined = t1.then(&t2);
        let p = Vec3::new(1.0, 0.0, 0.0);
        let result = combined.transform_point(p);

        // 1.0 * 2.0 + 1.0 = 3.0
        assert!((result.x - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_transform3d_to_columns() {
        let t = Transform3D::IDENTITY;
        let cols = t.to_columns();

        assert_eq!(cols[0], Vec4::new(1.0, 0.0, 0.0, 0.0));
        assert_eq!(cols[1], Vec4::new(0.0, 1.0, 0.0, 0.0));
        assert_eq!(cols[2], Vec4::new(0.0, 0.0, 1.0, 0.0));
        assert_eq!(cols[3], Vec4::new(0.0, 0.0, 0.0, 1.0));
    }
}
