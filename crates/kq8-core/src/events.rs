//! The event queue behind `sendEvent` (M3 step 3a-2).
//!
//! Rule from `docs/behaviour/doors.md` "Commands door scripts use"
//! (traced): `sendEvent <delay> <object> <type> [data…]` sends the event
//! after `<delay>` seconds of game time; `0` means the next time the queue
//! is handled, after events already waiting for "now"; at the same time,
//! the first sent is handled first.

/// An event waiting to be handled.
#[derive(Debug, Clone, PartialEq)]
pub struct Pending {
    /// Game time (seconds) it is due.
    pub due: f64,
    /// Send order, for events due at the same time.
    pub seq: u64,
    pub target: String,
    pub kind: String,
    pub data: Vec<String>,
    /// Script line that sent it (for the log).
    pub from: String,
}

#[derive(Debug, Clone, Default)]
pub struct EventQueue {
    waiting: Vec<Pending>,
    next_seq: u64,
}

impl EventQueue {
    /// Queues an event `delay` seconds after `now`.
    pub fn send(&mut self, now: f64, delay: f64, target: &str, kind: &str, data: &[String], from: &str) {
        self.waiting.push(Pending {
            due: now + delay.max(0.0),
            seq: self.next_seq,
            target: target.to_string(),
            kind: kind.to_string(),
            data: data.to_vec(),
            from: from.to_string(),
        });
        self.next_seq += 1;
    }

    /// The next event due by `now`: earliest time first, then send order.
    pub fn take_due(&mut self, now: f64) -> Option<Pending> {
        // A small allowance so 0.07 s after one update is not missed by
        // rounding when it falls exactly on a later update's time.
        let i = self
            .waiting
            .iter()
            .enumerate()
            .filter(|(_, e)| e.due <= now + 1e-9)
            .min_by(|(_, a), (_, b)| a.due.total_cmp(&b.due).then(a.seq.cmp(&b.seq)))
            .map(|(i, _)| i)?;
        Some(self.waiting.remove(i))
    }

    pub fn len(&self) -> usize {
        self.waiting.len()
    }

    pub fn is_empty(&self) -> bool {
        self.waiting.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_order_then_send_order() {
        let mut q = EventQueue::default();
        q.send(1.0, 0.5, "late", "KQDoorEvent", &[], "");
        q.send(1.0, 0.0, "first", "KQDoorEvent", &[], "");
        q.send(1.0, 0.0, "second", "KQDoorEvent", &[], "");
        q.send(1.0, 0.07, "soon", "KQCueEvent", &["1".into(), "0".into()], "");
        let names = |q: &mut EventQueue, now| std::iter::from_fn(|| q.take_due(now)).map(|e| e.target).collect::<Vec<_>>();
        assert_eq!(names(&mut q, 1.0), ["first", "second"]);
        // 0.07 s later: not after one update (1/15 s), but after two.
        assert!(names(&mut q, 1.0 + 1.0 / 15.0).is_empty());
        assert_eq!(names(&mut q, 1.0 + 2.0 / 15.0), ["soon"]);
        assert_eq!(names(&mut q, 2.0), ["late"]);
        assert!(q.is_empty());
    }
}
