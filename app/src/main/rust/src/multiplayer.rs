//! Módulo de Multijugador Bluetooth para hasta 4 Jugadores
//! Protocolo binario compacto para sincronización en tiempo real vía RFCOMM

use crate::bullet::Bullet;
use crate::player::Player;

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum PacketType {
    Ping = 0,
    JoinRequest = 1,
    JoinAccept = 2,
    PlayerInput = 3,
    PlayerSync = 4,
    FireEvent = 5,
    BombEvent = 6,
    StageSync = 7,
}

#[derive(Clone, Debug)]
pub struct MultiplayerManager {
    pub is_host: bool,
    pub local_player_id: u8,
    pub connected_players: [bool; 4],
    pub outgoing_buffer: Vec<u8>,
    pub sync_timer: f32,
    pub trigger_bomb_request: Option<u8>,
}

impl MultiplayerManager {
    pub fn new(is_host: bool, local_id: u8) -> Self {
        let mut connected = [false; 4];
        if (local_id as usize) < 4 {
            connected[local_id as usize] = true;
        }
        Self {
            is_host,
            local_player_id: local_id.min(3),
            connected_players: connected,
            outgoing_buffer: Vec::with_capacity(1024),
            sync_timer: 0.0,
            trigger_bomb_request: None,
        }
    }

    /// Configura el rol y el ID del jugador
    pub fn set_mode(&mut self, is_host: bool, local_id: u8) {
        self.is_host = is_host;
        self.local_player_id = local_id.min(3);
        self.connected_players = [false; 4];
        self.connected_players[self.local_player_id as usize] = true;
    }

    /// Codifica el estado de entrada del jugador local para enviarlo por Bluetooth
    pub fn encode_input_packet(&mut self, move_x: f32, move_y: f32, fire: bool, bomb: bool, sat_lock: bool) {
        let flags: u8 = (if fire { 1 } else { 0 })
            | (if bomb { 2 } else { 0 })
            | (if sat_lock { 4 } else { 0 });

        let mx = (move_x.clamp(-1.0, 1.0) * 127.0) as i8;
        let my = (move_y.clamp(-1.0, 1.0) * 127.0) as i8;

        self.outgoing_buffer.push(PacketType::PlayerInput as u8);
        self.outgoing_buffer.push(self.local_player_id);
        self.outgoing_buffer.push(mx as u8);
        self.outgoing_buffer.push(my as u8);
        self.outgoing_buffer.push(flags);
    }

    /// Codifica la sincronización de estado de un jugador (posición, HP, puntaje)
    pub fn encode_player_sync(&mut self, id: u8, x: f32, y: f32, hp: f32, score: u32, weapon: u8) {
        self.outgoing_buffer.push(PacketType::PlayerSync as u8);
        self.outgoing_buffer.push(id);
        let x_u16 = (x.clamp(0.0, 3000.0) as u16).to_le_bytes();
        let y_u16 = (y.clamp(0.0, 2000.0) as u16).to_le_bytes();
        self.outgoing_buffer.extend_from_slice(&x_u16);
        self.outgoing_buffer.extend_from_slice(&y_u16);
        self.outgoing_buffer.push(hp.clamp(0.0, 100.0) as u8);
        self.outgoing_buffer.push(weapon);
        self.outgoing_buffer.extend_from_slice(&score.to_le_bytes());
    }

    /// Codifica cambio de nivel (Stage)
    pub fn encode_stage_sync(&mut self, stage: u8) {
        self.outgoing_buffer.push(PacketType::StageSync as u8);
        self.outgoing_buffer.push(stage);
    }

    /// Procesa bytes entrantes recibidos desde la conexión Bluetooth RFCOMM
    pub fn process_incoming(
        &mut self,
        data: &[u8],
        players: &mut Vec<Player>,
        bullets: &mut Vec<Bullet>,
        screen_w: f32,
        screen_h: f32,
    ) {
        let mut i = 0;
        while i < data.len() {
            let p_type = data[i];
            match p_type {
                0 => {
                    i += 1;
                }
                1 => {
                    if i + 2 <= data.len() {
                        let _req_id = data[i + 1];
                        if self.is_host {
                            let mut assigned = 0;
                            for (slot, connected) in self.connected_players.iter_mut().enumerate() {
                                if !*connected && slot > 0 {
                                    *connected = true;
                                    assigned = slot as u8;
                                    break;
                                }
                            }
                            if assigned > 0 {
                                self.outgoing_buffer.push(PacketType::JoinAccept as u8);
                                self.outgoing_buffer.push(assigned);
                            }
                        }
                        i += 2;
                    } else {
                        break;
                    }
                }
                2 => {
                    if i + 2 <= data.len() {
                        let assigned_id = data[i + 1];
                        if !self.is_host && assigned_id < 4 {
                            self.local_player_id = assigned_id;
                            self.connected_players[assigned_id as usize] = true;
                        }
                        i += 2;
                    } else {
                        break;
                    }
                }
                3 => {
                    if i + 5 <= data.len() {
                        let pid = data[i + 1];
                        let mx = (data[i + 2] as i8) as f32 / 127.0;
                        let my = (data[i + 3] as i8) as f32 / 127.0;
                        let flags = data[i + 4];
                        let fire = (flags & 1) != 0;
                        let bomb = (flags & 2) != 0;
                        let sat_lock = (flags & 4) != 0;

                        if (pid as usize) < 4 && pid != self.local_player_id {
                            self.connected_players[pid as usize] = true;

                            while players.len() <= (pid as usize) {
                                let new_id = players.len() as u8;
                                let py = (100.0 + (new_id as f32) * 80.0).min(screen_h - 100.0);
                                players.push(Player::new(new_id, 160.0, py));
                            }

                            if let Some(remote_p) = players.get_mut(pid as usize) {
                                remote_p.active = true;
                                remote_p.vx = mx * remote_p.speed;
                                remote_p.vy = my * remote_p.speed;
                                remote_p.x = (remote_p.x + remote_p.vx * 0.016).clamp(40.0, screen_w - 40.0);
                                remote_p.y = (remote_p.y + remote_p.vy * 0.016).clamp(40.0, screen_h - 40.0);

                                for sat in remote_p.satellites.iter_mut() {
                                    sat.update(0.016, sat_lock);
                                }

                                if fire {
                                    remote_p.fire(bullets);
                                }

                                if bomb && remote_p.bombs > 0 {
                                    remote_p.bombs -= 1;
                                    self.trigger_bomb_request = Some(pid);
                                }
                            }
                        }
                        i += 5;
                    } else {
                        break;
                    }
                }
                4 => {
                    if i + 12 <= data.len() {
                        let pid = data[i + 1];
                        let px = u16::from_le_bytes([data[i + 2], data[i + 3]]) as f32;
                        let py = u16::from_le_bytes([data[i + 4], data[i + 5]]) as f32;
                        let hp = data[i + 6] as f32;
                        let _wp = data[i + 7];
                        let score = u32::from_le_bytes([data[i + 8], data[i + 9], data[i + 10], data[i + 11]]);

                        if (pid as usize) < 4 && pid != self.local_player_id {
                            self.connected_players[pid as usize] = true;
                            while players.len() <= (pid as usize) {
                                let new_id = players.len() as u8;
                                players.push(Player::new(new_id, px, py));
                            }

                            if let Some(remote_p) = players.get_mut(pid as usize) {
                                remote_p.active = true;
                                remote_p.x = px;
                                remote_p.y = py;
                                remote_p.health = hp;
                                remote_p.score = score;
                            }
                        }
                        i += 12;
                    } else {
                        break;
                    }
                }
                7 => {
                    if i + 2 <= data.len() {
                        i += 2;
                    } else {
                        break;
                    }
                }
                _ => {
                    i += 1;
                }
            }
        }
    }

    /// Obtiene y limpia los bytes listos para transmitir
    pub fn take_outgoing_bytes(&mut self) -> Vec<u8> {
        let bytes = self.outgoing_buffer.clone();
        self.outgoing_buffer.clear();
        bytes
    }
}
