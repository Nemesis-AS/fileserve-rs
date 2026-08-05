use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use actix_web::HttpRequest;

/// Keys tracked before a full sweep runs. Bounds memory against an attacker
/// cycling source addresses to grow the map.
const SWEEP_THRESHOLD: usize = 4096;

/// A fixed-window rate limiter keyed by caller.
///
/// Deliberately in-process and dependency-free: it exists to protect a couple
/// of expensive routes, not to be a general limiter. The real rate limit for a
/// public deployment belongs at the reverse proxy, which can reject a request
/// before it costs a worker. This is the backstop for when that is missing or
/// misconfigured.
pub struct Throttle {
    inner: Mutex<HashMap<String, Vec<Instant>>>,
    window: Duration,
    limit: u32,
}

impl Throttle {
    pub fn new(limit: u32, window: Duration) -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
            window,
            limit,
        }
    }

    /// Records an attempt for `key`. `Err` carries how long until the caller
    /// may retry, suitable for a `Retry-After` header.
    pub fn check(&self, key: &str) -> Result<(), Duration> {
        let now = Instant::now();
        let mut map = self.inner.lock().unwrap_or_else(|e| e.into_inner());

        if map.len() > SWEEP_THRESHOLD {
            map.retain(|_, hits| hits.iter().any(|hit| now.duration_since(*hit) < self.window));
        }

        let hits = map.entry(key.to_string()).or_default();
        hits.retain(|hit| now.duration_since(*hit) < self.window);

        if hits.len() as u32 >= self.limit {
            // Unwrap-free: `hits` is non-empty here, since `limit` is never 0.
            let oldest = hits.iter().min().copied().unwrap_or(now);
            return Err(self.window.saturating_sub(now.duration_since(oldest)));
        }

        hits.push(now);
        Ok(())
    }
}

/// The rate limiters shared across every worker.
///
/// Built once in `main` and handed to the app factory as `web::Data`. Building
/// these per worker would give each its own counters, and since requests
/// round-robin across workers the effective limit would be N times the
/// configured one.
pub struct Throttles {
    /// Demo account provisioning.
    pub provision: Throttle,
    /// Password login. Unlimited before now, and the most expensive endpoint in
    /// the app: it runs bcrypt on every attempt, so it is both the cheapest way
    /// to burn server CPU and the obvious target for credential guessing.
    pub login: Throttle,
}

/// The bucket a request is rate-limited under.
///
/// `trusted_hops` is how many reverse proxies we control. At zero,
/// `X-Forwarded-For` is ignored entirely, because on a directly exposed server
/// any client can set that header and pick its own bucket. Above zero we take
/// the entry our own proxy appended, counting from the right: everything to the
/// left of it is client-supplied and therefore forgeable.
///
/// Note this is deliberately not `ConnectionInfo::realip_remote_addr`, which
/// takes the *leftmost* entry and is trivially spoofed.
pub fn throttle_key(req: &HttpRequest, trusted_hops: usize) -> String {
    let ip = forwarded_ip(req, trusted_hops).or_else(|| req.peer_addr().map(|addr| addr.ip()));

    match ip {
        // A single visitor is routinely handed an entire IPv6 prefix, so
        // bucketing by full address would hand out free bypasses.
        Some(IpAddr::V6(v6)) => {
            let s = v6.segments();
            format!("v6:{:x}:{:x}:{:x}:{:x}", s[0], s[1], s[2], s[3])
        }
        Some(IpAddr::V4(v4)) => format!("v4:{v4}"),
        None => "unknown".to_string(),
    }
}

fn forwarded_ip(req: &HttpRequest, trusted_hops: usize) -> Option<IpAddr> {
    if trusted_hops == 0 {
        return None;
    }

    let raw = req.headers().get("X-Forwarded-For")?.to_str().ok()?;
    let entries: Vec<&str> = raw.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();

    let index = entries.len().checked_sub(trusted_hops)?;
    entries.get(index)?.parse().ok()
}
