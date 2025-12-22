use wasm_bindgen::prelude::*;
use web_sys::{WebGlRenderingContext, WebGlProgram, WebGlBuffer};
use rand::Rng;
use std::f32::consts::PI;

const GRAVITY: f32 = 0.0002;
const BOUNCE: f32 = 0.85;
const EXPLOSION_FORCE: f32 = 8.0;

#[wasm_bindgen]
pub struct ParticleSystem {
    particles: Vec<Particle>,
    gl: WebGlRenderingContext,
    program: WebGlProgram,
    position_buffer: WebGlBuffer,
    color_buffer: WebGlBuffer,
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
impl ParticleSystem {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str, particle_count: usize) -> Result<ParticleSystem, JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document
            .get_element_by_id(canvas_id)
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()?;

        let width = canvas.width() as f32;
        let height = canvas.height() as f32;

        let gl = canvas
            .get_context("webgl")?
            .unwrap()
            .dyn_into::<WebGlRenderingContext>()?;

        // シェーダーをコンパイル
        let vert_shader = compile_shader(
            &gl,
            WebGlRenderingContext::VERTEX_SHADER,
            VERTEX_SHADER_SOURCE,
        )?;

        let frag_shader = compile_shader(
            &gl,
            WebGlRenderingContext::FRAGMENT_SHADER,
            FRAGMENT_SHADER_SOURCE,
        )?;

        let program = link_program(&gl, &vert_shader, &frag_shader)?;
        gl.use_program(Some(&program));

        // バッファを作成
        let position_buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
        let color_buffer = gl.create_buffer().ok_or("Failed to create buffer")?;

        // パーティクルを生成
        let particles = create_particles(width, height, particle_count);

        Ok(ParticleSystem {
            particles,
            gl,
            program,
            position_buffer,
            color_buffer,
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
        let gl = &self.gl;

        // 画面クリア
        gl.clear_color(0.1, 0.1, 0.1, 1.0);
        gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

        // 位置データを準備 (100,000個分!)
        let mut positions = Vec::with_capacity(self.particles.len() * 2);
        let mut colors = Vec::with_capacity(self.particles.len() * 3);

        for p in &self.particles {
            // 正規化座標に変換 (-1.0 ~ 1.0)
            positions.push((p.x / self.width) * 2.0 - 1.0);
            positions.push(1.0 - (p.y / self.height) * 2.0);

            // HSLからRGBに変換
            let rgb = hsl_to_rgb(p.hue, 1.0, 0.5);
            colors.push(rgb.0);
            colors.push(rgb.1);
            colors.push(rgb.2);
        }

        // 位置バッファにデータを送る
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&self.position_buffer));
        unsafe {
            let positions_array = js_sys::Float32Array::view(&positions);
            gl.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ARRAY_BUFFER,
                &positions_array,
                WebGlRenderingContext::DYNAMIC_DRAW,
            );
        }

        let position_attrib = gl.get_attrib_location(&self.program, "a_position") as u32;
        gl.vertex_attrib_pointer_with_i32(
            position_attrib,
            2,
            WebGlRenderingContext::FLOAT,
            false,
            0,
            0,
        );
        gl.enable_vertex_attrib_array(position_attrib);

        // 色バッファにデータを送る
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&self.color_buffer));
        unsafe {
            let colors_array = js_sys::Float32Array::view(&colors);
            gl.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ARRAY_BUFFER,
                &colors_array,
                WebGlRenderingContext::DYNAMIC_DRAW,
            );
        }

        let color_attrib = gl.get_attrib_location(&self.program, "a_color") as u32;
        gl.vertex_attrib_pointer_with_i32(
            color_attrib,
            3,
            WebGlRenderingContext::FLOAT,
            false,
            0,
            0,
        );
        gl.enable_vertex_attrib_array(color_attrib);

        // ポイントサイズを設定（WebGLは直径、Canvas2Dは半径なので2倍）
        let point_size_location = gl.get_uniform_location(&self.program, "u_pointSize");
        gl.uniform1f(point_size_location.as_ref(), 2.5 * 2.0);

        // 描画! (GPUが一瞬で10万個を描画)
        gl.draw_arrays(WebGlRenderingContext::POINTS, 0, self.particles.len() as i32);
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

// シェーダーコンパイル
fn compile_shader(
    gl: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<web_sys::WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

// プログラムリンク
fn link_program(
    gl: &WebGlRenderingContext,
    vert_shader: &web_sys::WebGlShader,
    frag_shader: &web_sys::WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = gl
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    gl.attach_shader(&program, vert_shader);
    gl.attach_shader(&program, frag_shader);
    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}

// 頂点シェーダー
const VERTEX_SHADER_SOURCE: &str = r#"
    attribute vec2 a_position;
    attribute vec3 a_color;
    uniform float u_pointSize;
    varying vec3 v_color;

    void main() {
        gl_Position = vec4(a_position, 0.0, 1.0);
        gl_PointSize = u_pointSize;
        v_color = a_color;
    }
"#;

// フラグメントシェーダー
const FRAGMENT_SHADER_SOURCE: &str = r#"
    precision mediump float;
    varying vec3 v_color;

    void main() {
        gl_FragColor = vec4(v_color, 0.8);
    }
"#;
