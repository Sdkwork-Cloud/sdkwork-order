//! Bounded reads for order detail projections (PAGINATION_SPEC / OOM guard).

/// Maximum line items returned per order detail without a dedicated paginated API.
pub const MAX_ORDER_LINE_ITEMS: i64 = 500;
