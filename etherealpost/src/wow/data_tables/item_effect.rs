use serde::Deserialize;
use std::collections::{HashMap, HashSet};

const LEARN_TRIGGER_TYPE: i16 = 6;

/// Container struct for all of the [`Db2ItemEffect`] rows.
pub struct Db2ItemEffects {
    // TODO(seputaes) Do we need to keep this data in memory?
    // effects: HashMap<u32, Db2ItemEffect>,
    /// Mapping of Item IDs to Spell IDs that trigger
    /// something to be "learned."
    ///
    /// For the purpose of this lib, this is used to cross reference the
    /// [Db2BattlePetSpeciesTable](`crate::wow::data_tables::Db2BattlePetSpeciesTable`)
    /// table to identify which **Pet** will be learned when the item is used,
    /// which allows us to map Item IDs to Pet Species IDs.
    pub item_to_spell_learn: HashMap<u32, u32>,
}

/// A struct representation of a single row in the ItemEffect DB2 table
/// of World of Warcraft game files.
///
/// This table contains a significant amount of extra data than the fields on this struct,
/// but they're ignored here since those field's aren't used ... yet.
#[derive(Debug, Deserialize)]
pub struct Db2ItemEffect {
    /// The unique ID of the spell effect.
    #[serde(rename = "ID")]
    id: u32,

    /// The Spell ID that is associated with the effect.
    #[serde(rename = "SpellID")]
    spell_id: u32,

    /// The type that is triggered when the spell is activated.
    ///
    /// For example, this could be a "use" or "learn" action.
    /// For the purpose of this lib, only type `6` is used (learn)
    /// to identify pets.
    #[serde(rename = "TriggerType")]
    trigger_type: i16,

    /// The Item ID that will trigger this effect when it is activated.
    #[serde(rename = "ParentItemID")]
    parent_item_id: u32,
}

/// Functionality for working with Item Effects and their triggers by Items.
///
/// In addition to mapping the rows into a [`Db2ItemEffects`],
/// parsing is done which maps all Item IDs to a Spell ID for the _Learn_
/// trigger type.
impl Db2ItemEffects {
    /// Deserializes a CSV string which represents the DB2 ItemEffect table
    /// in World of Warcraft.
    pub fn from_csv(csv: &str) -> Db2ItemEffects {
        let mut reader = csv::Reader::from_reader(csv.as_bytes());
        let iter = reader.deserialize::<Db2ItemEffect>();

        let mut item_to_spell_learn = HashMap::new();
        let mut known_parent_ids = HashSet::new();

        for row in iter {
            // TOOD(seputaes): Logging for the error
            if row.is_err() {
                panic!("Parsing error: {}", row.err().unwrap());
            }

            let row = row.unwrap();

            // check for multiple spell IDS for the same parent item ID with trigger type of 6
            if row.trigger_type == LEARN_TRIGGER_TYPE {
                if known_parent_ids.contains(&row.parent_item_id) {
                    // TODO(seputaes): Logging. I think this is harmless, but
                    //                 it would be good to know when this happens.
                    continue;
                }
                known_parent_ids.insert(row.parent_item_id);

                item_to_spell_learn.insert(row.parent_item_id, row.spell_id);
            }
        }

        Db2ItemEffects {
            // TODO(seputaes) Do we need to keep this data in memory?
            // effects: HashMap::new(),
            item_to_spell_learn,
        }
    }
}
