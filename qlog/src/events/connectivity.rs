// Copyright (C) 2021, Cloudflare, Inc.
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
//     * Redistributions of source code must retain the above copyright notice,
//       this list of conditions and the following disclaimer.
//
//     * Redistributions in binary form must reproduce the above copyright
//       notice, this list of conditions and the following disclaimer in the
//       documentation and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS
// IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO,
// THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
// PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR
// CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
// EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
// PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use serde::Deserialize;
use serde::Serialize;

use super::ApplicationErrorCode;
use super::Bytes;
use super::ConnectionErrorCode;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum TransportOwner {
    Local,
    Remote,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionState {
    Attempted,
    PeerValidated,
    HandshakeStarted,
    EarlyWrite,
    HandshakeCompleted,
    HandshakeConfirmed,
    Closing,
    Draining,
    Closed,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ConnectivityEventType {
    ServerListening,
    ConnectionStarted,
    ConnectionClosed,
    ConnectionIdUpdated,
    SpinBitUpdated,
    ConnectionStateUpdated,
    MtuUpdated,
    CubicStateUpdate,
    BbrStateUpdate,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionClosedTrigger {
    Clean,
    HandshakeTimeout,
    IdleTimeout,
    Error,
    StatelessReset,
    VersionMismatch,
    Application,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct ServerListening {
    pub ip_v4: Option<String>, // human-readable or bytes
    pub ip_v6: Option<String>, // human-readable or bytes
    pub port_v4: Option<u16>,
    pub port_v6: Option<u16>,

    retry_required: Option<bool>,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct ConnectionStarted {
    pub ip_version: Option<String>, // "v4" or "v6"
    pub src_ip: String,             // human-readable or bytes
    pub dst_ip: String,             // human-readable or bytes

    pub protocol: Option<String>,
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,

    pub src_cid: Option<Bytes>,
    pub dst_cid: Option<Bytes>,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct ConnectionClosed {
    pub owner: Option<TransportOwner>,

    pub connection_code: Option<ConnectionErrorCode>,
    pub application_code: Option<ApplicationErrorCode>,
    pub internal_code: Option<u32>,

    pub reason: Option<String>,

    pub trigger: Option<ConnectionClosedTrigger>,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct ConnectionIdUpdated {
    pub owner: Option<TransportOwner>,

    pub old: Option<Bytes>,
    pub new: Option<Bytes>,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct SpinBitUpdated {
    pub state: bool,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct ConnectionStateUpdated {
    pub old: Option<ConnectionState>,
    pub new: ConnectionState,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct MtuUpdated {
    pub old: Option<u16>,
    pub new: u16,
    pub done: Option<bool>,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct CubicStateUpdateData {
    pub cwnd: u64,
    pub ssthresh: u64,
    pub w_max: u64,
    pub hystart_window_end: Option<u64>,
    pub hystart_last_min_rtt: Option<u128>,
    pub hystart_current_min_rtt: Option<u128>,
    // pub hystart_rtt_threshold: Option<u64>,
    pub hystart_rtt_sample_count: Option<usize>,
    pub hystart_in_css: Option<bool>,
    pub hystart_css_round_count: Option<usize>,

}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct CubicStateUpdate {
    pub path_id: u64,
    pub old_state: String,
    pub new_state: String,
    pub trigger: String,
    pub data: CubicStateUpdateData,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct BbrStateUpdateData {
    pub cwnd: u64,
    pub pacing_rate: u64,
    pub btlbw: u64,
    pub rtprop_us: u128,
    pub pacing_gain: f64,
    pub cwnd_gain: f64,
    pub filled_pipe: bool,
    pub round_count: u64,
    pub cycle_index: Option<usize>,
    pub bytes_in_flight: Option<u64>,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct BbrStateUpdate {
    pub path_id: u64,
    pub old_state: String,
    pub new_state: String,
    pub trigger: String,
    pub data: BbrStateUpdateData,
}
