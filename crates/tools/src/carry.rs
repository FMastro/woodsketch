//! Carrying a drawing in progress out of a group, for Esc on a group that
//! [`crate::Tools::open_group_to_draw_on`] opened by itself.
//!
//! A drawing tool holds its anchors in the coordinates of the container being drawn in, so
//! stepping back out of the group means saying every one of them again in the container outside.
//! [`Carry`] is that one hop, so each tool says which of its fields are places, which are
//! directions and which are lengths, and the arithmetic lives here rather than seven times over.
//!
//! What a tool remembers from hovering -- the snap under the cursor, the points it passed over,
//! the shape it drew a moment ago -- is dropped instead of carried. All of it is in the old
//! container's coordinates and all of it is answered again by the next mouse move; carrying it
//! would mean drawing a snap marker inches away from the cursor until then, since Esc doesn't
//! move the mouse.

use glam::{DAffine3, DVec3};

/// How far from square, and from each other in length, the hop's three axes may be and still
/// count as a similarity: a thousandth, so a group scaled to 1.001 by hand still carries and one
/// scaled to 1.01 doesn't.
const SIMILAR: f64 = 1e-3;

/// One hop out of a group, mapping its coordinates to the container drawn in now.
///
/// Only a **similarity that keeps its hand** -- a turn, a move and one scale the same way along
/// all three axes -- can be carried, which is what [`Carry::out_of`] checks. Two groups it turns
/// down:
///
/// - **Scaled unevenly** (the Scale tool writes a group's factors into its own transform): the
///   Rectangle tool's `u` and `v` come out neither one inch long nor square to each other, so
///   what is a rectangle inside the group is not one outside it, and there is nothing faithful
///   to carry.
/// - **Mirrored** (Edit > Flip Along, or a mirrored component read from a `.skp`): an arc's
///   sweep and a rotated rectangle's "how far across" are signed about a normal, and a mirror
///   turns both over, so the numbers would quietly mean the other thing.
///
/// The caller cancels the drawing and closes the group in those cases, which is what Push/Pull
/// and Offset do anyway.
///
/// Within a similarity a normal and a direction move the same way: the inverse transpose a
/// normal would otherwise need is `M^-T = R/s` for `M = sR`, which points where `M n` does, and
/// both are made unit length again here.
pub(crate) struct Carry {
    hop: DAffine3,
    /// The one scale, for lengths a tool holds as a number rather than as two places.
    scale: f64,
}

impl Carry {
    /// The hop, if it's one a drawing can be carried along; `None` if it isn't a similarity that
    /// keeps its hand.
    pub fn out_of(hop: DAffine3) -> Option<Self> {
        let m = hop.matrix3;
        let axes = [m.x_axis, m.y_axis, m.z_axis];
        let scale = axes[0].length();
        if scale < f64::EPSILON || m.determinant() <= 0.0 {
            return None;
        }
        let same_size = axes.iter().all(|a| ((a.length() - scale) / scale).abs() < SIMILAR);
        let square = [(0, 1), (1, 2), (2, 0)].iter().all(|&(i, j)| axes[i].dot(axes[j]).abs() < SIMILAR * scale * scale);
        (same_size && square).then_some(Self { hop, scale })
    }

    /// An anchor: what the same place in space is called outside the group.
    pub fn point(&self, p: DVec3) -> DVec3 {
        self.hop.transform_point3(p)
    }

    /// A unit direction, still unit.
    pub fn direction(&self, d: DVec3) -> DVec3 {
        self.hop.transform_vector3(d).try_normalize().unwrap_or(d)
    }

    /// A face's normal. The same arithmetic as [`Self::direction`] for a similarity, and named
    /// apart because a tool should say what it holds, not how to move it.
    pub fn normal(&self, n: DVec3) -> DVec3 {
        self.direction(n)
    }

    /// A length the tool holds as a number (a radius, a bulge, how far across), which grows with
    /// the group's scale so it still measures the same distance in the drawing.
    pub fn length(&self, l: f64) -> f64 {
        l * self.scale
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{DMat3, DQuat};
    use std::f64::consts::FRAC_PI_4;

    /// A group turned and moved: anchors land where they stood in space, directions stay unit,
    /// lengths are unchanged.
    #[test]
    fn a_turned_group_carries_its_drawing_unchanged_in_space() {
        let hop = DAffine3::from_rotation_translation(DQuat::from_rotation_z(FRAC_PI_4), DVec3::new(3.0, 4.0, 5.0));
        let carry = Carry::out_of(hop).expect("a turn and a move is a similarity");
        let p = DVec3::new(1.0, 2.0, 3.0);
        assert!(carry.point(p).abs_diff_eq(hop.transform_point3(p), 1e-12), "the anchor is the same place");
        let d = carry.direction(DVec3::X);
        assert!((d.length() - 1.0).abs() < 1e-12, "{d} is still unit");
        assert!(d.abs_diff_eq(DVec3::new(FRAC_PI_4.cos(), FRAC_PI_4.sin(), 0.0), 1e-12), "{d}");
        assert!((carry.length(2.5) - 2.5).abs() < 1e-12, "nothing was scaled");
    }

    /// An evenly scaled group carries, and the ruler the tool measures with stays unit: measuring
    /// with a doubled ruler is what made a 1" x 1" rectangle in a doubled group come out 4" x 1".
    #[test]
    fn an_evenly_scaled_group_leaves_the_tools_ruler_unit() {
        let carry = Carry::out_of(DAffine3::from_scale(DVec3::splat(2.0))).expect("one scale all ways is a similarity");
        assert!(carry.point(DVec3::X).abs_diff_eq(DVec3::X * 2.0, 1e-12), "the anchor doubles with the group");
        for d in [DVec3::X, DVec3::Y, DVec3::Z, DVec3::new(1.0, 1.0, 0.0).normalize()] {
            let carried = carry.direction(d);
            assert!((carried.length() - 1.0).abs() < 1e-12, "{carried} is still unit");
            assert!(carried.abs_diff_eq(d, 1e-12), "and still points the same way: {carried}");
        }
        assert!((carry.length(3.0) - 6.0).abs() < 1e-12, "a radius inside the group is twice that outside");
    }

    /// A group scaled unevenly has no hop: 4" one way and 1" the other isn't a scale of the
    /// rectangle, it's a different rectangle.
    #[test]
    fn an_unevenly_scaled_group_has_no_hop_to_carry_along() {
        assert!(Carry::out_of(DAffine3::from_scale(DVec3::new(4.0, 1.0, 1.0))).is_none());
    }

    /// Nor does a sheared one, where `u` and `v` would stop being square to each other.
    #[test]
    fn a_sheared_group_has_no_hop_either() {
        let shear = DMat3::from_cols(DVec3::X, DVec3::new(0.5, 1.0, 0.0), DVec3::Z);
        assert!(Carry::out_of(DAffine3::from_mat3(shear)).is_none());
    }

    /// Nor a mirrored one: an arc's sweep is signed about its normal, and a mirror turns the sign
    /// over without the number saying so.
    #[test]
    fn a_mirrored_group_has_no_hop() {
        assert!(Carry::out_of(DAffine3::from_scale(DVec3::new(-1.0, 1.0, 1.0))).is_none());
    }

    /// A group flattened to nothing (or a transform that lost a scale) carries nothing.
    #[test]
    fn a_collapsed_group_has_no_hop() {
        assert!(Carry::out_of(DAffine3::from_scale(DVec3::ZERO)).is_none());
        assert!(Carry::out_of(DAffine3::from_scale(DVec3::new(1.0, 1.0, 0.0))).is_none());
    }
}
