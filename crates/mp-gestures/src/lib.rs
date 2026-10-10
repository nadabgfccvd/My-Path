//! # mp-gestures
//!
//! Turns raw touchpad contact frames into high-level gestures.
//! Used on Windows (Precision Touchpad HID reports) and on Linux (evdev
//! multitouch slots) so both platforms behave identically.
//!
//! The classifier works on *touch sessions*: a session starts when the first
//! finger lands and ends when the last finger lifts. When the session ends we
//! look at duration, finger count, centroid displacement and finger spread to
//! decide what happened. Physical clicks (clickpad pressed) are reported
//! immediately.

use mp_core::{Contact, Direction, Gesture, PinchKind};
use std::collections::HashMap;

/// One snapshot of the touchpad.
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub t_ms: u64,
    pub contacts: Vec<Contact>,
    /// Clickpad physically pressed.
    pub button: bool,
}

/// Thresholds. Distances are fractions of the touchpad diagonal so the same
/// config works for any device resolution.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub pad_width: f32,
    pub pad_height: f32,
    pub tap_max_ms: u64,
    pub tap_max_move: f32,
    pub double_tap_ms: u64,
    pub swipe_min_dist: f32,
    pub pinch_min_ratio: f32,
    pub hold_min_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pad_width: 1000.0,
            pad_height: 700.0,
            tap_max_ms: 220,
            tap_max_move: 0.03,
            double_tap_ms: 350,
            swipe_min_dist: 0.12,
            pinch_min_ratio: 0.25,
            hold_min_ms: 600,
        }
    }
}

impl Config {
    pub fn with_pad_size(width: f32, height: f32) -> Self {
        Self {
            pad_width: width.max(1.0),
            pad_height: height.max(1.0),
            ..Self::default()
        }
    }
    fn diag(&self) -> f32 {
        (self.pad_width.powi(2) + self.pad_height.powi(2)).sqrt()
    }
}

#[derive(Debug, Default)]
struct Session {
    start_ms: u64,
    max_fingers: u8,
    first: HashMap<u32, (f32, f32)>,
    last: HashMap<u32, (f32, f32)>,
    /// Contacts at the moment the max finger count was first reached.
    max_start: Vec<(f32, f32)>,
    /// Contacts at the last frame that still had the max finger count.
    max_end: Vec<(f32, f32)>,
    clicked: bool,
}

/// Stateful classifier. Feed it every frame in order.
#[derive(Debug)]
pub struct Classifier {
    cfg: Config,
    session: Option<Session>,
    button_was_down: bool,
    last_tap: Option<(u64, u8)>,
}

fn centroid(points: &[(f32, f32)]) -> (f32, f32) {
    let n = points.len().max(1) as f32;
    let (sx, sy) = points
        .iter()
        .fold((0.0, 0.0), |(ax, ay), (x, y)| (ax + x, ay + y));
    (sx / n, sy / n)
}

fn spread(points: &[(f32, f32)]) -> f32 {
    let (cx, cy) = centroid(points);
    let n = points.len().max(1) as f32;
    points
        .iter()
        .map(|(x, y)| ((x - cx).powi(2) + (y - cy).powi(2)).sqrt())
        .sum::<f32>()
        / n
}

fn direction(dx: f32, dy: f32) -> Direction {
    if dx.abs() >= dy.abs() {
        if dx >= 0.0 {
            Direction::Right
        } else {
            Direction::Left
        }
    } else if dy >= 0.0 {
        Direction::Down
    } else {
        Direction::Up
    }
}

impl Classifier {
    pub fn new(cfg: Config) -> Self {
        Self {
            cfg,
            session: None,
            button_was_down: false,
            last_tap: None,
        }
    }

    pub fn config(&self) -> &Config {
        &self.cfg
    }

    /// Feed one frame, get zero or more recognised gestures.
    pub fn feed(&mut self, frame: &Frame) -> Vec<Gesture> {
        let mut out = Vec::new();
        let count = frame.contacts.len() as u8;
        let pts: Vec<(f32, f32)> = frame
            .contacts
            .iter()
            .map(|c| (c.x as f32, c.y as f32))
            .collect();

        // Physical click: report on press edge.
        if frame.button && !self.button_was_down {
            out.push(Gesture::PhysicalClick {
                fingers: count.max(1),
            });
            if let Some(s) = self.session.as_mut() {
                s.clicked = true;
            }
        }
        self.button_was_down = frame.button;

        if count > 0 {
            let s = self.session.get_or_insert_with(|| Session {
                start_ms: frame.t_ms,
                clicked: frame.button,
                ..Session::default()
            });
            for c in &frame.contacts {
                s.first.entry(c.id).or_insert((c.x as f32, c.y as f32));
                s.last.insert(c.id, (c.x as f32, c.y as f32));
            }
            if count > s.max_fingers {
                s.max_fingers = count;
                s.max_start = pts.clone();
                s.max_end = pts.clone();
            } else if count == s.max_fingers {
                s.max_end = pts.clone();
            }
        } else if let Some(s) = self.session.take()
            && let Some(g) = self.classify(&s, frame.t_ms)
        {
            out.push(g);
        }
        out
    }

    fn classify(&mut self, s: &Session, end_ms: u64) -> Option<Gesture> {
        if s.clicked || s.max_fingers == 0 {
            return None;
        }
        let n = s.max_fingers;
        let diag = self.cfg.diag();
        let duration = end_ms.saturating_sub(s.start_ms);

        let max_move = s
            .first
            .iter()
            .filter_map(|(id, (fx, fy))| {
                s.last
                    .get(id)
                    .map(|(lx, ly)| ((lx - fx).powi(2) + (ly - fy).powi(2)).sqrt())
            })
            .fold(0.0f32, f32::max)
            / diag;

        if duration <= self.cfg.tap_max_ms && max_move <= self.cfg.tap_max_move {
            if let Some((t, f)) = self.last_tap
                && f == n
                && s.start_ms.saturating_sub(t) <= self.cfg.double_tap_ms
            {
                self.last_tap = None;
                return Some(Gesture::DoubleTap { fingers: n });
            }
            self.last_tap = Some((end_ms, n));
            return Some(Gesture::Tap { fingers: n });
        }

        let (sx, sy) = centroid(&s.max_start);
        let (ex, ey) = centroid(&s.max_end);
        let (dx, dy) = (ex - sx, ey - sy);
        let disp = (dx.powi(2) + dy.powi(2)).sqrt() / diag;

        if n >= 2 {
            let s0 = spread(&s.max_start);
            let s1 = spread(&s.max_end);
            if s0 > 0.0 {
                let ratio = s1 / s0;
                let spread_change = (s1 - s0).abs() / diag;
                if (ratio - 1.0).abs() >= self.cfg.pinch_min_ratio && spread_change >= disp {
                    let kind = if ratio > 1.0 {
                        PinchKind::Out
                    } else {
                        PinchKind::In
                    };
                    return Some(Gesture::Pinch { fingers: n, kind });
                }
            }
        }

        if disp >= self.cfg.swipe_min_dist && n >= 2 {
            let dir = direction(dx, dy);
            return Some(if n == 2 {
                Gesture::Scroll {
                    fingers: 2,
                    direction: dir,
                }
            } else {
                Gesture::Swipe {
                    fingers: n,
                    direction: dir,
                }
            });
        }

        if duration >= self.cfg.hold_min_ms && max_move <= self.cfg.tap_max_move * 2.0 {
            return Some(Gesture::Hold { fingers: n });
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(id: u32, x: i32, y: i32) -> Contact {
        Contact { id, x, y }
    }
    fn f(t: u64, contacts: Vec<Contact>) -> Frame {
        Frame {
            t_ms: t,
            contacts,
            button: false,
        }
    }
    fn run(frames: Vec<Frame>) -> Vec<Gesture> {
        let mut cl = Classifier::new(Config::default());
        frames.iter().flat_map(|fr| cl.feed(fr)).collect()
    }
    /// Moves `n` fingers from (x0,y0) by (dx,dy) over `steps` frames.
    fn slide(n: u32, x0: i32, y0: i32, dx: i32, dy: i32, t0: u64, steps: u64) -> Vec<Frame> {
        let mut v = Vec::new();
        for i in 0..=steps {
            let k = i as i32;
            let s = steps as i32;
            let cs = (0..n)
                .map(|id| c(id, x0 + id as i32 * 80 + dx * k / s, y0 + dy * k / s))
                .collect();
            v.push(f(t0 + i * 20, cs));
        }
        v.push(f(t0 + steps * 20 + 20, vec![]));
        v
    }

    #[test]
    fn tap_one_two_three_fingers() {
        for n in 1..=3u32 {
            let cs: Vec<Contact> = (0..n).map(|i| c(i, 300 + i as i32 * 80, 300)).collect();
            let g = run(vec![f(0, cs.clone()), f(80, cs), f(120, vec![])]);
            assert_eq!(g, vec![Gesture::Tap { fingers: n as u8 }]);
        }
    }

    #[test]
    fn double_tap() {
        let cs = vec![c(0, 300, 300)];
        let g = run(vec![
            f(0, cs.clone()),
            f(90, vec![]),
            f(200, cs.clone()),
            f(280, vec![]),
        ]);
        assert_eq!(
            g,
            vec![
                Gesture::Tap { fingers: 1 },
                Gesture::DoubleTap { fingers: 1 }
            ]
        );
    }

    #[test]
    fn three_finger_swipe_up() {
        let g = run(slide(3, 300, 600, 0, -400, 0, 15));
        assert_eq!(
            g,
            vec![Gesture::Swipe {
                fingers: 3,
                direction: Direction::Up
            }]
        );
    }

    #[test]
    fn four_finger_swipe_left() {
        let g = run(slide(4, 600, 300, -450, 0, 0, 15));
        assert_eq!(
            g,
            vec![Gesture::Swipe {
                fingers: 4,
                direction: Direction::Left
            }]
        );
    }

    #[test]
    fn two_finger_scroll_down() {
        let g = run(slide(2, 400, 100, 0, 350, 0, 15));
        assert_eq!(
            g,
            vec![Gesture::Scroll {
                fingers: 2,
                direction: Direction::Down
            }]
        );
    }

    #[test]
    fn pinch_out_and_in() {
        let mut out = Vec::new();
        for i in 0..=10 {
            let d = 40 + i * 25;
            out.push(f(
                i as u64 * 20,
                vec![c(0, 500 - d, 350), c(1, 500 + d, 350)],
            ));
        }
        out.push(f(240, vec![]));
        assert_eq!(
            run(out),
            vec![Gesture::Pinch {
                fingers: 2,
                kind: PinchKind::Out
            }]
        );

        let mut inn = Vec::new();
        for i in 0..=10 {
            let d = 300 - i * 25;
            inn.push(f(
                i as u64 * 20,
                vec![c(0, 500 - d, 350), c(1, 500 + d, 350)],
            ));
        }
        inn.push(f(240, vec![]));
        assert_eq!(
            run(inn),
            vec![Gesture::Pinch {
                fingers: 2,
                kind: PinchKind::In
            }]
        );
    }

    #[test]
    fn physical_click_is_not_a_tap() {
        let cs = vec![c(0, 300, 600)];
        let mut cl = Classifier::new(Config::default());
        let mut g = Vec::new();
        g.extend(cl.feed(&f(0, cs.clone())));
        g.extend(cl.feed(&Frame {
            t_ms: 30,
            contacts: cs.clone(),
            button: true,
        }));
        g.extend(cl.feed(&Frame {
            t_ms: 90,
            contacts: cs,
            button: false,
        }));
        g.extend(cl.feed(&f(120, vec![])));
        assert_eq!(g, vec![Gesture::PhysicalClick { fingers: 1 }]);
    }

    #[test]
    fn hold_two_fingers() {
        let cs = vec![c(0, 300, 300), c(1, 380, 300)];
        let g = run(vec![
            f(0, cs.clone()),
            f(400, cs.clone()),
            f(800, cs),
            f(820, vec![]),
        ]);
        assert_eq!(g, vec![Gesture::Hold { fingers: 2 }]);
    }

    #[test]
    fn one_finger_pointer_motion_is_ignored() {
        let g = run(slide(1, 100, 100, 500, 0, 0, 15));
        assert!(g.is_empty());
    }
}
