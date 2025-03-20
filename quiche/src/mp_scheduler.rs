// // You may place this module in your library (e.g. under src/mp_scheduler.rs)
// // and add: `pub mod mp_scheduler;` in your lib.rs.

// use std::time::Duration;

// use crate::Error;
// use crate::Result;
// use crate::frame;
// use crate::packet;
// use crate::packet::ConnectionId;
// use crate::octets; // assuming your octets crate is imported via your project

// // // We assume that you already define a PathStats type in your multipath code (e.g., in path.rs)
// // // with at least the following fields:
// // #[derive(Debug, Clone)]
// // pub struct PathStats {
// //     pub delivery_rate: u64,    // bytes per second
// //     pub rtt: Duration,         // measured round-trip time
// //     // ... additional fields as needed
// // }

// /// Scheduler trait for choosing a path from a slice of PathStats.
// pub trait Scheduler {
//     /// Selects the index of the optimal path from the available ones.
//     fn select_path(&self, paths: &[PathStats]) -> Result<usize>;
// }

// /// Scheduler for data frames: chooses the path with the highest delivery rate.
// pub struct HighBandwidthScheduler;

// impl Scheduler for HighBandwidthScheduler {
//     fn select_path(&self, paths: &[PathStats]) -> Result<usize> {
//         paths
//             .iter()
//             .enumerate()
//             .max_by_key(|(_idx, stats)| stats.delivery_rate)
//             .map(|(idx, _)| idx)
//             .ok_or(Error::NoAvailablePath)
//     }
// }

// /// Scheduler for ACK frames: chooses the path with the lowest RTT.
// pub struct LowLatencyScheduler;

// impl Scheduler for LowLatencyScheduler {
//     fn select_path(&self, paths: &[PathStats]) -> Result<usize> {
//         paths
//             .iter()
//             .enumerate()
//             .min_by_key(|(_idx, stats)| stats.rtt)
//             .map(|(idx, _)| idx)
//             .ok_or(Error::NoAvailablePath)
//     }
// }

// /// AckController controls the frequency of sending ACKs.
// /// For example, you can set it to send one ACK per `ack_ratio` data packets.
// pub struct AckController {
//     pub ack_ratio: u32,
//     pub data_packet_counter: u32,
// }

// impl AckController {
//     pub fn new(ack_ratio: u32) -> Self {
//         AckController {
//             ack_ratio,
//             data_packet_counter: 0,
//         }
//     }

//     /// Increment counter and return true if an ACK should be sent.
//     pub fn should_send_ack(&mut self) -> bool {
//         self.data_packet_counter += 1;
//         if self.data_packet_counter >= self.ack_ratio {
//             self.data_packet_counter = 0;
//             true
//         } else {
//             false
//         }
//     }

//     /// Force an ACK by resetting the counter.
//     pub fn trigger_ack(&mut self) {
//         self.data_packet_counter = 0;
//     }
// }

// /// MpPacketBuilder builds packets by selecting the appropriate path using a given scheduler
// /// and then serializes a header and frame using your existing functions.
// pub struct MpPacketBuilder<'a> {
//     scheduler: &'a dyn Scheduler,
// }

// impl<'a> MpPacketBuilder<'a> {
//     pub fn new(scheduler: &'a dyn Scheduler) -> Self {
//         MpPacketBuilder { scheduler }
//     }

//     /// Build a packet that contains the given frame.
//     ///
//     /// This example uses your full definitions:
//     /// – A dummy header is constructed (using `packet::Header::to_bytes` from packet.rs).
//     /// – The frame is serialized using `frame::Frame::to_bytes`.
//     ///
//     /// In practice, you’d incorporate path-specific values (e.g. connection IDs, packet numbers)
//     /// based on the chosen path.
//     pub fn build_packet(&self, frame: &frame::Frame, paths: &[PathStats]) -> Result<Vec<u8>> {
//         // Select the path index using the scheduler.
//         let path_idx = self.scheduler.select_path(paths)?;
//         let chosen_path = &paths[path_idx];
//         // (For demonstration, we print the chosen path metrics.)
//         log::debug!(
//             "Selected path {}: delivery_rate={} bytes/s, rtt={:?}",
//             path_idx,
//             chosen_path.delivery_rate,
//             chosen_path.rtt
//         );

//         // Construct a dummy header.
//         // You will want to integrate with your multipath connection state (CID, etc.)
//         let header = packet::Header {
//             ty: packet::Type::Short, // For example, use a short header for application packets.
//             version: 0,              // Not used for short headers.
//             dcid: ConnectionId::from_vec(vec![0u8; 8]),
//             scid: ConnectionId::from_vec(vec![0u8; 8]),
//             pkt_num: 0,      // In a real implementation, use proper packet numbering.
//             pkt_num_len: 4,  // For example, a 4-byte packet number.
//             token: None,
//             versions: None,
//             key_phase: false,
//         };

//         // Create a buffer for the packet.
//         let mut buf = vec![0u8; 1500];
//         let mut octets = octets::OctetsMut::with_slice(&mut buf);

//         // Serialize the header.
//         header.to_bytes(&mut octets)?;
//         // Serialize the frame.
//         frame.to_bytes(&mut octets)?;

//         // Determine the packet length.
//         let packet_len = buf.len() - octets.cap();
//         buf.truncate(packet_len);

//         // In a full implementation you would also apply header protection and encrypt the payload
//         // (using functions from packet.rs like encrypt_pkt).

//         Ok(buf)
//     }
// }

// scheduler.rs

use std::time::Duration;
use crate::path::{Path, PathId, PathMap};
use crate::Error;

/// (Assuming you have a PathStats struct defined somewhere)
/// For example:
/// pub struct PathStats {
///     pub delivery_rate: f64, // measured in bytes/second
///     pub rtt: Duration,
/// }
/// And each Path contains a field `stats: PathStats`.

pub struct Scheduler;

impl Scheduler {
    /// Selects the path with the highest delivery rate (i.e. best for data).
    pub fn select_path_for_data(paths: &PathMap) -> Result<PathId, Error> {
        paths.iter()
            .filter(|p| p.usable()) // only consider usable paths
            .max_by(|a, b| {
                a.stats
                    .delivery_rate
                    .partial_cmp(&b.stats.delivery_rate)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|p| p.path_id())
            .ok_or(Error::UnavailablePath)
    }

    /// Selects the path with the lowest RTT (i.e. best for ACKs).
    pub fn select_path_for_ack(paths: &PathMap) -> Result<PathId, Error> {
        paths.iter()
            .filter(|p| p.usable())
            .min_by(|a, b| a.stats.rtt.cmp(&b.stats.rtt))
            .map(|p| p.path_id())
            .ok_or(Error::UnavailablePath)
    }
}
