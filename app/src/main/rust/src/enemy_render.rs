//! Renderizador Especializado de Enemigos de IA Rebellion
//! Dibuja naves, drones, sintéticos humanoides y bio-plantas con orientación continua 360°,
//! rotación coherente con la trayectoria de vuelo (heading) y montaje estructural sólido.

use crate::enemy::{Enemy, EnemyAnchor, EnemyType};
use crate::renderer::Renderer;

#[inline(always)]
fn rotate_local(lx: f32, ly: f32, cos_h: f32, sin_h: f32, cx: f32, cy: f32) -> (f32, f32) {
    (cx + lx * cos_h - ly * sin_h, cy + lx * sin_h + ly * cos_h)
}

impl Renderer {
    /// Despacho principal de dibujo de enemigos según arquetipo y estado táctico
    pub fn draw_enemy(&self, buffer: &mut [u32], e: &Enemy) {
        match e.enemy_type {
            EnemyType::ItemCarrier => {
                self.draw_item_carrier(buffer, e, self.anim_time);
            }
            EnemyType::SynthKatana => {
                self.draw_modern_humanoid_synth(buffer, e, false, true);
            }
            EnemyType::SynthRifle => {
                self.draw_modern_humanoid_synth(buffer, e, true, false);
            }
            EnemyType::BioPlantVine => {
                self.draw_cyber_vine(buffer, e.x, e.y, e.radius * 2.5, e.aim_angle, self.anim_time, 0xFF14532D, 0xFF22C55E);
            }
            EnemyType::BioPlantSporePod => {
                self.draw_spore_pod_turret(buffer, e, self.anim_time);
            }
            EnemyType::BioPlantFlowerTrap => {
                let open_factor = ((self.anim_time * 2.5).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
                self.draw_flower_trap(buffer, e, self.anim_time, open_factor);
            }
            EnemyType::PredatoryDrone => {
                self.draw_predatory_drone(buffer, e, self.anim_time);
            }
            EnemyType::BioMechLeviathan => {
                self.draw_biomech_leviathan(buffer, e, self.anim_time);
            }
            EnemyType::PatrolDrone => {
                self.draw_patrol_drone(buffer, e, self.anim_time);
            }
            EnemyType::KamikazeWasp => {
                self.draw_kamikaze_wasp(buffer, e, self.anim_time);
            }
            EnemyType::LaserTurret => {
                self.draw_laser_turret(buffer, e, self.anim_time);
            }
            EnemyType::CyberCrab => {
                self.draw_cyber_crab(buffer, e, self.anim_time);
            }
            EnemyType::AsteroidLeech => {
                self.draw_cyber_vine(buffer, e.x, e.y, e.radius * 2.5, e.aim_angle, self.anim_time, 0xFF2E1065, 0xFF9333EA);
            }
            EnemyType::StealthStriker => {
                self.draw_modern_humanoid_synth(buffer, e, true, true);
            }
        }
    }

    /// 1. Sintéticos Humanoides Modernos (Porcelana Blanca / Obsidiana) con giro horizontal coherente
    pub fn draw_modern_humanoid_synth(
        &self,
        buffer: &mut [u32],
        enemy: &Enemy,
        is_obsidian: bool,
        has_katana: bool,
    ) {
        let ex = enemy.x;
        let ey = enemy.y;
        let t = self.anim_time;
        let aim = enemy.aim_angle;
        let facing = enemy.facing_dir.clamp(-1.0, 1.0);

        let (mut plate_col, mut joint_col, glow_col, highlight_col) = if is_obsidian {
            (0xFF0F172A, 0xFF020617, 0xFF00F0FF, 0xFF38BDF8)
        } else {
            (0xFFF1F5F9, 0xFF1E293B, 0xFFFF0055, 0xFFFFFFFF)
        };

        if enemy.hit_flash > 0.0 {
            plate_col = Self::lerp_color(plate_col, 0xFFFFFFFF, enemy.hit_flash * 0.8);
            joint_col = Self::lerp_color(joint_col, 0xFFFFCC00, enemy.hit_flash * 0.7);
        }

        let hover_y = (t * 5.0 + ex * 0.05).sin() * 3.5;
        let cy = ey + hover_y;

        // Alas de energía dorsal orientadas hacia la popa
        let wing_dir = -facing;
        let wing_flare = (t * 14.0).sin().abs() * 6.0;
        self.draw_aa_capsule(buffer, ex - wing_dir * 4.0, cy - 6.0, ex + wing_dir * (18.0 + wing_flare), cy - 18.0, 2.4, glow_col);
        self.draw_aa_capsule(buffer, ex - wing_dir * 4.0, cy + 2.0, ex + wing_dir * (16.0 + wing_flare), cy + 12.0, 2.0, glow_col);

        // Piernas biónicas articuladas
        let leg_swing = (t * 7.0).sin();
        let left_thigh_end = (ex + facing * (leg_swing * 6.0), cy + 14.0);
        let right_thigh_end = (ex - facing * (leg_swing * 6.0), cy + 14.0);
        let left_foot = (left_thigh_end.0 + facing * 4.0, cy + 28.0);
        let right_foot = (right_thigh_end.0 - facing * 4.0, cy + 28.0);

        self.draw_aa_capsule(buffer, ex, cy, left_thigh_end.0, left_thigh_end.1, 4.0, joint_col);
        self.draw_aa_capsule(buffer, left_thigh_end.0, left_thigh_end.1, left_foot.0, left_foot.1, 3.2, joint_col);
        self.draw_aa_capsule(buffer, ex, cy, right_thigh_end.0, right_thigh_end.1, 4.0, joint_col);
        self.draw_aa_capsule(buffer, right_thigh_end.0, right_thigh_end.1, right_foot.0, right_foot.1, 3.2, joint_col);

        self.draw_aa_capsule(buffer, ex - 1.0, cy + 2.0, left_thigh_end.0, left_thigh_end.1, 3.0, plate_col);
        self.draw_aa_capsule(buffer, ex + 1.0, cy + 2.0, right_thigh_end.0, right_thigh_end.1, 3.0, plate_col);
        self.draw_aa_capsule(buffer, left_foot.0 - 4.0, left_foot.1, left_foot.0 + 5.0, left_foot.1, 2.2, highlight_col);
        self.draw_aa_capsule(buffer, right_foot.0 - 4.0, right_foot.1, right_foot.0 + 5.0, right_foot.1, 2.2, highlight_col);

        // Torso cibernético blindado
        self.draw_aa_capsule(buffer, ex, cy - 2.0, ex, cy + 10.0, 7.5, joint_col);
        self.draw_aa_capsule(buffer, ex, cy - 4.0, ex, cy + 6.0, 6.5, plate_col);
        self.draw_aa_capsule(buffer, ex, cy - 2.0, ex, cy + 5.0, 1.8, glow_col);
        self.draw_volumetric_light(buffer, ex, cy + 1.0, 16.0, glow_col, 0.55);

        // Cabeza con halo neuronal
        let head_y = cy - 14.0;
        self.draw_aa_capsule(buffer, ex - 8.0, head_y - 9.0, ex + 8.0, head_y - 9.0, 1.2, glow_col);
        self.draw_aa_circle(buffer, ex, head_y, 7.0, plate_col, joint_col, 1.2);
        self.draw_aa_capsule(buffer, ex - 3.0, head_y - 4.5, ex + 2.0, head_y - 4.5, 1.5, highlight_col);

        // Visor óptico telemétrico con seguimiento dinámico
        let eye_offset_x = (aim.cos() * 4.5).clamp(-5.0, 5.0);
        let eye_offset_y = (aim.sin() * 2.5).clamp(-3.0, 3.0);
        let eye_x = ex + eye_offset_x;
        let eye_y = head_y + eye_offset_y;

        self.draw_aa_capsule(buffer, ex - 4.5, head_y, ex + 4.5, head_y, 2.0, 0xFF020617);
        self.draw_aa_circle(buffer, eye_x, eye_y, 2.0, 0xFFFFFFFF, glow_col, 0.8);
        self.draw_aa_capsule(buffer, eye_x - 7.0, eye_y, eye_x + 7.0, eye_y, 0.9, glow_col);
        self.draw_volumetric_light(buffer, eye_x, eye_y, 22.0, glow_col, 0.65);

        if has_katana {
            let shoulder_x = ex + facing * 4.0;
            let shoulder_y = cy - 5.0;
            let hand_x = ex + aim.cos() * 12.0;
            let hand_y = cy + aim.sin() * 12.0;
            self.draw_aa_capsule(buffer, shoulder_x, shoulder_y, hand_x, hand_y, 3.0, plate_col);

            let hilt_len = 8.0;
            let blade_dir_x = aim.cos();
            let blade_dir_y = aim.sin();
            let hilt_tip_x = hand_x + blade_dir_x * hilt_len;
            let hilt_tip_y = hand_y + blade_dir_y * hilt_len;
            self.draw_aa_capsule(buffer, hand_x, hand_y, hilt_tip_x, hilt_tip_y, 2.2, 0xFF1E293B);

            let blade_len = 36.0;
            let blade_tip_x = hilt_tip_x + blade_dir_x * blade_len;
            let blade_tip_y = hilt_tip_y + blade_dir_y * blade_len;
            self.draw_aa_capsule(buffer, hilt_tip_x, hilt_tip_y, blade_tip_x, blade_tip_y, 5.0, glow_col);
            self.draw_aa_capsule(buffer, hilt_tip_x, hilt_tip_y, blade_tip_x, blade_tip_y, 2.0, 0xFFFFFFFF);
            self.draw_volumetric_light(buffer, (hilt_tip_x + blade_tip_x) * 0.5, (hilt_tip_y + blade_tip_y) * 0.5, 30.0, glow_col, 0.55);
        } else {
            let shoulder_x = ex + facing * 3.0;
            let shoulder_y = cy - 4.0;
            let hand_x = ex + aim.cos() * 14.0;
            let hand_y = cy + aim.sin() * 14.0;
            self.draw_aa_capsule(buffer, shoulder_x, shoulder_y, hand_x, hand_y, 3.2, plate_col);

            let barrel_dir_x = aim.cos();
            let barrel_dir_y = aim.sin();
            let r_body_x = hand_x + barrel_dir_x * 10.0;
            let r_body_y = hand_y + barrel_dir_y * 10.0;
            self.draw_aa_capsule(buffer, hand_x, hand_y, r_body_x, r_body_y, 4.2, 0xFF0F172A);

            let muzzle_x = r_body_x + barrel_dir_x * 16.0;
            let muzzle_y = r_body_y + barrel_dir_y * 16.0;
            self.draw_aa_capsule(buffer, r_body_x, r_body_y, muzzle_x, muzzle_y, 2.4, 0xFF64748B);
            self.draw_aa_circle(buffer, r_body_x, r_body_y, 2.0, glow_col, 0xFFFFFFFF, 0.5);

            let sight_len = 80.0;
            let sight_x = muzzle_x + barrel_dir_x * sight_len;
            let sight_y = muzzle_y + barrel_dir_y * sight_len;
            self.draw_aa_capsule(buffer, muzzle_x, muzzle_y, sight_x, sight_y, 0.8, glow_col);
        }
    }

    /// 2. Dron Cazador Predatorio de Alas en Flecha Invertida con rotación de rumbo continua 360°
    pub fn draw_predatory_drone(&self, buffer: &mut [u32], enemy: &Enemy, anim_time: f32) {
        let ex = enemy.x;
        let ey = enemy.y;
        let h = enemy.heading;
        let cos_h = h.cos();
        let sin_h = h.sin();

        let mut body_col = 0xFF0F172A;
        let mut wing_col = 0xFF38BDF8;
        let sensor_col = 0xFFFF0033;
        if enemy.hit_flash > 0.0 {
            body_col = Self::lerp_color(body_col, 0xFFFFFFFF, enemy.hit_flash * 0.85);
            wing_col = Self::lerp_color(wing_col, 0xFFFFCC00, enemy.hit_flash * 0.7);
        }

        let rot = |lx: f32, ly: f32| rotate_local(lx, ly, cos_h, sin_h, ex, ey);

        // Toberas y propulsores de plasma en popa (-lx)
        let (e1_x, e1_y) = rot(-14.0, -8.0);
        let (e2_x, e2_y) = rot(-14.0, 8.0);
        let (e1_b_x, e1_b_y) = rot(-2.0, -8.0);
        let (e2_b_x, e2_b_y) = rot(-2.0, 8.0);
        self.draw_aa_capsule(buffer, e1_x, e1_y, e1_b_x, e1_b_y, 4.0, 0xFF1E293B);
        self.draw_aa_capsule(buffer, e2_x, e2_y, e2_b_x, e2_b_y, 4.0, 0xFF1E293B);

        let exhaust_len = 16.0 + (anim_time * 30.0).sin().abs() * 8.0;
        self.draw_plasma_plume(buffer, e1_x, e1_y, -cos_h, -sin_h, exhaust_len, 5.0, 0xFFFFFFFF, 0xFFFF4500);
        self.draw_plasma_plume(buffer, e2_x, e2_y, -cos_h, -sin_h, exhaust_len, 5.0, 0xFFFFFFFF, 0xFFFF4500);

        // Alas en flecha invertida (adelantadas hacia el frente +lx)
        let (w_root1_x, w_root1_y) = rot(-4.0, -4.0);
        let (w_mid1_x, w_mid1_y) = rot(14.0, -20.0);
        let (w_tip1_x, w_tip1_y) = rot(22.0, -16.0);
        self.draw_aa_capsule(buffer, w_root1_x, w_root1_y, w_mid1_x, w_mid1_y, 4.5, body_col);
        self.draw_aa_capsule(buffer, w_mid1_x, w_mid1_y, w_tip1_x, w_tip1_y, 3.2, wing_col);

        let (w_root2_x, w_root2_y) = rot(-4.0, 4.0);
        let (w_mid2_x, w_mid2_y) = rot(14.0, 20.0);
        let (w_tip2_x, w_tip2_y) = rot(22.0, 16.0);
        self.draw_aa_capsule(buffer, w_root2_x, w_root2_y, w_mid2_x, w_mid2_y, 4.5, body_col);
        self.draw_aa_capsule(buffer, w_mid2_x, w_mid2_y, w_tip2_x, w_tip2_y, 3.2, wing_col);

        // Fuselaje central aerodinámico
        let (f_tail_x, f_tail_y) = rot(-10.0, 0.0);
        let (f_nose_x, f_nose_y) = rot(16.0, 0.0);
        self.draw_aa_capsule(buffer, f_tail_x, f_tail_y, f_nose_x, f_nose_y, 7.5, 0xFF1E293B);
        self.draw_aa_capsule(buffer, f_tail_x + cos_h * 4.0, f_tail_y + sin_h * 4.0, f_nose_x - cos_h * 2.0, f_nose_y - sin_h * 2.0, 5.5, 0xFF475569);

        // Sensor óptico frontal rojo en la proa
        self.draw_aa_circle(buffer, f_nose_x, f_nose_y, 4.8, sensor_col, 0xFFFFFFFF, 1.4);
        let (ant1_x, ant1_y) = rot(16.0, -9.0);
        let (ant2_x, ant2_y) = rot(16.0, 9.0);
        self.draw_aa_capsule(buffer, ant1_x, ant1_y, ant2_x, ant2_y, 1.2, 0xFF00F0FF);
        self.draw_volumetric_light(buffer, f_nose_x, f_nose_y, 26.0, sensor_col, 0.75);
    }

    /// 3. Dron de Patrulla y Reconocimiento con turbinas VTOL y foco rotatorio
    pub fn draw_patrol_drone(&self, buffer: &mut [u32], enemy: &Enemy, anim_time: f32) {
        let ex = enemy.x;
        let ey = enemy.y;
        let h = enemy.heading;
        let cos_h = h.cos();
        let sin_h = h.sin();

        let mut body_col = 0xFF0F172A;
        if enemy.hit_flash > 0.0 {
            body_col = Self::lerp_color(body_col, 0xFFFFFFFF, enemy.hit_flash * 0.85);
        }

        let rot = |lx: f32, ly: f32| rotate_local(lx, ly, cos_h, sin_h, ex, ey);

        // Turbinas ducted-fan laterales en las alas (±ly)
        let t_rot = anim_time * 28.0;
        let fan_blade = (t_rot.sin() * 5.0).abs() + 2.0;

        let (left_w_x, left_w_y) = rot(0.0, -14.0);
        let (right_w_x, right_w_y) = rot(0.0, 14.0);

        self.draw_aa_circle(buffer, left_w_x, left_w_y, 7.5, body_col, 0xFF38BDF8, 1.8);
        self.draw_aa_circle(buffer, right_w_x, right_w_y, 7.5, body_col, 0xFF38BDF8, 1.8);

        let perp_x = -sin_h * fan_blade;
        let perp_y = cos_h * fan_blade;
        self.draw_aa_capsule(buffer, left_w_x - perp_x, left_w_y - perp_y, left_w_x + perp_x, left_w_y + perp_y, 1.2, 0xFF00E5FF);
        self.draw_aa_capsule(buffer, right_w_x - perp_x, right_w_y - perp_y, right_w_x + perp_x, right_w_y + perp_y, 1.2, 0xFF00E5FF);

        // Estructura alar que conecta las turbinas al fuselaje
        let (wing_center_x, wing_center_y) = rot(-6.0, 0.0);
        self.draw_aa_capsule(buffer, wing_center_x, wing_center_y, left_w_x, left_w_y, 3.5, 0xFF1E293B);
        self.draw_aa_capsule(buffer, wing_center_x, wing_center_y, right_w_x, right_w_y, 3.5, 0xFF1E293B);

        // Fuselaje furtivo
        let (f_tail_x, f_tail_y) = rot(-10.0, 0.0);
        let (f_nose_x, f_nose_y) = rot(14.0, 0.0);
        self.draw_aa_capsule(buffer, f_tail_x, f_tail_y, f_nose_x, f_nose_y, 6.5, body_col);
        self.draw_aa_capsule(buffer, f_tail_x + cos_h * 4.0, f_tail_y + sin_h * 4.0, f_nose_x - cos_h * 2.0, f_nose_y - sin_h * 2.0, 4.5, 0xFF334155);

        // Sensor y cono de foco de barrido
        self.draw_aa_circle(buffer, f_nose_x, f_nose_y, 4.0, 0xFFFFCC00, 0xFFFFFFFF, 1.0);
        self.draw_volumetric_light(buffer, f_nose_x, f_nose_y, 22.0, 0xFFFFCC00, 0.65);

        let beam_len = 36.0;
        let beam_end_x = f_nose_x + cos_h * beam_len;
        let beam_end_y = f_nose_y + sin_h * beam_len;
        let beam_spread_x = -sin_h * 10.0;
        let beam_spread_y = cos_h * 10.0;
        self.draw_aa_capsule(buffer, f_nose_x, f_nose_y, beam_end_x - beam_spread_x, beam_end_y - beam_spread_y, 0.8, 0x44FFD700);
        self.draw_aa_capsule(buffer, f_nose_x, f_nose_y, beam_end_x + beam_spread_x, beam_end_y + beam_spread_y, 0.8, 0x44FFD700);

        // Pluma de propulsión de cola
        self.draw_plasma_plume(buffer, f_tail_x, f_tail_y, -cos_h, -sin_h, 12.0, 4.0, 0xFFFFFFFF, 0xFF00E5FF);
    }

    /// 4. Avispa Kamikaze Cibernética orientada con la trayectoria
    pub fn draw_kamikaze_wasp(&self, buffer: &mut [u32], enemy: &Enemy, anim_time: f32) {
        let ex = enemy.x;
        let ey = enemy.y;
        let h = enemy.heading;
        let cos_h = h.cos();
        let sin_h = h.sin();

        let rot = |lx: f32, ly: f32| rotate_local(lx, ly, cos_h, sin_h, ex, ey);

        let wing_flap = (anim_time * 55.0).sin();
        let wing_span = 12.0 * wing_flap;

        // Alas vibrantes translúcidas
        let (w_root1_x, w_root1_y) = rot(-2.0, -2.0);
        let (w_tip1_x, w_tip1_y) = rot(-6.0, -10.0 + wing_span);
        let (w_root2_x, w_root2_y) = rot(-2.0, 2.0);
        let (w_tip2_x, w_tip2_y) = rot(-6.0, 10.0 - wing_span);
        self.draw_aa_capsule(buffer, w_root1_x, w_root1_y, w_tip1_x, w_tip1_y, 1.6, 0x9900E5FF);
        self.draw_aa_capsule(buffer, w_root2_x, w_root2_y, w_tip2_x, w_tip2_y, 1.6, 0x9900E5FF);

        // Tórax mecanizado
        self.draw_aa_circle(buffer, ex, ey, 6.2, 0xFF2E1065, 0xFF581C87, 1.5);

        // Abdomen reactor supercalentado en popa (-lx)
        let heat_pulse = (anim_time * 16.0).sin().abs();
        let heat_col = Self::lerp_color(0xFFFF3300, 0xFFFFCC00, heat_pulse);
        let (abd_x, abd_y) = rot(-10.0, 0.0);
        self.draw_aa_circle(buffer, abd_x, abd_y, 5.5, heat_col, 0xFFFFFFFF, 1.2);
        self.draw_volumetric_light(buffer, abd_x, abd_y, 22.0, heat_col, 0.75);

        // Ojos facetados y aguijón frontal en proa (+lx)
        let (head_x, head_y) = rot(7.0, 0.0);
        self.draw_aa_circle(buffer, head_x, head_y, 3.8, 0xFF020617, 0xFFFF0033, 1.2);

        let (stinger_x, stinger_y) = rot(16.0, 0.0);
        self.draw_aa_capsule(buffer, head_x, head_y, stinger_x, stinger_y, 1.4, 0xFFFFFFFF);
        self.draw_aa_circle(buffer, stinger_x, stinger_y, 1.8, 0xFFFFFFFF, 0xFFFF0055, 0.8);
    }

    /// 5. Torreta Antiaérea Pesada con montaje articulado a superficie y doble cañón
    pub fn draw_laser_turret(&self, buffer: &mut [u32], enemy: &Enemy, anim_time: f32) {
        let ex = enemy.x;
        let ey = enemy.y;

        self.draw_turret_mounting_base(buffer, ex, ey, enemy.anchor, anim_time);

        let cos_a = enemy.aim_angle.cos();
        let sin_a = enemy.aim_angle.sin();
        let perp_x = -sin_a;
        let perp_y = cos_a;

        // Base circular blindada
        let base_r = 16.5;
        let mut base_col = 0xFF0F172A;
        if enemy.hit_flash > 0.0 {
            base_col = Self::lerp_color(base_col, 0xFFFFFFFF, enemy.hit_flash * 0.8);
        }
        self.draw_aa_circle(buffer, ex, ey, base_r, base_col, 0xFF334155, 2.4);
        self.draw_aa_circle(buffer, ex, ey, base_r - 3.5, 0xFF1E293B, 0xFF475569, 1.5);

        // Pernos de blindaje perimetrales
        for r_i in 0..6 {
            let r_ang = (r_i as f32) * (std::f32::consts::PI / 3.0) + (anim_time * 0.5);
            let rx = ex + r_ang.cos() * (base_r - 2.0);
            let ry = ey + r_ang.sin() * (base_r - 2.0);
            self.draw_circle(buffer, rx as isize, ry as isize, 1, 0xFF94A3B8);
        }

        // Bloque de recámara
        let breech_x = ex + cos_a * 4.0;
        let breech_y = ey + sin_a * 4.0;
        self.draw_aa_capsule(buffer, breech_x - perp_x * 9.0, breech_y - perp_y * 9.0, breech_x + perp_x * 9.0, breech_y + perp_y * 9.0, 5.0, 0xFF0F172A);

        // Doble cañón pesado
        let barrel_spacing = 5.5;
        let barrel_len = 20.0;
        let sleeve_len = 10.0;

        for &sign in &[-1.0f32, 1.0f32] {
            let offset_x = perp_x * barrel_spacing * sign;
            let offset_y = perp_y * barrel_spacing * sign;

            let b_start_x = ex + offset_x + cos_a * 2.0;
            let b_start_y = ey + offset_y + sin_a * 2.0;
            let b_end_x = b_start_x + cos_a * barrel_len;
            let b_end_y = b_start_y + sin_a * barrel_len;

            self.draw_aa_capsule(buffer, b_start_x, b_start_y, b_end_x, b_end_y, 3.2, 0xFF0B0F19);
            self.draw_aa_capsule(buffer, b_start_x, b_start_y, b_end_x, b_end_y, 1.8, 0xFF1E293B);

            let s_end_x = b_start_x + cos_a * sleeve_len;
            let s_end_y = b_start_y + sin_a * sleeve_len;
            self.draw_aa_capsule(buffer, b_start_x, b_start_y, s_end_x, s_end_y, 4.2, 0xFF1E293B);

            let muzzle_x = b_end_x;
            let muzzle_y = b_end_y;
            let brake_x1 = muzzle_x - perp_x * 2.5;
            let brake_y1 = muzzle_y - perp_y * 2.5;
            let brake_x2 = muzzle_x + perp_x * 2.5;
            let brake_y2 = muzzle_y + perp_y * 2.5;
            self.draw_aa_capsule(buffer, brake_x1, brake_y1, brake_x2, brake_y2, 3.0, 0xFF0F172A);
            self.draw_aa_capsule(buffer, brake_x1, brake_y1, brake_x2, brake_y2, 1.5, 0xFF64748B);
            self.draw_circle(buffer, muzzle_x as isize, muzzle_y as isize, 1, 0xFF020617);

            if enemy.fire_timer < 0.35 {
                let charge_glow = ((0.35 - enemy.fire_timer) / 0.35).clamp(0.0, 1.0);
                let charge_col = Self::lerp_color(0xFF0099FF, 0xFF00FFFF, charge_glow);
                self.draw_circle(buffer, muzzle_x as isize, muzzle_y as isize, 2, charge_col);
                self.draw_point_light(buffer, muzzle_x as isize, muzzle_y as isize, 8, charge_col, charge_glow * 0.7);
            }
        }

        // Mantlete frontal con franja de advertencia
        let shield_x = ex + cos_a * 6.0;
        let shield_y = ey + sin_a * 6.0;
        let s_top_x = shield_x + perp_x * 12.0;
        let s_top_y = shield_y + perp_y * 12.0;
        let s_bot_x = shield_x - perp_x * 12.0;
        let s_bot_y = shield_y - perp_y * 12.0;
        self.draw_aa_capsule(buffer, s_top_x, s_top_y, s_bot_x, s_bot_y, 4.0, 0xFF1E293B);
        self.draw_aa_capsule(buffer, s_top_x, s_top_y, s_bot_x, s_bot_y, 2.0, 0xFF64748B);

        // Sensor óptico central rojo
        let optic_x = ex - cos_a * 2.0;
        let optic_y = ey - sin_a * 2.0;
        self.draw_aa_circle(buffer, optic_x, optic_y, 3.8, 0xFF0F172A, 0xFFFF1133, 1.4);
        self.draw_circle(buffer, optic_x as isize, optic_y as isize, 1, 0xFFFFFFFF);
        self.draw_point_light(buffer, optic_x as isize, optic_y as isize, 10, 0xFFFF0033, 0.5);
    }

    /// 6. Cangrejo Cibernético Blindado con patas y pinzas
    pub fn draw_cyber_crab(&self, buffer: &mut [u32], enemy: &Enemy, anim_time: f32) {
        let ex = enemy.x;
        let ey = enemy.y;
        let r = enemy.radius;
        let leg_step = (anim_time * 6.0).sin();

        let leg_angles = [
            (-0.7, -18.0 + leg_step * 4.0),
            (-0.3, -22.0 - leg_step * 4.0),
            (0.3, 22.0 + leg_step * 4.0),
            (0.7, 18.0 - leg_step * 4.0),
        ];

        for (angle_off, spread_y) in leg_angles {
            let hip_x = ex + angle_off * (r * 0.5);
            let hip_y = ey + spread_y * 0.35;
            let knee_x = hip_x - 12.0;
            let knee_y = ey + spread_y * 0.8;
            let foot_x = knee_x - 8.0;
            let foot_y = ey + spread_y;

            self.draw_aa_capsule(buffer, hip_x, hip_y, knee_x, knee_y, 3.8, 0xFF0F172A);
            self.draw_aa_capsule(buffer, hip_x, hip_y, knee_x, knee_y, 1.6, 0xFF64748B);
            self.draw_aa_capsule(buffer, knee_x, knee_y, foot_x, foot_y, 2.8, 0xFF1E293B);
            self.draw_aa_circle(buffer, foot_x, foot_y, 2.0, 0xFFFF5500, 0xFFFFFFFF, 0.6);
        }

        let mut shell_col = 0xFF1E293B;
        if enemy.hit_flash > 0.0 {
            shell_col = Self::lerp_color(shell_col, 0xFFFFFFFF, enemy.hit_flash * 0.85);
        }

        self.draw_aa_circle(buffer, ex, ey, r * 0.82, shell_col, 0xFFC2410C, 2.8);
        self.draw_aa_circle(buffer, ex, ey, r * 0.62, 0xFF0F172A, 0xFFEA580C, 1.8);

        self.draw_aa_capsule(buffer, ex - 8.0, ey - 4.0, ex + 8.0, ey - 4.0, 1.8, 0xFFFFD700);
        self.draw_aa_capsule(buffer, ex - 8.0, ey + 4.0, ex + 8.0, ey + 4.0, 1.8, 0xFFFFD700);

        let claw_cycle = (anim_time * 5.0).sin().abs() * 6.0;
        let c_top_y = ey - 10.0 - claw_cycle;
        let c_bot_y = ey + 10.0 + claw_cycle;

        self.draw_aa_capsule(buffer, ex - r * 0.5, ey - 6.0, ex - r * 0.8 - 6.0, c_top_y, 3.5, 0xFF334155);
        self.draw_aa_capsule(buffer, ex - r * 0.8 - 6.0, c_top_y, ex - r - 8.0, ey - 4.0, 2.4, 0xFFEA580C);

        self.draw_aa_capsule(buffer, ex - r * 0.5, ey + 6.0, ex - r * 0.8 - 6.0, c_bot_y, 3.5, 0xFF334155);
        self.draw_aa_capsule(buffer, ex - r * 0.8 - 6.0, c_bot_y, ex - r - 8.0, ey + 4.0, 2.4, 0xFFEA580C);

        self.draw_aa_circle(buffer, ex - r * 0.65, ey, 4.2, 0xFFFF0033, 0xFFFFFFFF, 1.0);
        self.draw_volumetric_light(buffer, ex - r * 0.65, ey, 20.0, 0xFFFF0033, 0.7);
    }

    /// 7. Leviatán Mecánico Biomecánico con pinzas trituradoras
    pub fn draw_biomech_leviathan(&self, buffer: &mut [u32], enemy: &Enemy, anim_time: f32) {
        let ex = enemy.x;
        let ey = enemy.y;
        let r = enemy.radius;
        let h = enemy.heading;
        let cos_h = h.cos();
        let sin_h = h.sin();

        let rot = |lx: f32, ly: f32| rotate_local(lx, ly, cos_h, sin_h, ex, ey);

        let mut hull_col = 0xFF0F172A;
        if enemy.hit_flash > 0.0 {
            hull_col = Self::lerp_color(hull_col, 0xFFFFFFFF, enemy.hit_flash * 0.85);
        }

        self.draw_aa_circle(buffer, ex, ey, r, hull_col, 0xFF38BDF8, 3.0);
        self.draw_aa_circle(buffer, ex, ey, r - 5.0, 0xFF1E293B, 0xFF475569, 2.0);

        let pincer_cycle = (anim_time * 4.0).sin();
        let claw_open = (pincer_cycle * 8.0).max(0.0);

        // Pinza superior e inferior al frente (+lx)
        let (u_base_x, u_base_y) = rot(r * 0.3, -r * 0.4);
        let (u_j_x, u_j_y) = rot(r * 0.8, -r * 0.7);
        let (u_tip_x, u_tip_y) = rot(r * 1.3, -r * 0.7 - claw_open);
        self.draw_aa_capsule(buffer, u_base_x, u_base_y, u_j_x, u_j_y, 5.0, 0xFF334155);
        self.draw_aa_capsule(buffer, u_j_x, u_j_y, u_tip_x, u_tip_y, 4.0, 0xFF64748B);

        let (l_base_x, l_base_y) = rot(r * 0.3, r * 0.4);
        let (l_j_x, l_j_y) = rot(r * 0.8, r * 0.7);
        let (l_tip_x, l_tip_y) = rot(r * 1.3, r * 0.7 + claw_open);
        self.draw_aa_capsule(buffer, l_base_x, l_base_y, l_j_x, l_j_y, 5.0, 0xFF334155);
        self.draw_aa_capsule(buffer, l_j_x, l_j_y, l_tip_x, l_tip_y, 4.0, 0xFF64748B);

        let core_pulse = (anim_time * 6.0).sin().abs();
        let core_r = 10.0 + core_pulse * 2.5;
        let core_col = 0xFF00F0FF;
        self.draw_aa_circle(buffer, ex, ey, core_r, 0xFFFFFFFF, core_col, 2.5);
        self.draw_volumetric_light(buffer, ex, ey, 45.0, core_col, 0.85);
    }

    /// 8. Dron Transportador Dorado con Cápsula de Suministro
    pub fn draw_item_carrier(&self, buffer: &mut [u32], enemy: &Enemy, anim_time: f32) {
        let ex = enemy.x;
        let ey = enemy.y;
        let r = enemy.radius;
        let pulse = (anim_time * 5.0).sin().abs();

        let glow_col = 0xFFFFD700;
        self.draw_volumetric_light(buffer, ex, ey, r * 2.6, glow_col, 0.55 + pulse * 0.35);
        self.draw_volumetric_light(buffer, ex, ey, r * 1.4, 0xFFFFFBEB, 0.8);

        let ring_spin = anim_time * 3.2;
        for i in 0..6 {
            let a = ring_spin + (i as f32) * (std::f32::consts::PI / 3.0);
            let rx = ex + a.cos() * (r + 5.0 + pulse * 2.0);
            let ry = ey + a.sin() * (r * 0.75 + pulse * 1.5);
            self.draw_aa_circle(buffer, rx, ry, 1.8, 0xFFFFFFFF, 0xFFFFD700, 0.6);
        }

        let nacelle_y1 = ey - (r + 2.0);
        let nacelle_y2 = ey + (r + 2.0);
        self.draw_aa_capsule(buffer, ex - 10.0, nacelle_y1, ex + 8.0, nacelle_y1, 4.2, 0xFF1E293B);
        self.draw_aa_capsule(buffer, ex - 8.0, nacelle_y1, ex + 6.0, nacelle_y1, 2.2, 0xFFFFD700);
        self.draw_aa_capsule(buffer, ex - 10.0, nacelle_y2, ex + 8.0, nacelle_y2, 4.2, 0xFF1E293B);
        self.draw_aa_capsule(buffer, ex - 8.0, nacelle_y2, ex + 6.0, nacelle_y2, 2.2, 0xFFFFD700);

        self.draw_volumetric_light(buffer, ex, nacelle_y1, 14.0, 0xFF00E5FF, 0.5);
        self.draw_volumetric_light(buffer, ex, nacelle_y2, 14.0, 0xFF00E5FF, 0.5);

        self.draw_aa_capsule(buffer, ex + 10.0, ey, ex - 8.0, ey, r * 0.88, 0xFFB45309);
        self.draw_aa_capsule(buffer, ex + 8.0, ey, ex - 6.0, ey, r * 0.78, 0xFFFFD700);
        self.draw_aa_capsule(buffer, ex + 6.0, ey - 3.0, ex - 4.0, ey - 3.0, 2.2, 0xFFFFFBEB);

        let dome_r = 9.0;
        self.draw_aa_circle(buffer, ex - 1.0, ey, dome_r, 0x5500F0FF, 0xFFFFFFFF, 1.2);

        let crystal_rot = anim_time * 4.5;
        for c in 0..4 {
            let ca = crystal_rot + (c as f32) * (std::f32::consts::PI * 0.5);
            let cx = ex - 1.0 + ca.cos() * 5.2;
            let cy = ey + ca.sin() * 5.2;
            self.draw_aa_capsule(buffer, ex - 1.0, ey, cx, cy, 1.4, 0xFFFFFFFF);
            self.draw_aa_circle(buffer, cx, cy, 1.8, 0xFFFFFFFF, 0xFFFFD700, 0.5);
        }
        self.draw_aa_circle(buffer, ex - 1.0, ey, 4.0, 0xFFFFFFFF, 0xFFFDE047, 1.0);

        let thruster_x = ex + r * 0.75;
        self.draw_plasma_plume(buffer, thruster_x, ey - 5.0, 1.0, 0.0, 15.0 + pulse * 6.0, 3.8, 0xFFFFFFFF, 0xFF00E5FF);
        self.draw_plasma_plume(buffer, thruster_x, ey + 5.0, 1.0, 0.0, 15.0 + pulse * 6.0, 3.8, 0xFFFFFFFF, 0xFF00E5FF);
    }

    /// 9. Torretas de Esporas Biomecánicas
    pub fn draw_spore_pod_turret(&self, buffer: &mut [u32], enemy: &Enemy, anim_time: f32) {
        let ex = enemy.x;
        let ey = enemy.y;

        self.draw_turret_mounting_base(buffer, ex, ey, enemy.anchor, anim_time);

        let breath = (anim_time * 4.5).sin() * 1.5;
        let bulb_r = 16.0 + breath;

        let mut pod_col = 0xFF2E1065;
        if enemy.hit_flash > 0.0 {
            pod_col = Self::lerp_color(pod_col, 0xFFFFFFFF, enemy.hit_flash * 0.85);
        }

        self.draw_aa_circle(buffer, ex, ey, bulb_r, pod_col, 0xFF581C87, 2.5);
        self.draw_aa_circle(buffer, ex, ey, bulb_r - 3.5, 0xFF1A0B2E, 0xFF7E22CE, 1.5);

        let sac_r = 10.0 + breath * 0.7;
        let sac_col = 0xFF10B981;
        let inner_col = 0xFF34D399;
        self.draw_aa_circle(buffer, ex, ey, sac_r, inner_col, sac_col, 1.5);
        self.draw_volumetric_light(buffer, ex, ey, bulb_r + 20.0, sac_col, 0.65);

        let aim_cos = enemy.aim_angle.cos();
        let aim_sin = enemy.aim_angle.sin();
        let barrel_len = bulb_r + 6.0;
        let aim_x = ex + aim_cos * barrel_len;
        let aim_y = ey + aim_sin * barrel_len;

        self.draw_aa_capsule(buffer, ex, ey, aim_x, aim_y, 4.5, 0xFF0F172A);
        self.draw_aa_capsule(buffer, ex + aim_cos * (bulb_r * 0.5), ey + aim_sin * (bulb_r * 0.5), aim_x, aim_y, 2.2, 0xFF34D399);
        self.draw_aa_circle(buffer, aim_x, aim_y, 3.5, 0xFFFFFFFF, sac_col, 1.2);
    }

    /// 10. Flores Mecánicas Trampa con Pétalos Navaja
    pub fn draw_flower_trap(&self, buffer: &mut [u32], enemy: &Enemy, _anim_time: f32, open_factor: f32) {
        let ex = enemy.x;
        let ey = enemy.y;
        let petal_count = 6;
        let base_radius = 20.0;

        match enemy.anchor {
            EnemyAnchor::LeftWall => {
                let stem_base_x = 0.0f32;
                self.draw_rect(buffer, 0, (ey - 6.0) as isize, (ex - stem_base_x) as usize, 12, 0xFF14532D);
                self.draw_aa_capsule(buffer, stem_base_x, ey, ex, ey, 5.0, 0xFF14532D);
            }
            EnemyAnchor::RightWall => {
                let stem_base_x = self.width as f32;
                self.draw_rect(buffer, ex as isize, (ey - 6.0) as isize, (stem_base_x - ex) as usize, 12, 0xFF14532D);
                self.draw_aa_capsule(buffer, stem_base_x, ey, ex, ey, 5.0, 0xFF14532D);
            }
            EnemyAnchor::Ceiling => {
                self.draw_rect(buffer, (ex - 6.0) as isize, 0, 12, ey as usize, 0xFF14532D);
                self.draw_aa_capsule(buffer, ex, 0.0, ex, ey, 5.0, 0xFF14532D);
            }
            _ => {
                let ground_y = (ey + 45.0).min(self.height as f32);
                self.draw_rect(buffer, (ex - 6.0) as isize, ey as isize, 12, (ground_y - ey) as usize, 0xFF14532D);
                self.draw_aa_capsule(buffer, ex, ground_y, ex, ey, 5.0, 0xFF14532D);
            }
        }

        let stamen_glow = 0xFF00F0FF;
        let stamen_r = 7.5;
        self.draw_aa_circle(buffer, ex, ey, stamen_r, 0xFFFFFFFF, stamen_glow, 2.0);
        self.draw_volumetric_light(buffer, ex, ey, 16.0, stamen_glow, 0.65);

        for p in 0..petal_count {
            let angle = (p as f32) * (std::f32::consts::PI * 2.0 / petal_count as f32);
            let petal_dist = base_radius + 8.0;
            let px = ex + angle.cos() * petal_dist;
            let py = ey + angle.sin() * petal_dist;

            let side_angle1 = angle + 0.35 * (1.0 - open_factor * 0.3);
            let side_angle2 = angle - 0.35 * (1.0 - open_factor * 0.3);
            let sx1 = ex + side_angle1.cos() * (base_radius * 0.7);
            let sy1 = ey + side_angle1.sin() * (base_radius * 0.7);
            let sx2 = ex + side_angle2.cos() * (base_radius * 0.7);
            let sy2 = ey + side_angle2.sin() * (base_radius * 0.7);

            self.draw_aa_capsule(buffer, sx1, sy1, px, py, 2.2, 0xFF94A3B8);
            self.draw_aa_capsule(buffer, sx2, sy2, px, py, 2.2, 0xFF475569);
            self.draw_aa_capsule(buffer, ex, ey, px, py, 2.8, 0xFF1E293B);
            self.draw_aa_circle(buffer, px, py, 2.5, 0xFFFFFFFF, 0xFF38BDF8, 0.8);
        }
    }

    /// Base de anclaje físico reforzado para torretas fijadas a superficies
    pub fn draw_turret_mounting_base(
        &self,
        buffer: &mut [u32],
        ex: f32,
        ey: f32,
        anchor: EnemyAnchor,
        _anim_time: f32,
    ) {
        match anchor {
            EnemyAnchor::Floor => {
                let base_y = ey + 14.0;
                let ground_y = (ey + 55.0).min(self.height as f32);
                let foot_w = 46.0;
                self.draw_rect(buffer, (ex - foot_w * 0.5) as isize, base_y as isize + 10, foot_w as usize, (ground_y - base_y - 8.0).max(12.0) as usize, 0xFF334155);
                self.draw_rect(buffer, (ex - foot_w * 0.5) as isize, base_y as isize + 10, foot_w as usize, 2, 0xFF64748B);

                self.draw_aa_capsule(buffer, ex - 22.0, base_y + 10.0, ex + 22.0, base_y + 10.0, 4.0, 0xFF1E293B);
                self.draw_aa_capsule(buffer, ex - 18.0, base_y + 8.0, ex + 18.0, base_y + 8.0, 2.0, 0xFF94A3B8);

                self.draw_aa_capsule(buffer, ex - 18.0, base_y + 8.0, ex - 8.0, ey + 4.0, 4.0, 0xFF0F172A);
                self.draw_aa_capsule(buffer, ex - 16.0, base_y + 6.0, ex - 8.0, ey + 4.0, 1.8, 0xFF38BDF8);
                self.draw_aa_capsule(buffer, ex + 18.0, base_y + 8.0, ex + 8.0, ey + 4.0, 4.0, 0xFF0F172A);
                self.draw_aa_capsule(buffer, ex + 16.0, base_y + 6.0, ex + 8.0, ey + 4.0, 1.8, 0xFF38BDF8);

                self.draw_aa_circle(buffer, ex, ey + 7.0, 8.0, 0xFF0F172A, 0xFF475569, 1.8);
            }
            EnemyAnchor::Ceiling => {
                let ceiling_y = 0.0f32;
                let bracket_y = ey - 14.0;
                self.draw_rect(buffer, (ex - 9.0) as isize, ceiling_y as isize, 18, (bracket_y - ceiling_y).max(14.0) as usize, 0xFF1E293B);
                self.draw_rect(buffer, (ex - 10.0) as isize, ceiling_y as isize, 2, (bracket_y - ceiling_y).max(14.0) as usize, 0xFF475569);
                self.draw_rect(buffer, (ex + 8.0) as isize, ceiling_y as isize, 2, (bracket_y - ceiling_y).max(14.0) as usize, 0xFF475569);

                self.draw_rect(buffer, (ex - 20.0) as isize, bracket_y as isize - 8, 40, 6, 0xFFFFD700);
                self.draw_rect(buffer, (ex - 14.0) as isize, bracket_y as isize - 8, 6, 6, 0xFF111111);
                self.draw_rect(buffer, (ex + 6.0) as isize, bracket_y as isize - 8, 6, 6, 0xFF111111);

                self.draw_aa_capsule(buffer, ex - 18.0, bracket_y, ex - 7.0, ey - 4.0, 4.0, 0xFF0F172A);
                self.draw_aa_capsule(buffer, ex + 18.0, bracket_y, ex + 7.0, ey - 4.0, 4.0, 0xFF0F172A);

                self.draw_circle(buffer, ex as isize, bracket_y as isize - 10, 3, 0xFFFF0033);
                self.draw_point_light(buffer, ex as isize, bracket_y as isize - 10, 14, 0xFFFF0033, 0.6);
            }
            EnemyAnchor::LeftWall => {
                let wall_x = 0.0f32;
                let bracket_x = ex - 14.0;
                self.draw_rect(buffer, wall_x as isize, (ey - 9.0) as isize, (bracket_x - wall_x).max(12.0) as usize, 18, 0xFF1E293B);
                self.draw_aa_capsule(buffer, bracket_x, ey - 18.0, bracket_x, ey + 18.0, 4.5, 0xFF334155);
                self.draw_aa_capsule(buffer, bracket_x, ey - 14.0, ex - 4.0, ey - 6.0, 3.8, 0xFF0F172A);
                self.draw_aa_capsule(buffer, bracket_x, ey + 14.0, ex - 4.0, ey + 6.0, 3.8, 0xFF0F172A);
            }
            EnemyAnchor::RightWall => {
                let right_w = self.width as f32;
                let bracket_x = ex + 14.0;
                self.draw_rect(buffer, bracket_x as isize, (ey - 9.0) as isize, (right_w - bracket_x).max(12.0) as usize, 18, 0xFF1E293B);
                self.draw_aa_capsule(buffer, bracket_x, ey - 18.0, bracket_x, ey + 18.0, 4.5, 0xFF334155);
                self.draw_aa_capsule(buffer, bracket_x, ey - 14.0, ex + 4.0, ey - 6.0, 3.8, 0xFF0F172A);
                self.draw_aa_capsule(buffer, bracket_x, ey + 14.0, ex + 4.0, ey + 6.0, 3.8, 0xFF0F172A);
            }
            EnemyAnchor::FreeFlying => {}
        }
    }
}
