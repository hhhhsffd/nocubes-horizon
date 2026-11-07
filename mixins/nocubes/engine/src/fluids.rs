//! Source-fluid extension used by rendering and entity fluid queries.
use super::Pos;
pub fn extended<T: Copy>(
    p: Pos,
    range: u8,
    smooth: impl Fn(Pos) -> bool,
    empty: impl Fn(T) -> bool,
    source: impl Fn(T) -> bool,
    raw: impl Fn(Pos) -> T,
) -> T {
    let own = raw(p);
    if range == 0 || !empty(own) || !smooth(p) {
        return own;
    }
    let above = raw([p[0], p[1] + 1, p[2]]);
    if source(above) {
        return above;
    }
    let n = i32::from(range);
    for z in p[2] - n..=p[2] + n {
        for x in p[0] - n..=p[0] + n {
            if x == p[0] && z == p[2] {
                continue;
            }
            let f = raw([x, p[1], z]);
            if source(f) {
                return f;
            }
        }
    }
    own
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_above_precedes_neighbors_and_flow_does_not_extend() {
        let raw = |p| {
            if p == [0, 1, 0] {
                2
            } else if p == [-1, 0, -1] {
                3
            } else {
                0
            }
        };
        assert_eq!(
            extended([0; 3], 1, |_| true, |f| f == 0, |f| f >= 2, raw),
            2
        );
        assert_eq!(
            extended([0; 3], 1, |_| false, |f| f == 0, |f| f >= 2, raw),
            0
        );
        assert_eq!(
            extended([0; 3], 1, |_| true, |f| f == 0, |f| f >= 2, |_| 1),
            1
        );
    }
    #[test]
    fn extension_crosses_negative_chunk_boundaries() {
        let raw = |p| u8::from(p == [-17, 0, 0]);
        assert_eq!(
            extended([-16, 0, 0], 1, |_| true, |f| f == 0, |f| f == 1, raw),
            1
        );
    }
}
