use super::*;

use iced::Vector;
use nyanko::graphics::tools::joint::Joint;


const X_FIELD: usize = 4;
const Y_FIELD: usize = 5;
const PIVOT_X_FIELD: usize = 6;
const PIVOT_Y_FIELD: usize = 7;
const SCALE_X_FIELD: usize = 8;
const SCALE_Y_FIELD: usize = 9;
const ANGLE_FIELD: usize = 10;
const OPACITY_FIELD: usize = 11;

const X_KIND: i32 = 4;
const Y_KIND: i32 = 5;
const PIVOT_X_KIND: i32 = 6;
const PIVOT_Y_KIND: i32 = 7;
const SCALE_X_KIND: i32 = 9;
const SCALE_Y_KIND: i32 = 10;
const ANGLE_KIND: i32 = 11;
const OPACITY_KIND: i32 = 12;

const TURN_PROBE: i32 = 8;

type Step = (usize, i32, f32);
type Moved = (usize, Vec<(usize, i32, i32)>);

pub(super) struct Slide {
    part: usize,
    hand: Gizmo,
    travel: (f32, f32),
    carry: [(f32, f32); 2],
    shift: [(f32, f32); 2],
    rest: [i32; 4],
    children: Vec<Tether>,
}

struct Tether {
    part: usize,
    reach: [(f32, f32); 2],
    rest: [i32; 2],
}

impl Slide {
    fn targets(&self) -> Option<Vec<Moved>> {
        let pivot = posing::split(self.shift, (-self.travel.0, -self.travel.1))?;
        let pivot = (pivot.0.round(), pivot.1.round());
        let strayed = weigh(self.shift, pivot);

        let carried = posing::split(self.carry, (-strayed.0, -strayed.1))?;
        let carried = (carried.0.round(), carried.1.round());
        let joint = weigh(self.carry, carried);

        let [x, y, pivot_x, pivot_y] = self.rest;

        let mut moved = vec![(
            self.part,
            vec![
                (X_FIELD, X_KIND, nudged(x, carried.0)),
                (Y_FIELD, Y_KIND, nudged(y, carried.1)),
                (PIVOT_X_FIELD, PIVOT_X_KIND, nudged(pivot_x, pivot.0)),
                (PIVOT_Y_FIELD, PIVOT_Y_KIND, nudged(pivot_y, pivot.1)),
            ],
        )];

        for tether in &self.children {
            let Some((across, down)) = posing::split(tether.reach, (-joint.0, -joint.1)) else {
                continue;
            };

            let [x, y] = tether.rest;

            moved.push((
                tether.part,
                vec![(X_FIELD, X_KIND, nudged(x, across.round())), (Y_FIELD, Y_KIND, nudged(y, down.round()))],
            ));
        }

        Some(moved)
    }
}

fn weigh(reach: [(f32, f32); 2], by: (f32, f32)) -> (f32, f32) {
    (reach[0].0 * by.0 + reach[1].0 * by.1, reach[0].1 * by.0 + reach[1].1 * by.1)
}

fn nudged(rest: i32, by: f32) -> i32 {
    rest.saturating_add(by as i32)
}

impl Session {
    pub(super) fn animated(&self) -> bool {
        self.viewer.animation().is_some()
    }

    pub(super) fn hand(&self, settings: &Settings) -> Gizmo {
        handed(self.animated(), &settings.studio)
    }

    pub(super) fn chosen_part(&self) -> Option<usize> {
        self.pose.as_ref().and_then(|pose| pose.part)
    }

    pub(super) fn grasp(&mut self, part: usize, grip: gizmo::Grip, hand: Gizmo) {
        self.gizmo.show(true);
        self.gizmo.seize(Some(grip));
        self.viewer.pause();
        self.drift.clear();
        self.sliding = None;

        if grip == gizmo::Grip::Rotate {
            self.winding = self.wound(part, hand).unwrap_or(1.0);
        }
    }

    pub(super) fn haul(&mut self, sweep: gizmo::Sweep, hand: Gizmo) -> Task<Message> {
        let entity = self.entity;

        let Some(part) = self.chosen_part() else {
            return Task::none();
        };

        let task = match sweep.grip {
            gizmo::Grip::Move => self.shove(part, sweep.travel, hand),
            gizmo::Grip::Scale { .. } => self.stretch(part, sweep, hand),
            gizmo::Grip::Rotate => self.spin(part, sweep.spun, hand),
            gizmo::Grip::Pivot => self.slide(part, sweep.travel, hand),
        };

        self.reposed(entity);

        task
    }

    pub(super) fn rooted(&self, part: usize) -> bool {
        self.pose.as_ref().is_some_and(|pose| pose.doc.parent(part).is_none())
    }

    pub(super) fn pinned_notice(&self) -> Option<Notice> {
        if self.mode != Mode::Entity || self.focus != Focus::Curve {
            return None;
        }

        let draft = self.draft.as_ref()?;
        let held = draft.doc.track(draft.track?)?;

        if !matches!(held.kind, X_KIND | Y_KIND) {
            return None;
        }

        let part = usize::try_from(held.part).ok()?;

        self.pinned(part).then(|| (PINNED_NOTICE.to_owned(), None))
    }

    pub(super) fn pinned(&self, part: usize) -> bool {
        let Some(row) = self.viewer.offset() else {
            return false;
        };

        self.viewer
            .rig()
            .and_then(|rig| rig.model.alignment.get(row))
            .is_some_and(|align| usize::try_from(align.part).is_ok_and(|named| named == part))
    }

    pub(super) fn tint(&mut self, step: f32, hand: Gizmo) -> Task<Message> {
        let Some(part) = self.chosen_part() else {
            return Task::none();
        };

        self.apply(part, &[(OPACITY_FIELD, OPACITY_KIND, step)], hand)
    }

    fn shove(&mut self, part: usize, travel: Vector, hand: Gizmo) -> Task<Message> {
        if hand == Gizmo::Model && self.rooted(part) {
            return Task::done(Message::Refused(ROOT_MOVE_NOTICE));
        }

        if hand == Gizmo::Channel && self.pinned(part) {
            return Task::done(Message::Refused(PINNED_NOTICE));
        }

        let Some(travel) = self.worldly(travel) else {
            return Task::none();
        };

        let Some(reach) = self.pivot_reach(part, hand) else {
            return Task::none();
        };

        let Some((across, down)) = posing::split(reach, travel) else {
            return Task::none();
        };

        self.apply(part, &[(X_FIELD, X_KIND, across), (Y_FIELD, Y_KIND, down)], hand)
    }

    fn slide(&mut self, part: usize, travel: Vector, hand: Gizmo) -> Task<Message> {
        if hand == Gizmo::Model && self.rooted(part) {
            return Task::done(Message::Refused(ROOT_PIVOT_NOTICE));
        }

        if hand == Gizmo::Channel && self.pinned(part) {
            return Task::done(Message::Refused(PINNED_NOTICE));
        }

        let Some(travel) = self.worldly(travel) else {
            return Task::none();
        };

        if !self.sliding.as_ref().is_some_and(|held| held.part == part && held.hand == hand) {
            self.sliding = self.brace(part, hand);
        }

        let Some(slide) = self.sliding.as_mut() else {
            return Task::none();
        };

        slide.travel = (slide.travel.0 + travel.0, slide.travel.1 + travel.1);

        let Some(targets) = slide.targets() else {
            return Task::none();
        };

        let moved: Vec<Moved> = targets
            .into_iter()
            .map(|(at, fields)| {
                let changed = fields
                    .into_iter()
                    .filter(|(field, kind, value)| self.held(at, *field, *kind, hand) != Some(*value))
                    .collect();

                (at, changed)
            })
            .filter(|(_, fields): &Moved| !fields.is_empty())
            .collect();

        self.commit(part, &moved, hand)
    }

    fn brace(&self, part: usize, hand: Gizmo) -> Option<Slide> {
        let carry = self.pivot_reach(part, hand)?;
        let shift = self.corner_reach(part, part, (PIVOT_X_FIELD, PIVOT_Y_FIELD), (PIVOT_X_KIND, PIVOT_Y_KIND), hand)?;
        let rest = [
            self.held(part, X_FIELD, X_KIND, hand)?,
            self.held(part, Y_FIELD, Y_KIND, hand)?,
            self.held(part, PIVOT_X_FIELD, PIVOT_X_KIND, hand)?,
            self.held(part, PIVOT_Y_FIELD, PIVOT_Y_KIND, hand)?,
        ];

        let joints = self.viewer.joints();
        let drawn: Vec<usize> = self.viewer.posed(Scope::Rig).iter().map(|posed| posed.part).collect();
        let children = joints
            .iter()
            .filter(|joint| joint.parent == Some(part))
            .filter_map(|joint| self.tether(joint.part, hand, &joints, &drawn))
            .collect();

        Some(Slide { part, hand, travel: (0.0, 0.0), carry, shift, rest, children })
    }

    fn tether(&self, child: usize, hand: Gizmo, joints: &[Joint], drawn: &[usize]) -> Option<Tether> {
        if hand == Gizmo::Channel && self.pinned(child) {
            return None;
        }

        let seen = drawn_within(child, joints, drawn)?;
        let reach = self.corner_reach(child, seen, (X_FIELD, Y_FIELD), (X_KIND, Y_KIND), hand)?;
        let rest = [self.held(child, X_FIELD, X_KIND, hand)?, self.held(child, Y_FIELD, Y_KIND, hand)?];

        Some(Tether { part: child, reach, rest })
    }

    fn stretch(&mut self, part: usize, sweep: gizmo::Sweep, hand: Gizmo) -> Task<Message> {
        let gizmo::Grip::Scale { across, down } = sweep.grip else {
            return Task::none();
        };

        let (Some(travel), Some((grabbed, anchor))) = (self.worldly(sweep.travel), sweep.grip.corners())
        else {
            return Task::none();
        };

        let spots = [posing::Spot::Corner(grabbed), posing::Spot::Corner(anchor)];

        let Some(levers) = self.reach(part, part, (SCALE_X_FIELD, SCALE_Y_FIELD), (SCALE_X_KIND, SCALE_Y_KIND), hand, &spots)
        else {
            return Task::none();
        };

        let (Some(pulled), Some(held)) = (levers.first(), levers.get(1)) else {
            return Task::none();
        };

        let apart = [lessen(pulled[0], held[0]), lessen(pulled[1], held[1])];
        let grown = match (across != 0, down != 0) {
            (true, true) => posing::split(apart, travel),
            (true, false) => posing::along(apart[0], travel).map(|step| (step, 0.0)),
            (false, true) => posing::along(apart[1], travel).map(|step| (0.0, step)),
            (false, false) => None,
        };

        let Some((widen, heighten)) = grown else {
            return Task::none();
        };

        let strayed = (
            -(held[0].0 * widen + held[1].0 * heighten),
            -(held[0].1 * widen + held[1].1 * heighten),
        );

        let settled = self
            .pivot_reach(part, hand)
            .and_then(|reach| posing::split(reach, strayed))
            .unwrap_or((0.0, 0.0));

        let steps = [
            (SCALE_X_FIELD, SCALE_X_KIND, widen),
            (SCALE_Y_FIELD, SCALE_Y_KIND, heighten),
            (X_FIELD, X_KIND, settled.0),
            (Y_FIELD, Y_KIND, settled.1),
        ];

        self.apply(part, &steps, hand)
    }

    fn spin(&mut self, part: usize, spun: f32, hand: Gizmo) -> Task<Message> {
        let unit = self.viewer.rig().map_or(3600, |rig| rig.model.angle_unit).max(1) as f32;
        let step = spun / std::f32::consts::TAU * unit * self.winding;

        self.apply(part, &[(ANGLE_FIELD, ANGLE_KIND, step)], hand)
    }

    fn wound(&self, part: usize, hand: Gizmo) -> Option<f32> {
        let spots = [posing::Spot::Corner(0), posing::Spot::Corner(3)];
        let mut probe = self.probe(part, part)?;

        let swept = match hand {
            Gizmo::Model => probe.rest_sweep(ANGLE_FIELD, TURN_PROBE, &spots),
            Gizmo::Channel => {
                let doc = &self.draft.as_ref()?.doc;
                let held = self.held(part, ANGLE_FIELD, ANGLE_KIND, hand)?;

                probe.channel_sweep(doc, ANGLE_KIND, held, TURN_PROBE, &spots)
            }
        }?;

        let found = self.viewer.posed(Scope::Rig).into_iter().find(|entry| entry.part == part)?;
        let seat = |at: usize| (found.quad[at * 2], found.quad[at * 2 + 1]);

        let turning = [0, 3]
            .into_iter()
            .zip(swept)
            .map(|(corner, reach)| {
                let arm = span(found.origin, seat(corner));

                arm.0 * reach.1 - arm.1 * reach.0
            })
            .max_by(|turn, other| turn.abs().total_cmp(&other.abs()))?;

        (turning.abs() > f32::EPSILON).then(|| turning.signum())
    }

    fn worldly(&self, travel: Vector) -> Option<(f32, f32)> {
        let (_, zoom) = self.viewer.camera();

        (zoom.abs() > f32::EPSILON).then(|| (travel.x / zoom, travel.y / zoom))
    }

    fn probe(&self, part: usize, seen: usize) -> Option<Probe> {
        let rig = self.viewer.shared_rig()?;
        let held = Probe::new(rig, self.viewer.shared_anim(), self.viewer.frame(), self.viewer.offset(), part)
            .watching(seen);

        Some(match self.placed.iter().find(|posed| posed.part == seen) {
            Some(posed) => held.seeded(posed.quad),
            None => held,
        })
    }

    fn pivot_reach(&self, part: usize, hand: Gizmo) -> Option<[(f32, f32); 2]> {
        let spots = [posing::Spot::Pivot];

        self.reach(part, part, (X_FIELD, Y_FIELD), (X_KIND, Y_KIND), hand, &spots)?.first().copied()
    }

    fn corner_reach(
        &self,
        part: usize,
        seen: usize,
        fields: (usize, usize),
        kinds: (i32, i32),
        hand: Gizmo,
    ) -> Option<[(f32, f32); 2]> {
        let spots = [posing::Spot::Corner(0)];

        self.reach(part, seen, fields, kinds, hand, &spots)?.first().copied()
    }

    fn reach(
        &self,
        part: usize,
        seen: usize,
        fields: (usize, usize),
        kinds: (i32, i32),
        hand: Gizmo,
        spots: &[posing::Spot],
    ) -> Option<Vec<[(f32, f32); 2]>> {
        let mut probe = self.probe(part, seen)?;

        match hand {
            Gizmo::Model => probe.rest_reach(fields, spots),
            Gizmo::Channel => {
                let doc = &self.draft.as_ref()?.doc;
                let held = (
                    self.held(part, fields.0, kinds.0, hand)?,
                    self.held(part, fields.1, kinds.1, hand)?,
                );

                probe.channel_reach(doc, kinds, held, spots)
            }
        }
    }

    fn held(&self, part: usize, field: usize, kind: i32, hand: Gizmo) -> Option<i32> {
        let rest = || {
            let model = self.viewer.rig().map(|rig| &rig.model);

            model.map(|model| authoring::neutral_value(kind, part, Some(model)))
        };

        match hand {
            Gizmo::Model => self.pose.as_ref()?.doc.field(part, field).or_else(rest),
            Gizmo::Channel => {
                let frame = self.viewer.frame();

                self.draft.as_ref()?.doc.posed(part, kind, frame).or_else(rest)
            }
        }
    }

    fn carry(&mut self, field: usize, step: f32) -> i32 {
        let carried = self.drift.iter().find(|(at, _)| *at == field).map_or(0.0, |(_, held)| *held);
        let wanted = step + carried;
        let taken = wanted.round();

        match self.drift.iter_mut().find(|(at, _)| *at == field) {
            Some(slot) => slot.1 = wanted - taken,
            None => self.drift.push((field, wanted - taken)),
        }

        taken as i32
    }

    fn apply(&mut self, part: usize, steps: &[Step], hand: Gizmo) -> Task<Message> {
        let moved: Vec<(usize, i32, i32)> = steps
            .iter()
            .filter(|(_, _, step)| step.is_finite() && *step != 0.0)
            .filter_map(|(field, kind, step)| {
                let taken = self.carry(*field, *step);
                let held = self.held(part, *field, *kind, hand)?;

                (taken != 0).then(|| (*field, *kind, held.saturating_add(taken)))
            })
            .collect();

        self.commit(part, &[(part, moved)], hand)
    }

    fn commit(&mut self, lead: usize, moved: &[Moved], hand: Gizmo) -> Task<Message> {
        if moved.iter().all(|(_, fields)| fields.is_empty()) {
            return Task::none();
        }

        self.remember(Tag::Gizmo(lead, hand));

        match hand {
            Gizmo::Model => self.reset_fields(lead, moved),
            Gizmo::Channel => self.key_fields(moved),
        }
    }

    fn reset_fields(&mut self, lead: usize, moved: &[Moved]) -> Task<Message> {
        let Some(pose) = self.pose.as_mut() else {
            return Task::none();
        };

        for (part, fields) in moved {
            pose.pick(*part);

            for (field, _, value) in fields {
                pose.edit(*field, &value.to_string());
            }
        }

        pose.pick(lead);

        let task = pose.persist_if_dirty();

        self.settle_pose();

        task
    }

    fn key_fields(&mut self, moved: &[Moved]) -> Task<Message> {
        let frame = self.viewer.frame();
        let fresh = self.draft.as_ref().is_some_and(|draft| {
            moved.iter().any(|(part, fields)| {
                let wanted = i32::try_from(*part).ok();

                fields.iter().any(|(_, kind, _)| wanted.and_then(|at| draft.doc.effective(at, *kind)).is_none())
            })
        });

        let model = fresh.then(|| self.viewer.rig().map(|rig| rig.model.clone())).flatten();

        let Some(draft) = self.draft.as_mut() else {
            return Task::none();
        };

        for (part, fields) in moved {
            for (_, kind, value) in fields {
                draft.backing.dirty |= draft.doc.pose(*part, *kind, frame, *value, model.as_ref());
            }
        }

        draft.retrack_clamped();

        let task = draft.persist_if_dirty();
        let updated = draft.doc.shared();

        if let Some(showing) = self.viewer.selected_anim().cloned() {
            self.viewer.adopt_anim(&showing, updated);
        }

        self.relist();

        task
    }
}

fn drawn_within(part: usize, joints: &[Joint], drawn: &[usize]) -> Option<usize> {
    let mut frontier = vec![part];

    for _ in 0..=joints.len() {
        if let Some(found) = frontier.iter().find(|at| drawn.contains(at)) {
            return Some(*found);
        }

        frontier = joints
            .iter()
            .filter(|joint| joint.parent.is_some_and(|parent| frontier.contains(&parent)))
            .map(|joint| joint.part)
            .collect();

        if frontier.is_empty() {
            return None;
        }
    }

    None
}

fn lessen(pulled: (f32, f32), held: (f32, f32)) -> (f32, f32) {
    (pulled.0 - held.0, pulled.1 - held.1)
}

fn span(from: (f32, f32), to: (f32, f32)) -> (f32, f32) {
    (to.0 - from.0, to.1 - from.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slide(travel: (f32, f32)) -> Slide {
        Slide {
            part: 2,
            hand: Gizmo::Model,
            travel,
            carry: [(1.0, 0.0), (0.0, 1.0)],
            shift: [(-1.5, 0.0), (0.0, -1.5)],
            rest: [10, 20, 30, 40],
            children: vec![Tether { part: 3, reach: [(0.0, 2.0), (-2.0, 0.0)], rest: [5, 6] }],
        }
    }

    fn values(moved: &[Moved], part: usize) -> Vec<i32> {
        moved.iter().find(|(at, _)| *at == part).map(|(_, fields)| fields.iter().map(|field| field.2).collect()).unwrap_or_default()
    }

    #[test]
    fn a_scaled_pivot_drag_holds_the_sprite_and_its_children_within_rounding() {
        // At 1.5x scale no integer pivot/X pair cancels exactly, so rounding each one on
        // its own let the sprite wander. X now cancels the rounded pivot, which caps the
        // miss at half an X unit, and the same travel always lands on the same values.
        for step in 0..200 {
            let travel = (step as f32 * 0.37, step as f32 * -0.23);
            let moved = slide(travel).targets().expect("the reaches are invertible");
            let lead = values(&moved, 2);
            let (dx, dy, dpx, dpy) = ((lead[0] - 10) as f32, (lead[1] - 20) as f32, (lead[2] - 30) as f32, (lead[3] - 40) as f32);

            let sprite = (dx - 1.5 * dpx, dy - 1.5 * dpy);

            assert!(sprite.0.abs() <= 0.5 && sprite.1.abs() <= 0.5, "step {step}: {sprite:?}");
            assert_eq!(moved, slide(travel).targets().expect("same travel, same answer"));

            let child = values(&moved, 3);
            let held = (dx - 2.0 * (child[1] - 6) as f32, dy + 2.0 * (child[0] - 5) as f32);

            assert!(held.0.abs() <= 1.0 && held.1.abs() <= 1.0, "step {step}: child strayed {held:?}");
        }
    }
}

