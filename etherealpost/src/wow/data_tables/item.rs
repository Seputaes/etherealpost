use serde::Deserialize;

const MISC_CLASS_ID: u32 = 15;
const PET_SUBCLASS_ID: u32 = 2;

/// Container struct for all of the [`Db2Item`] rows.
pub struct Db2Items {
    // TODO(seputaes) Do we need to keep this data in memory?
    // items: HashMap<u32, Db2Item>,
    /// Contains a list of all items which are Pet Items.
    /// This list isn't perfectly accurate. There are some items in the game
    /// which are classified as "Companion Pets", but are actually things like
    /// toys which augment companion pets (pet bed, for example).
    ///
    /// In order to accurately determine items are actual pets, and more specifically,
    /// what is an _obtainable_ pet in the game, you need to cross reference these
    /// items against both the [Db2ItemEffects](`crate::wow::data_tables::Db2ItemEffects`)
    /// and [Db2BattlePetSpeciesTable](`crate::wow::data_tables:Db2BattlePetSpeciesTable`)
    /// tables to determine what spell triggers the pet to be learned.
    pub pet_item_ids: Vec<u32>,
}

/// A struct representation of a single row in the Item DB2 table
/// of World of Warcraft game files.
///
/// This table contains a significant amount of extra data than the fields on this struct,
/// but they're ignored here since those field's aren't used ... yet.
#[derive(Debug, Deserialize)]
pub struct Db2Item {
    /// The unique ID of the item.
    #[serde(rename = "ID")]
    pub id: u32,

    /// The class ID associated with the item.
    /// For the purposes of this library, the only class currently used
    /// is `15` which is **Miscellaneous**.
    #[serde(rename = "ClassID")]
    pub class_id: u32,

    /// The subclass ID of the `class_id` associated with the item.
    ///
    /// For the purposes of this library, the only class currently used
    /// is `2` which is **Companion Pet**.
    ///
    /// Subclasses are not globally unique. They are only unique within
    /// the scope of a "parent" `class_id`.
    #[serde(rename = "SubclassID")]
    pub subclass_id: u32,
}

/// Functionality for working with Items from the DB2 table.
///
/// Currently, all this does is extract out Pet items and place them into their own vector
/// and enables us to keep a relatively small amount of semi-static data in memory
/// so there is no need to hit the API to get this information.
impl Db2Items {
    /// Deserializes a CSV string which represents the DB2 Item table
    /// in World of Warcraft.
    pub fn from_csv(csv: &str) -> Db2Items {
        let mut reader = csv::Reader::from_reader(csv.as_bytes());
        let iter = reader.deserialize::<Db2Item>();

        let mut pet_item_ids = Vec::new();

        for row in iter {
            // TODO(seputaes): Logging for the error
            if row.is_err() {
                continue;
            }

            let row = row.unwrap();

            if row.class_id == MISC_CLASS_ID && row.subclass_id == PET_SUBCLASS_ID {
                pet_item_ids.push(row.id);
            }
        }

        Db2Items {
            // TODO(seputaes) Do we need to keep this data in memory?
            // items: HashMap::new(),
            pet_item_ids,
        }
    }
}
