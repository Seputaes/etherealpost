use crate::wow::data_tables::Db2CurvePoints;
use std::collections::HashMap;

/// Item Level Curve, which loosely corresponds to the
/// `CurvePoint` DB2 table in World of Warcraft, can be used to determine
/// the effective item level of an item based on the level the player was
/// when they looted it by applying
/// <a href="https://en.wikipedia.org/wiki/Linear_interpolation" target="_blank">linear interpolation</a>.
///
/// An Item Level Curve consists of a vector of `(u32, u32)` data points which
/// represent `(x, y)` coordinates on a graph, where `x` is player level and
/// `y` is the effective item level of the item. Using interpolation,
/// on this data set, we can determine what `y` should be for any given `x`
/// which in this case is the player's level when the item dropped.
///
/// In the [Item](`crate::battlenet::auctions::Item`) on an Auction, the player's level
/// when the item dropped can be found in the
/// [modifiers](`crate::battlenet::auctions::Item::modifiers`) field for the modifier
/// with a type of `9`. If this is not set, the item level falls back on the base
/// level of the item.
///
/// See [`ItemLevelCurve::calc_ilvl`] for a detailed explanation of the formula used.
pub struct ItemLevelCurve {
    /// The curve points (`(x,y)` coordinates) which are associated with this item
    /// level curve.
    points: Vec<CurvePoint>,
}

impl ItemLevelCurve {
    /// Create a new Item Level curve from an array of `(x, y)` coordinates.
    /// These values will be cloned into the resulting struct. As such,
    /// they do not need to be mutable or previously sorted.
    ///
    /// The curve points can be found for a given _curve ID_ in the `CurvePoint`
    /// DB2 table in World of Warcraft.
    pub fn from_points(points: &[(f64, f64)]) -> ItemLevelCurve {
        let mut curve = ItemLevelCurve {
            points: points
                .iter()
                .map(|(x, y)| CurvePoint {
                    player_level: *x,
                    item_level: *y,
                })
                .collect(),
        };
        curve
            .points
            .sort_by(|a, b| a.player_level.partial_cmp(&b.player_level).unwrap());
        curve
    }

    /// Maps an the entire [Db2CurvePoints](`crate::wow::data_tables::Db2CurvePoints`) table
    /// into a mapping of curve IDs to an [`ItemLevelCurve`] wrapping the
    /// curve coordinates.
    pub fn for_whole_table(table: &Db2CurvePoints) -> HashMap<u32, ItemLevelCurve> {
        table
            .curve_ids
            .iter()
            .map(|(curve_id, points)| (*curve_id, ItemLevelCurve::from_points(&points)))
            .collect()
    }

    /// Create a new Item Level Curve for a `curve_id`.
    ///
    /// The curve will be looked up in the provided
    /// [Db2CurvePoints](`crate::wow::data_tables::Db2CurvePoints`) table
    /// and those curve points used, if they exist.
    ///
    pub fn from_table(curve_id: &u32, table: &Db2CurvePoints) -> Option<ItemLevelCurve> {
        table
            .points(&curve_id)
            .map(|curve_points| ItemLevelCurve::from_points(&curve_points))
    }

    /// Using the item level curve points, calculates the effective item level
    /// based on the player's level when the item was looted.
    ///
    /// # Formula
    ///
    /// The equation used is <a href="https://en.wikipedia.org/wiki/Linear_interpolation" target="_blank">linear interpolation</a>.
    ///
    /// ```text
    /// y = y₀ + (x - x₀) * ((y₁ - y₀) / (x₁ - x₀))
    /// ```
    ///
    /// * `y` = effective item level
    /// * `x` = looted item level
    /// * `x₀` = player item level in the curve at position `n - 1`
    /// * `x₁` = player item level in the curve at position `n`
    /// * `y₀` = item level in the curve at position `n - 1`
    /// * `y₁` = item level in the curve at position `n`
    /// * `n` = position in the curve series where `x₀ > x <= x₁`
    ///
    /// Let's use the following curve points as an example. This corresponds
    /// to curve ID `1748` (as of 9.0.5).
    ///
    /// ```text
    /// [
    ///   (1, 6), (25, 31), (26, 32), (27, 33), (28, 34),
    ///   (31, 38), (32, 39), (39, 46), (40, 47), (42, 50),
    ///   (43, 50), (44, 50), (45, 50), (46, 51), (49, 57),
    ///   (50, 57), (51, 98), (59, 146), (60, 146)
    /// ]
    /// ```
    ///
    /// If a player looted an item with this curve ID at level `37`, the formula would be:
    ///
    /// ```text
    /// y = 39 + (37 - 32) * ((46 - 39) / (39 - 32))
    /// y = 44 (effective item level)
    /// ```
    ///
    /// # Special Conditions
    ///
    /// The interpolation formula does not strictly apply in the following situations:
    ///
    /// * Where `x` equals any `xₙ` in the curve. In this case, `y = yₙ`.
    /// * Where `x` is greater than _all_ `xₙ` in the curve. In this case, `y = yₗ` where `yₗ`
    ///   is the last item in the curve series.
    ///
    /// # Example
    ///
    /// ```rust
    /// use etherealpost::parse::ItemLevelCurve;
    ///
    /// let curve_points: Vec<(f64, f64)> = vec![
    ///     (1.0, 6.0), (25.0, 31.0), (26.0, 32.0), (27.0, 33.0), (28.0, 34.0),
    ///     (31.0, 38.0), (32.0, 39.0), (39.0, 46.0), (40.0, 47.0), (42.0, 50.0),
    ///     (43.0, 50.0), (44.0, 50.0), (45.0, 50.0), (46.0, 51.0), (49.0, 57.0),
    ///     (50.0, 57.0), (51.0, 98.0), (59.0, 146.0), (60.0, 146.0)
    ///  ];
    ///
    /// let ilvl_curve = ItemLevelCurve::from_points(&curve_points);
    /// assert_eq!(12, ilvl_curve.calc_ilvl(&7));
    /// assert_eq!(44, ilvl_curve.calc_ilvl(&37));
    /// assert_eq!(104, ilvl_curve.calc_ilvl(&52));
    /// assert_eq!(146, ilvl_curve.calc_ilvl(&65));
    /// ```
    pub fn calc_ilvl(&self, looted_level: &u32) -> u32 {
        let looted_level = *looted_level as f64;
        let error_margin = 0.01f64;

        let mut prev = &self.points[0];

        for point in &mut self.points.iter() {
            // if the player level is the looted level, we don't need to interpolate
            if (looted_level - point.player_level).abs() < error_margin {
                return point.item_level as u32;
            }
            // interpolate: y = y0 + (x - x0) * ( (y1 - y0) / (x1 - x0) )
            if looted_level < point.player_level {
                return (prev.item_level
                    + (looted_level - prev.player_level)
                        * ((point.item_level - prev.item_level)
                            / (point.player_level - prev.player_level)))
                    as u32;
            }
            prev = point;
        }

        prev.item_level as u32
    }
}

pub type ItemLevelCurvePoints = HashMap<u32, ItemLevelCurve>;

/// An individual curve point, which represents `(x, y)` coordinates on a graph,
/// where `x` is player level and `y` is the effective level of the item.
struct CurvePoint {
    player_level: f64,
    item_level: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_ilvl_looted_level_in_curve() {
        let curve_points: Vec<(f64, f64)> =
            vec![(1.0, 6.0), (25.0, 31.0), (26.0, 32.0), (27.0, 33.0)];
        let curve = ItemLevelCurve::from_points(&curve_points);
        assert_eq!(32, curve.calc_ilvl(&26));
    }

    #[test]
    fn calc_ilvl_curve_with_zero_level() {
        let curve_points: Vec<(f64, f64)> = vec![(0.0, 99.0)]; // Curve ID 19995
        let curve = ItemLevelCurve::from_points(&curve_points);
        assert_eq!(99, curve.calc_ilvl(&1));
        assert_eq!(99, curve.calc_ilvl(&25));
        assert_eq!(99, curve.calc_ilvl(&50));
        assert_eq!(99, curve.calc_ilvl(&60));
    }

    #[test]
    fn calc_ilvl_curve_looted_level_gt_max_curve() {
        let curve_points: Vec<(f64, f64)> =
            vec![(1.0, 6.0), (25.0, 31.0), (26.0, 32.0), (27.0, 33.0)];
        let curve = ItemLevelCurve::from_points(&curve_points);
        assert_eq!(33, curve.calc_ilvl(&60));
    }

    #[test]
    fn calc_ilvl_curve_looted_level_lt_one_curve() {
        let curve_points: Vec<(f64, f64)> =
            vec![(1.0, 6.0), (25.0, 31.0), (26.0, 32.0), (27.0, 33.0)];
        let curve = ItemLevelCurve::from_points(&curve_points);
        assert_eq!(12, curve.calc_ilvl(&7));
    }
}
