use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use rand::Rng;
use std::f32::consts::PI;

const GRAVITY: f32 = 0.0002;
const BOUNCE: f32 = 0.85;
const EXPLOSION_FORCE: f32 = 8.0;

#[wasm_bindgen]
pub struct ParticleSystemCanvas2D {
    particles: Vec<Particle>,
    ctx: CanvasRenderingContext2d,
    width: f32,
    height: f32,
    frame_count: u32,
    particle_count: usize,
}

struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    hue: f32,
}

#[wasm_bindgen]
impl ParticleSystemCanvas2D {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str, particle_count: usize) -> Result<ParticleSystemCanvas2D, JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document
            .get_element_by_id(canvas_id)
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()?;

        let width = canvas.width() as f32;
        let height = canvas.height() as f32;

        let ctx = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;

        // パーティクルを生成
        let particles = create_particles(width, height, particle_count);

        Ok(ParticleSystemCanvas2D {
            particles,
            ctx,
            width,
            height,
            frame_count: 0,
            particle_count,
        })
    }

    pub fn update(&mut self) {
        // Rustで高速物理演算!
        for p in &mut self.particles {
            // 重力
            p.vy += GRAVITY;

            // 位置更新
            p.x += p.vx;
            p.y += p.vy;

            // 壁で跳ね返る
            if p.x < 0.0 || p.x > self.width {
                p.vx *= -BOUNCE;
                p.x = p.x.clamp(0.0, self.width);
            }

            if p.y < 0.0 {
                p.vy *= -BOUNCE;
                p.y = 0.0;
            }

            if p.y > self.height {
                p.vy *= -BOUNCE;
                p.y = self.height;
                p.vx *= 0.98; // 摩擦
            }

            // 色を変化
            p.hue = (p.hue + 0.3) % 360.0;
        }

        self.frame_count += 1;
    }

    pub fn render(&self) {
        let ctx = &self.ctx;

        // 画面クリア
        ctx.set_fill_style(&JsValue::from_str("rgba(17, 17, 17, 1)"));
        ctx.fill_rect(0.0, 0.0, self.width as f64, self.height as f64);

        // 各パーティクルを描画（Canvas 2D APIで1個ずつ！）
        for p in &self.particles {
            let rgb = hsl_to_rgb(p.hue, 1.0, 0.5);
            let color = format!(
                "rgba({}, {}, {}, 0.8)",
                (rgb.0 * 255.0) as u8,
                (rgb.1 * 255.0) as u8,
                (rgb.2 * 255.0) as u8
            );

            ctx.set_fill_style(&JsValue::from_str(&color));
            ctx.begin_path();
            let _ = ctx.arc(p.x as f64, p.y as f64, 2.5, 0.0, 2.0 * PI as f64);
            ctx.fill();
        }
    }

    pub fn get_frame_count(&self) -> u32 {
        self.frame_count
    }

    pub fn reset(&mut self) {
        self.particles = create_particles(self.width, self.height, self.particle_count);
        self.frame_count = 0;
    }

    // クリックで爆発!
    pub fn explode(&mut self, click_x: f32, click_y: f32) {
        for p in &mut self.particles {
            let dx = p.x - click_x;
            let dy = p.y - click_y;
            let dist = (dx * dx + dy * dy).sqrt();

            // 近いパーティクルほど強く吹き飛ぶ
            if dist < 200.0 {
                let force = EXPLOSION_FORCE * (1.0 - dist / 200.0);
                let angle = dy.atan2(dx);
                p.vx += angle.cos() * force;
                p.vy += angle.sin() * force;
            }
        }
    }
}

// パーティクル生成
fn create_particles(width: f32, height: f32, particle_count: usize) -> Vec<Particle> {
    let mut rng = rand::thread_rng();
    (0..particle_count)
        .map(|_| {
            let angle = rng.gen::<f32>() * 2.0 * PI;
            let speed = rng.gen::<f32>() * 2.0 + 1.0;
            Particle {
                x: width / 2.0,
                y: height / 4.0,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed - 3.0,
                hue: rng.gen::<f32>() * 360.0,
            }
        })
        .collect()
}

// HSL to RGB変換
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());

    let (r1, g1, b1) = if h_prime < 1.0 {
        (c, x, 0.0)
    } else if h_prime < 2.0 {
        (x, c, 0.0)
    } else if h_prime < 3.0 {
        (0.0, c, x)
    } else if h_prime < 4.0 {
        (0.0, x, c)
    } else if h_prime < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    let m = l - c / 2.0;
    (r1 + m, g1 + m, b1 + m)
}
