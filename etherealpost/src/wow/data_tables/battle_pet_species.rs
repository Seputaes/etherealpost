use serde::Deserialize;
use std::collections::HashMap;

/// Container struct for all of the [`Db2BattlePetSpecies`] rows.
pub struct Db2BattlePetSpeciesTable {
    // TODO(seputaes) Do we need to keep this data in memory?
    // species: HashMap<u32, Db2BattlePetSpecies>,
    spell_to_species: HashMap<u32, u32>,
}

/// A struct representation of a single row in the BattlePetSpecies DB2 table
/// of World of Warcraft game files.
///
/// This table contains a significant amount of extra data than the fields on this struct,
/// but they're ignored here since those field's aren't used ... yet.
#[derive(Debug, Deserialize)]
pub struct Db2BattlePetSpecies {
    // The unique ID of the Battle Pet Species.
    #[serde(rename = "ID")]
    pub id: u32,

    // The spell that is used to summon the pet.
    // This can be used to reverse-lookup an
    // item on the AH that is a pet but isn't in a pet cage (82800)
    // in order to find its species
    #[serde(rename = "SummonSpellID")]
    pub summon_spell_id: u32,
}

/// Functionality for working with Battle Pet Species and their effect on items.
///
/// In addition to mapping the rows into a [`Db2BattlePetSpecies`],
/// parsing is done which maps all Summon Spell IDs to its associated Species ID
impl Db2BattlePetSpeciesTable {
    pub fn from_csv(csv: &str) -> Db2BattlePetSpeciesTable {
        let mut reader = csv::Reader::from_reader(csv.as_bytes());
        let iter = reader.deserialize::<Db2BattlePetSpecies>();

        let mut spell_to_species = HashMap::new();

        for row in iter {
            // TODO(seputaes): Logging for the error
            if row.is_err() {
                continue;
            }

            let row = row.unwrap();

            // If the summon spell ID is 0, the current theory is that it
            // is not able to be in a pet cage, and thus can't be sold on the AH
            // TODO(seputaes): Maybe have some logging around this to confirm this theory
            //                 using real AH data.
            if row.summon_spell_id == 0 {
                continue;
            }

            if spell_to_species.contains_key(&row.summon_spell_id) {
                // from testing, it looks like if there's a duplicate
                // spell ID, the first one in the table wins
                // as of 9.0.5, duplicate spell IDs are:

                // 15048
                // 25162
                // 53082
                // 89472
                // 132762
                // 135259
                // 138161
                // 149810
                // 170272
                // 291537
                continue;
            }

            spell_to_species.insert(row.summon_spell_id, row.id);
        }

        Db2BattlePetSpeciesTable { spell_to_species }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BATTLE_PET_SPECIES_CSV_HEADER: &str =
        "Description_lang,SourceText_lang,ID,CreatureID,SummonSpellID,IconFileDataID,PetTypeEnum,Flags,SourceTypeEnum,CardUIModelSceneID,LoadoutUIModelSceneID,CovenantID";

    #[test]
    fn battle_pet_species_uses_first_spell_id() {
        let mut csv = String::from(BATTLE_PET_SPECIES_CSV_HEADER);
        csv.push_str("\n\"Possibly explosive, definitely adorable. Keep away from open flame.\",|cFFFFD200Profession: |rEngineering,85,9656,15048,133712,9,2,3,6,7,0");
        csv.push_str("\n\"The first bombling created in the Underhold, Siegecrafter Blackfuse couldn't bear to see it destroyed, and kept it as a friendly, if explosive, pet.\",|cFFFFD200Drop:|r Siegecrafter Blackfuse|n|cFFFFD200Raid:|r Siege of Orgrimmar,1322,73352,15048,897633,9,2,0,6,7,0");

        let table = Db2BattlePetSpeciesTable::from_csv(&csv);
        assert_eq!(85, *table.spell_to_species.get(&15048).unwrap());
    }
}
