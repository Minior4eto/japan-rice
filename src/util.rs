//! Small helpers: colours, deterministic hashing and noise.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

pub const fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color { r, g, b }
}

impl Color {
    pub fn mix(self, o: Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color {
            r: self.r + (o.r - self.r) * t,
            g: self.g + (o.g - self.g) * t,
            b: self.b + (o.b - self.b) * t,
        }
    }

    pub fn scale(self, k: f32) -> Color {
        Color {
            r: self.r * k,
            g: self.g * k,
            b: self.b * k,
        }
    }

    pub fn luma(self) -> f32 {
        0.299 * self.r + 0.587 * self.g + 0.114 * self.b
    }

    pub fn to_u8(self) -> (u8, u8, u8) {
        (
            self.r.clamp(0.0, 255.0) as u8,
            self.g.clamp(0.0, 255.0) as u8,
            self.b.clamp(0.0, 255.0) as u8,
        )
    }
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Deterministic pseudo-random number in 0..1 from an integer.
pub fn hash(n: u32) -> f32 {
    let mut n = n.wrapping_mul(0x9E37_79B1);
    n ^= n >> 15;
    n = n.wrapping_mul(0x85EB_CA77);
    n ^= n >> 13;
    n = n.wrapping_mul(0xC2B2_AE3D);
    n ^= n >> 16;
    (n & 0x00FF_FFFF) as f32 / 16_777_215.0
}

/// Smooth 1D value noise in 0..1.
pub fn noise(x: f32) -> f32 {
    let i = x.floor();
    let f = x - i;
    let u = f * f * (3.0 - 2.0 * f);
    let a = hash(i as i32 as u32);
    let b = hash((i as i32 + 1) as u32);
    lerp(a, b, u)
}

/// Two octaves of noise in 0..1.
pub fn fbm(x: f32) -> f32 {
    0.65 * noise(x) + 0.35 * noise(x * 2.3 + 17.0)
}

pub fn rotate(p: (f32, f32), c: (f32, f32), ang: f32) -> (f32, f32) {
    let (s, co) = ang.sin_cos();
    let (dx, dy) = (p.0 - c.0, p.1 - c.1);
    (c.0 + dx * co - dy * s, c.1 + dx * s + dy * co)
}

/// Smooth 2D value noise in 0..1.
pub fn noise2(x: f32, y: f32) -> f32 {
    let (ix, iy) = (x.floor(), y.floor());
    let (fx, fy) = (x - ix, y - iy);
    let (u, v) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let h = |a: f32, b: f32| {
        hash(
            (a as i32 as u32).wrapping_mul(73_856_093) ^ (b as i32 as u32).wrapping_mul(19_349_663),
        )
    };
    lerp(
        lerp(h(ix, iy), h(ix + 1.0, iy), u),
        lerp(h(ix, iy + 1.0), h(ix + 1.0, iy + 1.0), u),
        v,
    )
}
