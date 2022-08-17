use std::ops::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct V2 {
    pub x: f32,
    pub y: f32
}

#[inline(always)]
pub fn v2(x: f32, y: f32) -> V2 {
    V2 { x, y }
}

impl Index<usize> for V2 {
    type Output = f32;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("V2 index too high")
        }
    }
}

impl IndexMut<usize> for V2 {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => panic!("V2 index too high")
        }
    }
}

impl Mul<V2> for f32 {
    type Output = V2;

    #[inline(always)]
    fn mul(self, rhs: V2) -> Self::Output {
        v2(self * rhs.x, self * rhs.y)
    }
}

impl Mul<f32> for V2 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: f32) -> Self::Output {
        rhs * self
    }
}

impl MulAssign<f32> for V2 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: f32) {
        self.x = rhs * self.x;
        self.y = rhs * self.y;
    }
}

impl Neg for V2 {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self::Output {
        v2(-self.x, -self.y)
    }
}

impl Add for V2 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        v2(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign<V2> for V2 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: V2) {
        self.x = self.x + rhs.x;
        self.y = self.y + rhs.y;
    }
}

impl Sub for V2 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        v2(self.x - rhs.x, self.y - rhs.y)
    }
}

#[inline(always)]
pub fn inner(a: V2, b: V2) -> f32 {
    let result = a.x * b.x + a.y * b.y;
    result
}

#[inline(always)]
pub fn len_sq(a: V2) -> f32 {
    let result = inner(a ,a );
    result
}

pub struct Rectangle2 {
    min: V2,
    max: V2
}

#[inline(always)]
pub fn rect_min_max(min: V2, max: V2) -> Rectangle2 {
    Rectangle2 {
        min,
        max
    }
}

#[inline(always)]
pub fn rect_centre_half_dim(centre: V2, half_dim: V2) -> Rectangle2 {
    Rectangle2 {
        min: centre - half_dim,
        max: centre + half_dim,
    }
}


#[inline(always)]
pub fn rect_centre_dim(centre: V2, dim: V2) -> Rectangle2 {
    rect_centre_half_dim(centre, dim * 0.5)
}

#[inline(always)]
pub fn rect_min_dim(min: V2, dim: V2) -> Rectangle2 {
    Rectangle2 {
        min,
        max: min + dim,
    }
}

#[inline(always)]
pub fn is_in_rectangle(rectangle: &Rectangle2, test: V2) -> bool {
    test.x >= rectangle.min.x 
    && test.y >= rectangle.min.y 
    && test.x < rectangle.max.x 
    && test.y < rectangle.max.y     
}