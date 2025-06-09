**Data-ack separation multipath quiche key code files SUMMARY**
 • **Modules**:
 – `args` parses CLI into `CommonArgs`, `ClientArgs`, `ServerArgs`, exposing multipath flags like `--initial-max-path-id` and `--perform-migration` args.
 – `quiche-server` boots a UDP reactor, builds `quiche::Config`, then multiplexes outgoing packets via two output buffers (`data_out`, `ack_out`) while optional GSO + SO_TXTIME pacing is enabled quiche-server.
 – `client` owns several `mio::UdpSocket`s (one per local NIC) and drives QUIC with `send_on_path()` (single path) or `send_on_path_separate()` (data/ack split) based on a scheduler client.
 – **Patched quiche `lib.rs`** adds multipath primitives: MP_ACK frame logic, per-path congestion control, and custom path chooser `get_send_path_id_separate()`.

• **Inter-component flow**: server and client both rely on modified `quiche` core; networking layer calls into `conn.recv()` / `conn.send_on_path[_separate]`, which in turn invoke path selection and frame builders inside `lib.rs`.

• **Core responsibilities**:
 – `get_send_path_id_separate()` decides the *next* path: lowest RTT for pure ACKs, highest delivery-rate path for data, skipping standby or cwnd-starved links lib.
 – ACK handling: `enqueue_ack_frames()` builds classic ACKs, while new logic emits `MPACK{path_identifier,…}` when multiple pkt-number spaces exist, checking cwnd and avoiding infinite bursts .
 – Client scheduler `lowest_latency_scheduler_flagged()` tags each path; loop sends data on high-BW path first, then ACK-only traffic on the low-latency link via the `is_ack` flag client.
 – CLI layer exposes flow-control, congestion, and multipath knobs which propagate into `quiche::Config` inside server/client main.

• **Algorithms & structures**:
 – **Data-ACK separation**: outbound call picks path0 for stream/DATAGRAM frames, path1 for MP_ACK; separation driven by `is_ack` boolean passed down from client and by internal path picker.
 – **Path metrics**: each `Path` holds delivery_rate & rtt in its `recovery` struct; these are the keys for selection logic.
 – **MP_ACK frame**: carries ranges + `path_identifier`, enabling per-path acknowledgment without cross-contaminating congestion windows.
 – **Performance tweaks**: cwnd-aware ACK bundling, buffer-length checks before push, Linux GSO and kernel pacing toggles to minimise syscall overhead and bursts.

• **External deps & constants**: relies on `mio`, `docopt`, `itertools`, Cloudflare quiche; constants like `MAX_DATAGRAM_SIZE 1350` and `MAX_BUF_SIZE 65507` guard IO sizes



• **`recovery` module (`recovery/mod.rs`)**
 – Owns `Recovery` struct that embeds congestion control, per‐epoch loss detection, pacing and RTT tracking .
 – Maintains three `RecoveryEpoch`s (Initial, Handshake, Application) each with `sent_packets`, `loss_time`, and counters for probes & in-flight packets mod.
 – Core algorithms:
 • *ACK processing* (`detect_and_remove_acked_packets`) drains acked ranges, computes bytes/packets acknowledged, flags spurious loss mod.
 • *Loss detection* (`detect_lost_packets`, `on_loss_detection_timeout`) compares time/packet thresholds then re-queues lost frames; cwnd is adapted with Reno/Cubic via `Congestion` helper.
 • Sent-packet metadata captured in `Sent` (pkt_num, size, RTT sample hooks, flags) enabling delivery-rate estimation and pacer quantum calculation mod.
 – Tunables: `INITIAL_PACKET_THRESHOLD`, `GRANULARITY`, `MAX_PTO_PROBES_COUNT`, `LOSS_REDUCTION_FACTOR`, etc. mod.

• **`path` module (`path.rs`)**
 – `Path` holds per-link state: IDs, socket tuple, validation progress (`PathValidationState`), congestion `Recovery`, counters, and standby flag .
 – `PathMap` orchestrates all paths: insert/remove paths with capacity guard, map (local,peer) → internal ID, expose iterators, and surface events (`PathEvent::{New,Validated,Failed,Closed}`) to the app .
 – Implements utility selectors:
 • `get_active[_path_id]` returns lowest active path;
 • `pid_from_path_id`, `path_id_from_addrs` translate IDs;
 • `consider_standby_paths` & `all_available_paths_standby` drive standby logic .
 – **Validation / MTU probing**: `request_validation`, `add_challenge_sent`, and timers retry or fail after `MAX_PROBING_TIMEOUTS` path.
 – **Path status signalling**: `PathStatus` enum (Standby/Available) and queues for PATH_STATUS/PATH_ABANDON frames; advertisement via `advertise_path_status` and reception handler `on_path_status_received` .
 – Metrics exposer `stats()` aggregates rtt, cwnd, delivery_rate for schedulers and qlog path.

• **Interaction with existing multipath logic**
 – `PathMap` feeds `get_send_path_id[_separate]` (in `lib.rs`) with RTT, cwnd_available, standby flags when selecting either *data path* or *ACK path*.
 – Each `Path` embeds its own `Recovery`, meaning congestion & loss decisions are per-link, while the global `Recovery` constants ensure homogeneous behaviour.

• **Key Data Structures**
 – `Sent`: lightweight packet log used by loss/ACK algorithms.
 – `PathMap` (slab-backed) + BTree index for O(log n) address lookup.
 – VecDeque queues (`events`, `path_abandon`, `path_status_to_advertise`) for lock-free handover to connection layer.