//! Módulo de Renderizado del Comando Exosuit Humano y Satélites Orbitales
//! Inspirado fielmente en Final Mission / S.C.A.T. (Natsume Famicom/NES)
//! y adaptado a la estética de exoesqueleto cibernético militar moderno para IA Rebellion.
//!
//! Soporta:
//! - Giro tridimensional continuo sin cortes bruscos (interpolación suave de facing_scale).
//! - Coherencia total con el movimiento: inclinación aerodinámica (pitch),
//!   postura de contra-empuje (backpedaling táctico disparando en retirada),
//!   vectorización angular de toberas de propulsión y micro-jets en talones.
//! - Exoesqueleto táctico con pistones hidráulicos, juntas mecánicas, reactor torácico Arc,
//!   casco presurizado con HUD reflectante y fusil pesado con retroceso reactivo.
//! - 4 variantes personalizadas de escuadrón (Arnold, Sigourney, Jax, Orion).

use crate::player::{Player, Satellite};
use crate::renderer::Renderer;

#[inline(always)]
fn xform(
    lx: f32,
    ly: f32,
    cos_p: f32,
    sin_p: f32,
    facing_scale: f32,
    cx: f32,
    cy: f32,
) -> (f32, f32) {
    let rx = lx * cos_p - ly * sin_p;
    let ry = lx * sin_p + ly * cos_p;
    (cx + rx * facing_scale, cy + ry)
}

impl Renderer {
    /// Renderizado del Comando Exosuit de IA Rebellion con cinemática continua estilo Final Mission
    pub fn draw_modern_commando(&self, buffer: &mut [u32], player: &Player) {
        let px = player.x;
        let py = player.y;
        let t = player.anim_timer;

        // Parpadeo de invulnerabilidad
        if player.invulnerable_timer > 0.0 && ((player.invulnerable_timer * 18.0) as usize % 2 == 0) {
            return;
        }

        // Parámetros de escala de giro y orientación
        let facing_scale = player.facing_scale.clamp(-1.0, 1.0);
        let dir_sign = if facing_scale >= 0.0 { 1.0 } else { -1.0 };
        let abs_scale = facing_scale.abs();
        let turn_factor = (1.0 - abs_scale).clamp(0.0, 1.0); // 0.0 = perfil lateral, 1.0 = frente

        // Inclinación de vuelo (pitch)
        let pitch = player.flight_pitch;
        let cos_p = pitch.cos();
        let sin_p = pitch.sin();

        // Oscilación flotante (hover bobbing)
        let hover_y = (t * 5.0).sin() * 2.0;
        let cy = py + hover_y;

        // Amortiguación y retroceso dinámico del arma
        let recoil_gun = player.recoil_anim * 5.5;
        let recoil_shoulder = player.recoil_anim * 1.8;

        // Paletas de color por jugador (Arnold P1, Sigourney P2, Jax P3, Orion P4)
        let (armor_base, armor_trim, armor_dark, visor_glow, plume_outer, plume_core) = match player.id {
            0 => (0xFF1E3A8A, 0xFFE2E8F0, 0xFF0A192F, 0xFF00F0FF, 0xFF0099FF, 0xFFE0FFFF), // Arnold: Titán Cobalto / Platino / Cian
            1 => (0xFF9F1239, 0xFFFFD700, 0xFF4C0519, 0xFFFFB703, 0xFFFF3300, 0xFFFFFBEB), // Sigourney: Carmesí / Oro / Ámbar
            2 => (0xFF065F46, 0xFF34D399, 0xFF022C22, 0xFF00FF77, 0xFF059669, 0xFFD1FAE5), // Jax: Esmeralda / Grafito / Verde
            _ => (0xFFB45309, 0xFFFDE68A, 0xFF451A03, 0xFFFFD700, 0xFFD97706, 0xFFFEF3C7), // Orion: Ámbar / Oro Solar
        };

        let xf = |lx: f32, ly: f32| xform(lx, ly, cos_p, sin_p, facing_scale, px, cy);

        // =========================================================================
        // 1. BRAZO TRASERO Y GUARDAMANOS (Capa de fondo para profundidad 3D)
        // =========================================================================
        let (far_sh_x, far_sh_y) = xf(-2.0 - recoil_shoulder * 0.5, -8.0);
        let (far_el_x, far_el_y) = xf(8.0, -1.0);
        let (far_hn_x, far_hn_y) = xf(18.0 - recoil_gun * 0.7, -2.0);
        self.draw_aa_capsule(buffer, far_sh_x, far_sh_y, far_el_x, far_el_y, 3.2, armor_dark);
        self.draw_aa_capsule(buffer, far_el_x, far_el_y, far_hn_x, far_hn_y, 2.8, armor_dark);

        // =========================================================================
        // 2. PIERNAS ARTICULADAS (Pose dinámica: vuelo extendido vs contra-empuje)
        // =========================================================================
        let backpedal = player.is_backpedaling;
        let leg_flutter = if player.jetpack_active { (t * 14.0).sin() * 2.5 } else { 0.0 };

        // Pierna trasera (Far Leg)
        let (hip_far_x, hip_far_y) = xf(-6.0, 9.0);
        let (knee_far_x, knee_far_y) = if backpedal {
            xf(0.0 + leg_flutter, 18.0) // Rodilla flexionada adelante en retirada
        } else {
            xf(-14.0 + leg_flutter, 16.0) // Pierna extendida atrás en vuelo rápido
        };
        let (foot_far_x, foot_far_y) = if backpedal {
            xf(-8.0 + leg_flutter, 25.0)
        } else {
            xf(-24.0 + leg_flutter * 0.7, 21.0)
        };
        self.draw_aa_capsule(buffer, hip_far_x, hip_far_y, knee_far_x, knee_far_y, 3.8, armor_dark);
        self.draw_aa_capsule(buffer, knee_far_x, knee_far_y, foot_far_x, foot_far_y, 3.4, armor_dark);
        // Bota trasera con refuerzo
        self.draw_aa_capsule(buffer, foot_far_x - 3.0 * dir_sign, foot_far_y, foot_far_x + 5.0 * dir_sign, foot_far_y, 2.4, 0xFF334155);

        // =========================================================================
        // 3. MOCHILA JETPACK DE DOBLE TOBERA VECTORIAL (Vectoring Thrusters)
        // =========================================================================
        let (pack_top_x, pack_top_y) = xf(-14.0, -11.0);
        let (pack_bot_x, pack_bot_y) = xf(-16.0, 8.0);
        self.draw_aa_capsule(buffer, pack_top_x, pack_top_y, pack_bot_x, pack_bot_y, 5.8, 0xFF0F172A);
        self.draw_aa_capsule(buffer, pack_top_x, pack_top_y, pack_bot_x, pack_bot_y, 3.8, armor_dark);

        // Aletas estabilizadoras superiores del jetpack
        let strake_deploy = (player.flight_thrust * 6.0).clamp(2.0, 8.0);
        let (strake_x, strake_y) = xf(-18.0 - strake_deploy, -14.0);
        self.draw_aa_capsule(buffer, pack_top_x, pack_top_y, strake_x, strake_y, 2.2, armor_trim);

        // Toberas vectoriales basculantes (Gimbaled Nozzles)
        let (noz1_x, noz1_y) = xf(-17.0, -6.0);
        let (noz2_x, noz2_y) = xf(-19.0, 5.0);

        // Dirección de la pluma de plasma opuesta al empuje
        let base_plume_len = if player.jetpack_active {
            26.0 + player.flight_thrust * 14.0
        } else {
            14.0
        };

        // Vector de salida de las toberas
        let noz_angle = if backpedal {
            // En contra-empuje el jetpack reduce potencia para dar paso a retro-jets
            std::f32::consts::PI - pitch * 0.5
        } else {
            std::f32::consts::PI - pitch * 0.9
        };
        let plume_dx = noz_angle.cos() * dir_sign;
        let plume_dy = noz_angle.sin();

        // Anillos de tobera cromados
        self.draw_aa_circle(buffer, noz1_x, noz1_y, 3.8, 0xFF0F172A, armor_trim, 1.2);
        self.draw_aa_circle(buffer, noz2_x, noz2_y, 3.8, 0xFF0F172A, armor_trim, 1.2);

        // Plumas dobles de plasma multicapa
        self.draw_plasma_plume(
            buffer,
            noz1_x,
            noz1_y,
            plume_dx,
            plume_dy,
            base_plume_len,
            5.8,
            plume_core,
            plume_outer,
        );
        self.draw_plasma_plume(
            buffer,
            noz2_x,
            noz2_y,
            plume_dx,
            plume_dy,
            base_plume_len * 0.88,
            5.0,
            plume_core,
            plume_outer,
        );

        // Micro-jets de contra-empuje en el pecho / talones al hacer backpedal (Final Mission)
        if backpedal {
            let (retro_x, retro_y) = xf(7.0, -1.0);
            self.draw_aa_capsule(buffer, retro_x, retro_y, retro_x + 9.0 * dir_sign, retro_y + 3.0, 1.8, 0xFFFFFFFF);
            self.draw_aa_capsule(buffer, retro_x, retro_y, retro_x + 12.0 * dir_sign, retro_y + 4.0, 1.0, plume_outer);
            self.draw_point_light(buffer, retro_x as isize, retro_y as isize, 14, plume_outer, 0.6);
        }

        // =========================================================================
        // 4. CORAZA TORÁCICA Y REACTOR CENTRAL ARC
        // =========================================================================
        let (torso_top_x, torso_top_y) = xf(-2.0, -8.0);
        let (torso_bot_x, torso_bot_y) = xf(1.0, 7.0);

        // Chasis del exoesqueleto (columna y placas laterales moduladas en 3D por turn_factor)
        let torso_r = 7.8 + turn_factor * 1.6;
        let torso_inner_r = 6.2 + turn_factor * 1.4;
        self.draw_aa_capsule(buffer, torso_top_x, torso_top_y, torso_bot_x, torso_bot_y, torso_r, armor_dark);
        self.draw_aa_capsule(buffer, torso_top_x, torso_top_y, torso_bot_x, torso_bot_y, torso_inner_r, armor_base);

        // Placas pectorales anguladas con filigrana
        let (pect_x, pect_y) = xf(3.0, -4.0);
        self.draw_aa_capsule(buffer, torso_top_x, torso_top_y, pect_x, pect_y, 3.2 + turn_factor * 0.8, armor_trim);

        // Reactor Arc torácico central (proyectado hacia el frente según giro)
        let (reactor_x, reactor_y) = xf(2.0, -1.0);
        let reactor_size = 3.6 + (t * 6.0).sin().abs() * 0.8;
        self.draw_aa_circle(buffer, reactor_x, reactor_y, reactor_size, 0xFFFFFFFF, visor_glow, 1.4);
        self.draw_volumetric_light(buffer, reactor_x, reactor_y, 22.0, visor_glow, 0.75);

        // Pistones hidráulicos laterales visibles en el torso
        let (piston_x1, piston_y1) = xf(-4.0, -2.0);
        let (piston_x2, piston_y2) = xf(-2.0, 5.0);
        self.draw_aa_capsule(buffer, piston_x1, piston_y1, piston_x2, piston_y2, 1.4, 0xFF94A3B8);

        // =========================================================================
        // 5. PIERNA DELANTERA (Near Leg - mayor detalle y volumen)
        // =========================================================================
        let (hip_near_x, hip_near_y) = xf(-2.0, 8.0);
        let (knee_near_x, knee_near_y) = if backpedal {
            xf(5.0 - leg_flutter, 19.0) // Rodilla delantera flexionada en táctica de absorción
        } else {
            xf(-8.0 - leg_flutter, 18.0) // Pierna extendida hacia atrás en vuelo aerodinámico
        };
        let (foot_near_x, foot_near_y) = if backpedal {
            xf(-2.0 - leg_flutter, 27.0)
        } else {
            xf(-19.0 - leg_flutter * 0.8, 24.0)
        };

        // Muslo acorazado con pistón exterior
        self.draw_aa_capsule(buffer, hip_near_x, hip_near_y, knee_near_x, knee_near_y, 4.4, armor_base);
        self.draw_aa_capsule(buffer, hip_near_x, hip_near_y, knee_near_x, knee_near_y, 2.0, armor_trim);
        // Rodillera esférica blindada
        self.draw_aa_circle(buffer, knee_near_x, knee_near_y, 3.6, armor_dark, armor_trim, 1.2);

        // Espinilla y pantorrilla con exo-varilla
        self.draw_aa_capsule(buffer, knee_near_x, knee_near_y, foot_near_x, foot_near_y, 3.8, armor_base);
        self.draw_aa_capsule(buffer, knee_near_x - 1.0 * dir_sign, knee_near_y, foot_near_x - 1.0 * dir_sign, foot_near_y, 1.6, 0xFF94A3B8);

        // Bota de combate táctica con suela y micro-tobera de talón
        self.draw_aa_capsule(buffer, foot_near_x - 4.0 * dir_sign, foot_near_y, foot_near_x + 7.0 * dir_sign, foot_near_y, 2.8, 0xFF1E293B);
        self.draw_aa_capsule(buffer, foot_near_x - 2.0 * dir_sign, foot_near_y - 1.0, foot_near_x + 6.0 * dir_sign, foot_near_y - 1.0, 1.4, armor_trim);

        // Micro-jet del talón cuando vuela rápido
        if player.jetpack_active && !backpedal {
            let heel_x = foot_near_x - 6.0 * dir_sign;
            let heel_y = foot_near_y;
            self.draw_aa_capsule(buffer, heel_x, heel_y, heel_x - 7.0 * dir_sign, heel_y + 1.0, 1.4, plume_outer);
            self.draw_aa_circle(buffer, heel_x, heel_y, 1.2, 0xFFFFFFFF, plume_core, 0.4);
        }

        // =========================================================================
        // 6. CASCO CIBERNÉTICO, VISOR HUD Y RESPIRADOR
        // =========================================================================
        let (head_x, head_y) = xf(2.0, -16.0);
        // Cúpula del casco
        self.draw_aa_circle(buffer, head_x, head_y, 7.5, armor_base, armor_dark, 1.6);
        // Cresta aerodinámica superior
        let (crest_x1, crest_y1) = xf(-2.0, -21.0);
        let (crest_x2, crest_y2) = xf(4.0, -20.0);
        self.draw_aa_capsule(buffer, crest_x1, crest_y1, crest_x2, crest_y2, 1.8, armor_trim);

        // Visor HUD táctico panorámico (se adapta a la perspectiva frontal durante el giro)
        let (visor_base_x, visor_base_y) = xf(4.5, -16.5);
        let visor_w = 4.2 * abs_scale.max(0.35);
        let (visor_end_x, visor_end_y) = (visor_base_x + visor_w * dir_sign, visor_base_y + 0.8);

        self.draw_aa_capsule(buffer, visor_base_x, visor_base_y, visor_end_x, visor_end_y, 2.8, visor_glow);
        self.draw_aa_capsule(buffer, visor_base_x + 0.8 * dir_sign, visor_base_y, visor_end_x, visor_end_y, 1.4, 0xFFFFFFFF);
        self.draw_volumetric_light(buffer, visor_end_x, visor_end_y, 20.0, visor_glow, 0.65);

        // Mascarilla respiratoria inferior de titanio
        let (jaw_x1, jaw_y1) = xf(3.0, -12.0);
        let (jaw_x2, jaw_y2) = xf(6.0, -12.0);
        self.draw_aa_capsule(buffer, jaw_x1, jaw_y1, jaw_x2, jaw_y2, 1.8, 0xFF64748B);

        // =========================================================================
        // 7. HOMBRERA / PAULDRON DELANTERA Y BRAZO
        // =========================================================================
        let (sh_x, sh_y) = xf(-1.0, -8.0);
        let pauldron_size = 4.8;
        self.draw_aa_circle(buffer, sh_x, sh_y, pauldron_size, armor_base, armor_trim, 1.6);
        self.draw_aa_circle(buffer, sh_x, sh_y, 2.2, 0xFFFFFFFF, armor_trim, 0.6);

        // Brazo delantero empuñando la culata y gatillo del fusil
        let (near_el_x, near_el_y) = xf(6.0, -2.0);
        let (near_hn_x, near_hn_y) = xf(13.0 - recoil_gun * 0.9, -1.0);
        self.draw_aa_capsule(buffer, sh_x, sh_y, near_el_x, near_el_y, 3.8, armor_dark);
        self.draw_aa_capsule(buffer, near_el_x, near_el_y, near_hn_x, near_hn_y, 3.2, armor_base);

        // =========================================================================
        // 8. FUSIL DE ASALTO PESADO ANTI-IA (Heavy Battle Rifle)
        // =========================================================================
        // El rifle se amortigua hacia atrás con recoil_gun
        let (gun_rx_x, gun_rx_y) = xf(8.0 - recoil_gun, -1.0);
        let (gun_muzzle_x, gun_muzzle_y) = xf(32.0 - recoil_gun, -1.5);

        // Solo se dibuja el fusil completo si no está en el corte exacto de giro frontal
        if abs_scale > 0.18 {
            // Receptor / Chasis del arma
            let (rec_start_x, rec_start_y) = xf(6.0 - recoil_gun, 1.0);
            let (rec_end_x, rec_end_y) = xf(18.0 - recoil_gun, 1.0);
            self.draw_aa_capsule(buffer, rec_start_x, rec_start_y, rec_end_x, rec_end_y, 3.8, 0xFF0F172A);

            // Cañón pesado estriado con camisa térmica
            let (b_start_x, b_start_y) = xf(16.0 - recoil_gun, -1.5);
            self.draw_aa_capsule(buffer, b_start_x, b_start_y, gun_muzzle_x, gun_muzzle_y, 2.8, 0xFF1E293B);
            self.draw_aa_capsule(buffer, b_start_x, b_start_y, gun_muzzle_x, gun_muzzle_y, 1.5, 0xFFCBD5E1);

            // Mira holográfica / Sensor telemétrico sobre el riel superior
            let (optic_x1, optic_y1) = xf(10.0 - recoil_gun, -5.0);
            let (optic_x2, optic_y2) = xf(16.0 - recoil_gun, -5.0);
            self.draw_aa_capsule(buffer, optic_x1, optic_y1, optic_x2, optic_y2, 1.8, 0xFF0F172A);
            self.draw_aa_circle(buffer, optic_x2, optic_y2, 2.0, 0xFFFFFFFF, visor_glow, 0.8);

            // Cargador / Tambor de munición
            let (mag_x, mag_y) = xf(11.0 - recoil_gun, 6.0);
            self.draw_aa_capsule(buffer, gun_rx_x, gun_rx_y + 1.0, mag_x, mag_y, 2.8, 0xFF334155);

            // Freno de boca militar compensador en la punta
            let (brk_top_x, brk_top_y) = xf(31.0 - recoil_gun, -4.0);
            let (brk_bot_x, brk_bot_y) = xf(31.0 - recoil_gun, 1.0);
            self.draw_aa_capsule(buffer, brk_top_x, brk_top_y, brk_bot_x, brk_bot_y, 2.2, 0xFF475569);
        }

        // Destello de disparo dinámico multicapa (Muzzle Flash)
        if player.fire_timer > 0.03 || player.recoil_anim > 0.4 {
            let (m_x, m_y) = (gun_muzzle_x + 3.0 * dir_sign, gun_muzzle_y);
            // Núcleo blanco ultra-caliente
            self.draw_aa_circle(buffer, m_x, m_y, 5.5, 0xFFFFFFFF, visor_glow, 1.6);

            // Rayos starburst en cruz
            let ray_len = 9.0;
            self.draw_aa_capsule(buffer, m_x - ray_len * dir_sign, m_y, m_x + ray_len * dir_sign, m_y, 1.4, visor_glow);
            self.draw_aa_capsule(buffer, m_x, m_y - ray_len, m_x, m_y + ray_len, 1.4, visor_glow);

            // Gases laterales del freno de boca
            self.draw_aa_capsule(buffer, m_x - 3.0 * dir_sign, m_y - 6.0, m_x - 3.0 * dir_sign, m_y + 6.0, 1.2, visor_glow);
            self.draw_volumetric_light(buffer, m_x, m_y, 32.0, visor_glow, 0.85);
        }

        // =========================================================================
        // 9. RETÍCULA HUD HOLOGRÁFICA Y SATÉLITES ORBITALES
        // =========================================================================
        let reticle_dist = 68.0;
        let reticle_x = px + dir_sign * reticle_dist;
        let reticle_y = cy + (pitch * 30.0).clamp(-20.0, 20.0);
        self.draw_holographic_reticle(buffer, reticle_x, reticle_y, 14.0, visor_glow, false);

        for sat in player.satellites.iter() {
            self.draw_modern_satellite(buffer, sat, px, py, visor_glow, t);
        }
    }

    /// Satélite táctico orbital bivalvo con escudo de plasma giratorio (Estilo Final Mission)
    pub fn draw_modern_satellite(
        &self,
        buffer: &mut [u32],
        sat: &Satellite,
        player_x: f32,
        player_y: f32,
        accent_color: u32,
        anim_time: f32,
    ) {
        let sx = player_x + sat.angle.cos() * sat.distance;
        let sy = player_y + sat.angle.sin() * sat.distance;
        let shell_rot = anim_time * 4.0;
        let is_locked = sat.is_locked;

        let shield_radius = 16.0;
        let shield_col = if is_locked { 0xFFFF0055 } else { accent_color };
        let shield_facing = sat.aim_angle;

        // Arco de barrera deflectora de plasma
        for a in -3..=3 {
            let arc_a = shield_facing + (a as f32) * 0.18;
            let arc_x = sx + arc_a.cos() * shield_radius;
            let arc_y = sy + arc_a.sin() * shield_radius;
            self.draw_aa_circle(buffer, arc_x, arc_y, 1.8, 0xFFFFFFFF, shield_col, 0.8);
        }
        self.draw_volumetric_light(
            buffer,
            sx + shield_facing.cos() * shield_radius,
            sy + shield_facing.sin() * shield_radius,
            22.0,
            shield_col,
            0.45,
        );

        // Carcasa bivalva orbital rotatoria
        let h1_x = sx + shell_rot.cos() * 5.0;
        let h1_y = sy + shell_rot.sin() * 5.0;
        let h2_x = sx - shell_rot.cos() * 5.0;
        let h2_y = sy - shell_rot.sin() * 5.0;

        self.draw_aa_capsule(buffer, h1_x - 3.0, h1_y - 2.0, h1_x + 3.0, h1_y + 2.0, 3.5, 0xFF1E293B);
        self.draw_aa_capsule(buffer, h2_x - 3.0, h2_y - 2.0, h2_x + 3.0, h2_y + 2.0, 3.5, 0xFF1E293B);
        self.draw_aa_circle(buffer, h1_x, h1_y, 2.0, 0xFFFFFFFF, shield_col, 0.5);
        self.draw_aa_circle(buffer, h2_x, h2_y, 2.0, 0xFFFFFFFF, shield_col, 0.5);

        // Núcleo de energía de antimateria
        self.draw_aa_circle(buffer, sx, sy, 3.8, 0xFFFFFFFF, shield_col, 1.0);
        self.draw_volumetric_light(buffer, sx, sy, 18.0, shield_col, 0.6);

        // Trazador láser táctico cuando está en modo fijado (Locked)
        if is_locked {
            let trace_len = 50.0;
            let tx = sx + sat.aim_angle.cos() * trace_len;
            let ty = sy + sat.aim_angle.sin() * trace_len;
            self.draw_aa_capsule(buffer, sx, sy, tx, ty, 0.9, 0xFFFF0055);
        }
    }
}
