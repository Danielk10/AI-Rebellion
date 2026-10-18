//! Módulo de Multijugador Bluetooth para hasta 4 Jugadores
//! Protocolo binario compacto para sincronización en tiempo real vía RFCOMM

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum PacketType {
    PlayerInput = 3,
    PlayerSync = 4,
}

#[derive(Clone, Debug)]
pub struct MultiplayerManager {
    pub is_host: bool,
    pub local_player_id: u8,
    pub connected_players: [bool; 4],
    pub outgoing_buffer: Vec<u8>,
    pub sync_timer: f32,
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
            outgoing_buffer: Vec::with_capacity(512),
            sync_timer: 0.0,
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
        if self.outgoing_buffer.len() > 256 {
            self.outgoing_buffer.clear();
        }
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
        if self.outgoing_buffer.len() > 256 {
            self.outgoing_buffer.clear();
        }
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
}
