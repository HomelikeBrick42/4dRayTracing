use bytemuck::NoUninit;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Vector2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vector2<T> {
    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> Vector2<U> {
        Vector2 {
            x: f(self.x),
            y: f(self.y),
        }
    }
}

unsafe impl<T: NoUninit> NoUninit for Vector2<T> {}

impl<T> Add<T> for Vector2<T>
where
    T: Add<T, Output = T> + Clone,
{
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        Self {
            x: self.x + rhs.clone(),
            y: self.y + rhs,
        }
    }
}

impl<T> AddAssign<T> for Vector2<T>
where
    T: AddAssign<T> + Clone,
{
    fn add_assign(&mut self, rhs: T) {
        self.x += rhs.clone();
        self.y += rhs;
    }
}

impl<T> Add<Self> for Vector2<T>
where
    T: Add<T, Output = T> + Clone,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> AddAssign<Self> for Vector2<T>
where
    T: AddAssign<T> + Clone,
{
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<T> Sub<T> for Vector2<T>
where
    T: Sub<T, Output = T> + Clone,
{
    type Output = Self;

    fn sub(self, rhs: T) -> Self::Output {
        Self {
            x: self.x - rhs.clone(),
            y: self.y - rhs,
        }
    }
}

impl<T> SubAssign<T> for Vector2<T>
where
    T: SubAssign<T> + Clone,
{
    fn sub_assign(&mut self, rhs: T) {
        self.x -= rhs.clone();
        self.y -= rhs;
    }
}

impl<T> Sub<Self> for Vector2<T>
where
    T: Sub<T, Output = T> + Clone,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T> SubAssign<Self> for Vector2<T>
where
    T: SubAssign<T> + Clone,
{
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl<T> Mul<T> for Vector2<T>
where
    T: Mul<T, Output = T> + Clone,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self {
            x: self.x * rhs.clone(),
            y: self.y * rhs,
        }
    }
}

impl<T> MulAssign<T> for Vector2<T>
where
    T: MulAssign<T> + Clone,
{
    fn mul_assign(&mut self, rhs: T) {
        self.x *= rhs.clone();
        self.y *= rhs;
    }
}

impl<T> Mul<Self> for Vector2<T>
where
    T: Mul<T, Output = T> + Clone,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl<T> MulAssign<Self> for Vector2<T>
where
    T: MulAssign<T> + Clone,
{
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl<T> Div<T> for Vector2<T>
where
    T: Div<T, Output = T> + Clone,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self {
            x: self.x / rhs.clone(),
            y: self.y / rhs,
        }
    }
}

impl<T> DivAssign<T> for Vector2<T>
where
    T: DivAssign<T> + Clone,
{
    fn div_assign(&mut self, rhs: T) {
        self.x /= rhs.clone();
        self.y /= rhs;
    }
}

impl<T> Div<Self> for Vector2<T>
where
    T: Div<T, Output = T> + Clone,
{
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl<T> DivAssign<Self> for Vector2<T>
where
    T: DivAssign<T> + Clone,
{
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Vector3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Vector3<T> {
    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> Vector3<U> {
        Vector3 {
            x: f(self.x),
            y: f(self.y),
            z: f(self.z),
        }
    }
}

unsafe impl<T: NoUninit> NoUninit for Vector3<T> {}

impl<T> Add<T> for Vector3<T>
where
    T: Add<T, Output = T> + Clone,
{
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        Self {
            x: self.x + rhs.clone(),
            y: self.y + rhs.clone(),
            z: self.z + rhs,
        }
    }
}

impl<T> AddAssign<T> for Vector3<T>
where
    T: AddAssign<T> + Clone,
{
    fn add_assign(&mut self, rhs: T) {
        self.x += rhs.clone();
        self.y += rhs.clone();
        self.z += rhs;
    }
}

impl<T> Add<Self> for Vector3<T>
where
    T: Add<T, Output = T> + Clone,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<T> AddAssign<Self> for Vector3<T>
where
    T: AddAssign<T> + Clone,
{
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl<T> Sub<T> for Vector3<T>
where
    T: Sub<T, Output = T> + Clone,
{
    type Output = Self;

    fn sub(self, rhs: T) -> Self::Output {
        Self {
            x: self.x - rhs.clone(),
            y: self.y - rhs.clone(),
            z: self.z - rhs,
        }
    }
}

impl<T> SubAssign<T> for Vector3<T>
where
    T: SubAssign<T> + Clone,
{
    fn sub_assign(&mut self, rhs: T) {
        self.x -= rhs.clone();
        self.y -= rhs.clone();
        self.z -= rhs;
    }
}

impl<T> Sub<Self> for Vector3<T>
where
    T: Sub<T, Output = T> + Clone,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl<T> SubAssign<Self> for Vector3<T>
where
    T: SubAssign<T> + Clone,
{
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl<T> Mul<T> for Vector3<T>
where
    T: Mul<T, Output = T> + Clone,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self {
            x: self.x * rhs.clone(),
            y: self.y * rhs.clone(),
            z: self.z * rhs,
        }
    }
}

impl<T> MulAssign<T> for Vector3<T>
where
    T: MulAssign<T> + Clone,
{
    fn mul_assign(&mut self, rhs: T) {
        self.x *= rhs.clone();
        self.y *= rhs.clone();
        self.z *= rhs;
    }
}

impl<T> Mul<Self> for Vector3<T>
where
    T: Mul<T, Output = T> + Clone,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl<T> MulAssign<Self> for Vector3<T>
where
    T: MulAssign<T> + Clone,
{
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl<T> Div<T> for Vector3<T>
where
    T: Div<T, Output = T> + Clone,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self {
            x: self.x / rhs.clone(),
            y: self.y / rhs.clone(),
            z: self.z / rhs,
        }
    }
}

impl<T> DivAssign<T> for Vector3<T>
where
    T: DivAssign<T> + Clone,
{
    fn div_assign(&mut self, rhs: T) {
        self.x /= rhs.clone();
        self.y /= rhs.clone();
        self.z /= rhs;
    }
}

impl<T> Div<Self> for Vector3<T>
where
    T: Div<T, Output = T> + Clone,
{
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

impl<T> DivAssign<Self> for Vector3<T>
where
    T: DivAssign<T> + Clone,
{
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Vector4<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

impl<T> Vector4<T> {
    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> Vector4<U> {
        Vector4 {
            x: f(self.x),
            y: f(self.y),
            z: f(self.z),
            w: f(self.w),
        }
    }
}

unsafe impl<T: NoUninit> NoUninit for Vector4<T> {}

impl<T> Add<T> for Vector4<T>
where
    T: Add<T, Output = T> + Clone,
{
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        Self {
            x: self.x + rhs.clone(),
            y: self.y + rhs.clone(),
            z: self.z + rhs.clone(),
            w: self.w + rhs,
        }
    }
}

impl<T> AddAssign<T> for Vector4<T>
where
    T: AddAssign<T> + Clone,
{
    fn add_assign(&mut self, rhs: T) {
        self.x += rhs.clone();
        self.y += rhs.clone();
        self.z += rhs.clone();
        self.w += rhs;
    }
}

impl<T> Add<Self> for Vector4<T>
where
    T: Add<T, Output = T> + Clone,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w,
        }
    }
}

impl<T> AddAssign<Self> for Vector4<T>
where
    T: AddAssign<T> + Clone,
{
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self.w += rhs.w;
    }
}

impl<T> Sub<T> for Vector4<T>
where
    T: Sub<T, Output = T> + Clone,
{
    type Output = Self;

    fn sub(self, rhs: T) -> Self::Output {
        Self {
            x: self.x - rhs.clone(),
            y: self.y - rhs.clone(),
            z: self.z - rhs.clone(),
            w: self.w - rhs,
        }
    }
}

impl<T> SubAssign<T> for Vector4<T>
where
    T: SubAssign<T> + Clone,
{
    fn sub_assign(&mut self, rhs: T) {
        self.x -= rhs.clone();
        self.y -= rhs.clone();
        self.z -= rhs.clone();
        self.w -= rhs;
    }
}

impl<T> Sub<Self> for Vector4<T>
where
    T: Sub<T, Output = T> + Clone,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            w: self.w - rhs.w,
        }
    }
}

impl<T> SubAssign<Self> for Vector4<T>
where
    T: SubAssign<T> + Clone,
{
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
        self.w -= rhs.w;
    }
}

impl<T> Mul<T> for Vector4<T>
where
    T: Mul<T, Output = T> + Clone,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self {
            x: self.x * rhs.clone(),
            y: self.y * rhs.clone(),
            z: self.z * rhs.clone(),
            w: self.w * rhs,
        }
    }
}

impl<T> MulAssign<T> for Vector4<T>
where
    T: MulAssign<T> + Clone,
{
    fn mul_assign(&mut self, rhs: T) {
        self.x *= rhs.clone();
        self.y *= rhs.clone();
        self.z *= rhs.clone();
        self.w *= rhs;
    }
}

impl<T> Mul<Self> for Vector4<T>
where
    T: Mul<T, Output = T> + Clone,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
            w: self.w * rhs.w,
        }
    }
}

impl<T> MulAssign<Self> for Vector4<T>
where
    T: MulAssign<T> + Clone,
{
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
        self.w *= rhs.w;
    }
}

impl<T> Div<T> for Vector4<T>
where
    T: Div<T, Output = T> + Clone,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self {
            x: self.x / rhs.clone(),
            y: self.y / rhs.clone(),
            z: self.z / rhs.clone(),
            w: self.w / rhs,
        }
    }
}

impl<T> DivAssign<T> for Vector4<T>
where
    T: DivAssign<T> + Clone,
{
    fn div_assign(&mut self, rhs: T) {
        self.x /= rhs.clone();
        self.y /= rhs.clone();
        self.z /= rhs.clone();
        self.w /= rhs;
    }
}

impl<T> Div<Self> for Vector4<T>
where
    T: Div<T, Output = T> + Clone,
{
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
            w: self.w / rhs.w,
        }
    }
}

impl<T> DivAssign<Self> for Vector4<T>
where
    T: DivAssign<T> + Clone,
{
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
        self.w /= rhs.w;
    }
}
