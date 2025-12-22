/**
 * パーティクルシミュレーション共通物理定数
 *
 * すべての実装（Rust+WebGL、TypeScript+Canvas、TypeScript+SVG）で
 * 同じ物理挙動を実現するための定数定義
 */

export const PHYSICS_CONSTANTS = {
    // 重力加速度
    GRAVITY: 0.0002,

    // 反発係数（壁との衝突時のエネルギー保存率）
    BOUNCE: 0.85,

    // 爆発力
    EXPLOSION_FORCE: 8.0,

    // 爆発半径（ピクセル）
    EXPLOSION_RADIUS: 200,

    // 床摩擦係数
    FRICTION: 0.98,

    // 初期速度範囲
    INITIAL_SPEED_MIN: 1.0,
    INITIAL_SPEED_MAX: 3.0,

    // 初期Y方向速度（上向き）
    INITIAL_VY_OFFSET: -3.0,

    // パーティクルサイズ範囲
    PARTICLE_SIZE_MIN: 1.0,
    PARTICLE_SIZE_MAX: 3.0,

    // 色相変化速度（フレームごと）
    HUE_SHIFT_SPEED: 0.3,

    // キャンバスサイズ
    CANVAS_WIDTH: 1200,
    CANVAS_HEIGHT: 800,

    // 描画サイズ（固定）
    RENDER_SIZE: 2.5,
};

/**
 * Rust側で使用するための定数をコメントで記載
 *
 * const GRAVITY: f32 = 0.0002;
 * const BOUNCE: f32 = 0.85;
 * const EXPLOSION_FORCE: f32 = 8.0;
 * const EXPLOSION_RADIUS: f32 = 200.0;
 * const FRICTION: f32 = 0.98;
 * const INITIAL_SPEED_MIN: f32 = 1.0;
 * const INITIAL_SPEED_MAX: f32 = 3.0;
 * const INITIAL_VY_OFFSET: f32 = -3.0;
 * const PARTICLE_SIZE_MIN: f32 = 1.0;
 * const PARTICLE_SIZE_MAX: f32 = 3.0;
 * const HUE_SHIFT_SPEED: f32 = 0.3;
 * const CANVAS_WIDTH: f32 = 1200.0;
 * const CANVAS_HEIGHT: f32 = 800.0;
 * const RENDER_SIZE: f32 = 2.5;
 */
