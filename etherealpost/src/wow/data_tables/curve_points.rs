use serde::Deserialize;
use std::collections::HashMap;

/// Container struct for all of the [`Db2CurvePoints`] rows.
pub struct Db2CurvePoints {
    // TODO(seputaes) Do we need to keep this data in memory?
    // points: Vec<DB2CurvePoint>,
    /// Mapping of Curve IDs to all of the curve `(x, y)` values.
    pub curve_ids: HashMap<u32, Vec<(f64, f64)>>,
}

/// A struct representation of a single row in the CurvePoint DB2 table
/// of World of Warcraft game files.
#[derive(Debug, Deserialize)]
pub struct Db2CurvePoint {
    /// The ID of a single Curve Point `(x, y)` coordinate.
    #[serde(rename = "ID")]
    pub id: u32,

    /// The `x` coordinate, which corresponds to Player Level
    #[serde(rename = "Pos[0]")]
    pub x: f64,

    /// The `y` coordinate, which corresponds to Item Level
    #[serde(rename = "Pos[1]")]
    pub y: f64,

    /// The `x` coordinate prior to the Patch 9.0 Item Level squish.
    #[serde(rename = "PosPreSquish[0]")]
    pub x_pre_squish: f64,

    /// The `y` coordinate prior to the Patch 9.0 Item Level squish.
    #[serde(rename = "PosPreSquish[1]")]
    pub y_pre_squish: f64,

    /// The ID of the overall curve, which is made up of one or more
    /// curve points. This ID is tied to Bonus IDs via the
    /// [Db2ItemBonuses](`super::Db2ItemBonuses`) table.
    #[serde(rename = "CurveID")]
    pub curve_id: u32,

    /// The order index within the table if between "like" items.
    /// Typically unused in the context of Ethereal Post.
    #[serde(rename = "OrderIndex")]
    pub order_index: u16,
}

/// Functionality for working with Curve Points and their effect on items.
///
/// In addition to mapping the rows into a [`Db2CurvePoint`],
/// parsing is done which maps all Curve IDs to all `(x, y)` coordinates
/// associated with that ID for fast retrieval.
impl Db2CurvePoints {
    /// Deserializes a CSV string which represents the DB2 CurvePoints table
    /// in World of Warcraft.
    pub fn from_csv(csv: &str) -> Db2CurvePoints {
        let mut reader = csv::Reader::from_reader(csv.as_bytes());
        let iter = reader.deserialize::<Db2CurvePoint>();

        // TODO(seputaes): Some rows contain floats for their `x` and `y` coordinates
        // I have no idea if they're ever used for our context, but for now
        // we're just casting them into u32 which is _not_ safe.
        let mut curve_ids: HashMap<u32, Vec<(f64, f64)>> = HashMap::new();

        for point in iter {
            if point.is_err() {
                continue;
            }
            let point = point.unwrap();

            curve_ids
                .entry(point.curve_id)
                .or_insert_with(Vec::new)
                .push((point.x, point.y));
        }

        Db2CurvePoints {
            // TODO(seputaes) Do we need to keep this data in memory?
            // points,
            curve_ids,
        }
    }

    /// Find the curve `(x, y)` coordinates associated with a Curve ID,
    /// if it exists.
    pub fn points(&self, curve_id: &u32) -> Option<&Vec<(f64, f64)>> {
        self.curve_ids.get(curve_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CURVE_CSV_HEADER: &str =
        "ID,Pos[0],Pos[1],PosPreSquish[0],PosPreSquish[1],CurveID,OrderIndex";

    #[test]
    fn curve_single_curve() {
        let mut csv = String::from(CURVE_CSV_HEADER);
        csv.push_str("\n5,1,6,0,1,5,0");

        let table = Db2CurvePoints::from_csv(&csv);
        assert_eq!(vec![(1.0f64, 6.0f64)], *table.points(&5).unwrap());
    }

    #[test]
    fn curve_multiple_curves() {
        let mut csv = String::from(CURVE_CSV_HEADER);
        csv.push_str("\n5,1,6,0,1,5,0\n9,25,31,0,1,5,0");

        let table = Db2CurvePoints::from_csv(&csv);
        assert_eq!(vec![(1.0, 6.0), (25.0, 31.0)], *table.points(&5).unwrap());
    }

    #[test]
    fn curve_mixed_curves() {
        let mut csv = String::from(CURVE_CSV_HEADER);
        csv.push_str("\n5,1,6,0,1,5,0\n2,3,4,0,1,9,0\n9,25,31,0,1,5,0");

        let table = Db2CurvePoints::from_csv(&csv);
        assert_eq!(vec![(1.0, 6.0), (25.0, 31.0)], *table.points(&5).unwrap());
    }

    #[test]
    fn curve_no_curve_with_id() {
        let mut csv = String::from(CURVE_CSV_HEADER);
        csv.push_str("\n5,1,6,0,1,5,0\n2,3,4,0,1,9,0\n9,25,31,0,1,5,0");

        let table = Db2CurvePoints::from_csv(&csv);
        assert!(table.points(&12).is_none());
    }
}
