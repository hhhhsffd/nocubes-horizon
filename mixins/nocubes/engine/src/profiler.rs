//! Rolling 256-call CPU timings, matching NoCubes' chunk and single-block profilers.
use std::{
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};
const WINDOW: usize = 256;
struct Rolling {
    times: [u64; WINDOW],
    faces: [u64; WINDOW],
    next: usize,
    len: usize,
}
impl Default for Rolling {
    fn default() -> Self {
        Self {
            times: [0; WINDOW],
            faces: [0; WINDOW],
            next: 0,
            len: 0,
        }
    }
}
impl Rolling {
    fn record(&mut self, time: Duration, faces: usize) -> bool {
        self.times[self.next] = time.as_nanos().min(u64::MAX as u128) as u64;
        self.faces[self.next] = faces as u64;
        self.next = (self.next + 1) % WINDOW;
        self.len = (self.len + 1).min(WINDOW);
        self.next == 0
    }
    fn snapshot(&self) -> (u64, u64, u64) {
        (
            self.len as u64,
            self.times.iter().copied().fold(0, u64::saturating_add),
            self.faces.iter().copied().fold(0, u64::saturating_add),
        )
    }
}
fn chunk() -> &'static Mutex<Rolling> {
    static P: OnceLock<Mutex<Rolling>> = OnceLock::new();
    P.get_or_init(Default::default)
}
fn block() -> &'static Mutex<Rolling> {
    static P: OnceLock<Mutex<Rolling>> = OnceLock::new();
    P.get_or_init(Default::default)
}
fn record(profiler: &Mutex<Rolling>, name: &str, time: Duration, faces: usize) {
    let mut p = profiler.lock().unwrap_or_else(|e| e.into_inner());
    if p.record(time, faces) {
        let (n, ns, _) = p.snapshot();
        eprintln!(
            "[NoCubes] {name}: average {} ms over the past {n} calls",
            ns as f64 / n as f64 / 1_000_000.
        );
    }
}
pub fn record_mesh(time: Duration, faces: usize) {
    record(chunk(), "Render chunk mesh", time, faces);
}
pub fn mesh_profile() -> (u64, u64, u64) {
    chunk().lock().unwrap_or_else(|e| e.into_inner()).snapshot()
}
pub fn block_mesh_profile() -> (u64, u64, u64) {
    block().lock().unwrap_or_else(|e| e.into_inner()).snapshot()
}
pub struct BlockTimer {
    start: Instant,
    faces: usize,
}
impl BlockTimer {
    pub fn set_faces(&mut self, faces: usize) {
        self.faces = faces;
    }
}
impl Drop for BlockTimer {
    fn drop(&mut self) {
        record(
            block(),
            "Render single block mesh",
            self.start.elapsed(),
            self.faces,
        );
    }
}
pub fn profile_block() -> Option<BlockTimer> {
    let c = super::config();
    (c.debug && c.debug_mesh_performance).then(|| BlockTimer {
        start: Instant::now(),
        faces: 0,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rolling_window_discards_old_samples_and_reports_each_rollover() {
        let mut p = Rolling::default();
        for i in 0..WINDOW {
            assert_eq!(p.record(Duration::from_nanos(1), 2), i == WINDOW - 1);
        }
        assert_eq!(p.snapshot(), (256, 256, 512));
        assert!(!p.record(Duration::from_nanos(1001), 3));
        assert_eq!(p.snapshot(), (256, 1256, 513));
        for _ in 1..WINDOW {
            p.record(Duration::from_nanos(1001), 3);
        }
        assert_eq!(p.snapshot(), (256, 256256, 768));
    }
}
