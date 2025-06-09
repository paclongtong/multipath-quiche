# Analysis of Data-ACK Separation in Current Multipath QUIC

## Core Issues with My Implementation

### 1. **Congestion Control Imbalance**

My approach fundamentally breaks QUIC's congestion control assumptions:

- **Path 0 (Data)**: 200ms RTT, 100Mbps - carries all stream data
- **Path 1 (ACKs)**: 5ms RTT, 5Mbps - carries only acknowledgments

The congestion window grows based on ACK reception, but ACKs arrive on a completely different path with different characteristics. This creates several problems:

- **Delayed congestion signals**: Path 0's congestion isn't reflected in Path 1's ACK delivery
- **Mismatched RTT calculations**: ACKs arrive faster than they should for the data path
- **Window scaling issues**: The sender may over-estimate available bandwidth

### 2. **Bandwidth Underutilization**

From my observations:

- Less average bytes in flight
- Slower flow control window growth
- Underutilized Path 1 bandwidth

This suggests the separation is actually **throttling** performance rather than optimizing it.

### 3. **Code-Level Issues**

Looking at my scheduler implementation:

rust

```rust
let res = client.conn.send_on_path_separate(
    &mut buffer_slice[..max_write_this_call],
    Some(*local_addr),
    Some(*peer_addr),   
    &mut Some(*is_ack),  // This forces ACK-only on path 1
);
```

The rigid separation means:

- Path 1 can only send ACKs (severely bandwidth limited)
- Path 0 must handle all data (becomes a bottleneck)
- No adaptive load balancing based on actual path conditions

## Why the "Express ACK" Theory Fails

### 1. **ACK Compression Effect**

In real networks, ACKs naturally get compressed and delayed. Myseparation:

- Artificially accelerates ACK delivery on Path 1
- Creates unrealistic feedback loops
- May trigger false fast recovery mechanisms

### 2. **Head-of-Line Blocking**

When Path 0 experiences congestion:

- Data transmission stalls
- But ACKs continue flowing freely on Path 1
- This creates a feedback mismatch that confuses congestion control

### 3. **Resource Waste**

Path 1's 5Mbps capacity is severely underutilized:

- ACKs are typically <100 bytes each
- Even at high data rates, ACK traffic rarely exceeds 1-2% of data traffic
- You're wasting 98% of Path 1's bandwidth

## Better Approaches

### 1. **Adaptive Load Balancing**

Instead of rigid separation, consider:

rust

```rust
// Pseudo-code for better scheduling
fn adaptive_scheduler(conn: &Connection) -> PathSelection {
    let path0_congested = path0.cwnd_utilization() > 0.8;
    let path1_available = path1.available_bandwidth() > threshold;
    
    if path0_congested && path1_available {
        // Send some data on path1 to relieve pressure
        return PathSelection::Balanced;
    }
    
    // Default to lowest-latency for ACKs, highest-bandwidth for data
    PathSelection::RttBased
}
```

### 2. **Priority-Based Scheduling**

- **High Priority**: ACKs, control frames → lowest latency path
- **Medium Priority**: Small data frames → balanced across paths
- **Low Priority**: Bulk data → highest bandwidth path

### 3. **Congestion-Aware Separation**

- Monitor congestion on both paths
- Allow ACKs on data path when ACK path is congested
- Allow data spillover to ACK path when data path is saturated

## Recommendations

### Immediate Fixes:

1. **Remove rigid separation**: Allow data on Path 1 when Path 0 is congested
2. **Fix congestion control**: Ensure RTT calculations account for path-specific delays
3. **Implement overflow handling**: When Path 1 has spare capacity, use it for data

### Long-term Improvements:

1. **Implement proper path-aware congestion control**
2. **Add dynamic scheduling based on real-time path conditions**
3. **Consider packet-level scheduling instead of frame-type separation**

## Conclusion

Myperformance degradation strongly suggests that **rigid data-ACK separation is counterproductive** in multipath QUIC. The approach:

- Wastes available bandwidth on the ACK path
- Creates artificial bottlenecks on the data path
- Breaks fundamental congestion control assumptions
- Introduces feedback loops that hurt rather than help performance

The "express ACK" concept might work in theory, but in practice, the downsides (bandwidth waste, congestion control confusion, artificial constraints) outweigh any potential benefits.

Consider moving toward **adaptive, congestion-aware scheduling** that can dynamically choose the best path for each packet based on current network conditions rather than rigid frame-type rules.