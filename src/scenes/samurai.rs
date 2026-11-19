//! A lone samurai under a huge moon, both hands on a katana planted in the
//! ground. He breathes slowly; his sleeves and scarf move in the wind.

use super::{mist, petals, Scene};
use crate::canvas::Pixels;
use crate::theme::Theme;
use crate::util::{hash, noise, rgb};

pub struct Samurai;

impl Scene for Samurai {
    fn name(&self) -> &'static str {
        "samurai"
    }

    fn blurb(&self) -> &'static str {
        "A ronin breathing slowly under the moon, katana planted before him"
    }

    fn draw(&self, p: &mut Pixels, t: f32, _theme: Option<&Theme>) {
        let (w, h) = (p.w as f32, p.h as f32);
        let s = h / 80.0;
        let cx = (w * 0.5).floor() + 0.5;
        let gy = h * 0.87;

        p.gradient(&[
            (0.0, rgb(10.0, 8.0, 26.0)),
            (0.55, rgb(52.0, 20.0, 50.0)),
            (0.87, rgb(150.0, 52.0, 56.0)),
            (1.0, rgb(24.0, 10.0, 24.0)),
        ]);

        for i in 0..70u32 {
            let x = hash(i * 3 + 1) * w;
            let y = hash(i * 3 + 2) * h * 0.55;
            let tw = 0.5 + 0.5 * (t * (0.8 + hash(i * 3 + 3) * 2.0) + i as f32).sin();
            p.blend(x as i32, y as i32, rgb(255.0, 245.0, 235.0), 0.2 + 0.5 * tw);
        }

        // huge moon, glowing slowly in time with the breathing
        let breath = (t * 1.2).sin();
        let (mx, my, mr) = (cx, h * 0.40, 21.0 * s);
        p.glow(
            mx,
            my,
            mr * 2.6,
            rgb(255.0, 170.0, 130.0),
            0.42 + 0.06 * breath,
        );
        p.circle(mx, my, mr, rgb(255.0, 236.0, 206.0), 1.0);
        for (dx, dy, r, a) in [
            (-6.0, -5.0, 4.5, 0.45),
            (7.0, 4.0, 3.2, 0.4),
            (-2.0, 9.0, 2.6, 0.35),
            (8.0, -8.0, 2.0, 0.3),
        ] {
            p.circle(mx + dx * s, my + dy * s, r * s, rgb(238.0, 210.0, 180.0), a);
        }

        // distant bamboo
        for i in 0..14u32 {
            let x = w * (hash(i * 9 + 4) * 0.34) + if i % 2 == 0 { 0.0 } else { w * 0.66 };
            let top = h * (0.18 + 0.3 * hash(i * 9 + 5));
            let sway = (t * 0.9 + i as f32).sin() * 2.2 * s;
            p.line(x, gy, x + sway, top, 1.3 * s, rgb(30.0, 12.0, 30.0), 0.85);
            for j in 0..5u32 {
                let ny = top + (gy - top) * (j as f32 + 0.5) / 5.0;
                let nx = x + sway * (1.0 - (ny - top) / (gy - top));
                p.rect(
                    nx - 0.9 * s,
                    ny,
                    nx + 0.9 * s,
                    ny + 0.35 * s,
                    rgb(70.0, 28.0, 50.0),
                    0.7,
                );
            }
        }

        mist(p, t, 0.70, 0.90, rgb(236.0, 120.0, 110.0), 0.42);

        // ground
        p.rect(0.0, gy, w, h, rgb(10.0, 6.0, 16.0), 1.0);

        // ---- the samurai ----
        let sil = rgb(8.0, 6.0, 14.0);
        let b = 0.5 + 0.5 * (t * 1.2).sin(); // 0..1, slow breath
        let rise = b * 0.7 * s; // chest rises
        let expand = 1.0 + 0.045 * b; // chest widens

        let head_y = gy - 38.0 * s;
        let sh_y = head_y + 4.2 * s - rise; // shoulder line
        let waist_y = head_y + 18.0 * s;
        let hem_y = gy;
        let flutter =
            |k: f32, a: f32| (t * 2.6 + k).sin() * a * s + (t * 5.1 + k * 2.0).sin() * a * 0.3 * s;

        // hakama (wide trousers)
        p.poly(
            &[
                (cx - 5.6 * s, waist_y),
                (cx + 5.6 * s, waist_y),
                (cx + 10.0 * s + flutter(0.0, 0.35), hem_y),
                (cx + 3.0 * s, hem_y - 0.8 * s),
                (cx - 3.0 * s, hem_y - 0.8 * s),
                (cx - 10.0 * s + flutter(1.0, 0.35), hem_y),
            ],
            sil,
            1.0,
        );
        // torso (kimono)
        p.poly(
            &[
                (cx - 7.2 * s * expand, sh_y),
                (cx + 7.2 * s * expand, sh_y),
                (cx + 5.6 * s, waist_y + 0.5 * s),
                (cx - 5.6 * s, waist_y + 0.5 * s),
            ],
            sil,
            1.0,
        );
        // obi (belt), the one touch of colour
        p.rect(
            cx - 5.7 * s,
            waist_y - 1.2 * s,
            cx + 5.7 * s,
            waist_y + 1.1 * s,
            rgb(126.0, 26.0, 40.0),
            1.0,
        );
        // neck and straw hat
        p.rect(
            cx - 1.5 * s,
            sh_y - 1.6 * s,
            cx + 1.5 * s,
            sh_y + 0.6 * s,
            sil,
            1.0,
        );
        p.circle(cx, head_y + 2.0 * s - rise * 0.6, 3.3 * s, sil, 1.0);
        let hat_y = head_y + 0.2 * s - rise * 0.6;
        p.poly(
            &[
                (cx - 10.5 * s, hat_y + 1.2 * s),
                (cx + 10.5 * s, hat_y + 1.2 * s),
                (cx, hat_y - 8.0 * s),
            ],
            sil,
            1.0,
        );
        p.glow(cx, hat_y - 1.0 * s, 9.0 * s, rgb(255.0, 190.0, 150.0), 0.07);

        // sword: planted point-down in front, hands on the hilt
        let hand_y = sh_y + 9.5 * s;
        let blade_top = hand_y + 2.4 * s;
        p.line(
            cx,
            hand_y - 5.5 * s,
            cx,
            blade_top,
            1.5 * s,
            rgb(60.0, 24.0, 36.0),
            1.0,
        ); // hilt
        for k in 0..4 {
            let yy = hand_y - 4.6 * s + k as f32 * 1.3 * s;
            p.rect(
                cx - 0.9 * s,
                yy,
                cx + 0.9 * s,
                yy + 0.4 * s,
                rgb(150.0, 120.0, 90.0),
                0.8,
            );
        }
        p.ellipse(
            cx,
            blade_top,
            2.6 * s,
            0.7 * s,
            rgb(170.0, 140.0, 80.0),
            1.0,
        ); // tsuba (guard)
        p.line(
            cx,
            blade_top,
            cx,
            gy + 1.0 * s,
            0.9 * s,
            rgb(196.0, 206.0, 222.0),
            1.0,
        ); // blade
        p.line(
            cx - 0.25 * s,
            blade_top,
            cx - 0.25 * s,
            gy,
            0.3 * s,
            rgb(255.0, 255.0, 255.0),
            0.55,
        );
        let glint = blade_top + (gy - blade_top) * ((t * 0.22) % 1.0);
        p.glow(cx, glint, 3.0 * s, rgb(210.0, 230.0, 255.0), 0.9);

        // arms and wide sleeves
        for side in [-1.0f32, 1.0] {
            let shoulder = (cx + side * 6.6 * s * expand, sh_y + 0.6 * s);
            let hand = (cx + side * 0.9 * s, hand_y);
            p.line(shoulder.0, shoulder.1, hand.0, hand.1, 3.0 * s, sil, 1.0);
            let sway = flutter(side * 2.0, 0.5);
            p.poly(
                &[
                    (shoulder.0 - side * 0.6 * s, shoulder.1 + 3.0 * s),
                    (shoulder.0 + side * 2.2 * s, shoulder.1 + 3.0 * s),
                    (shoulder.0 + side * 3.2 * s + sway, shoulder.1 + 12.0 * s),
                    (shoulder.0 - side * 0.8 * s + sway, shoulder.1 + 12.5 * s),
                ],
                sil,
                1.0,
            );
        }

        // scarf tail streaming in the wind
        let sx = cx + 1.8 * s;
        let sy = sh_y - 0.4 * s;
        for j in 0..22 {
            let jf = j as f32;
            let x = sx + jf * 0.9 * s;
            let y = sy + (t * 4.0 - jf * 0.45).sin() * (0.4 + jf * 0.07) * s + jf * 0.08 * s;
            p.rect(
                x,
                y,
                x + 1.1 * s,
                y + (1.8 - jf * 0.06).max(0.6) * s,
                rgb(126.0, 26.0, 40.0),
                1.0 - jf * 0.02,
            );
        }

        // low grass in front
        let mut x = 0.0f32;
        while x < w {
            let hh = (1.5 + 3.5 * noise(x * 0.3)) * s;
            let sway = ((t * 2.0 + x * 0.14).sin() + 0.3 * (t * 4.0 + x * 0.3).sin()) * 1.1 * s;
            p.line(
                x,
                gy + 0.8 * s,
                x + sway,
                gy - hh,
                0.8 * s,
                rgb(6.0, 4.0, 12.0),
                1.0,
            );
            x += 1.5 * s;
        }

        petals(p, t, 42, 31);
    }
}
