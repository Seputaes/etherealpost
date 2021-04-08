use serde::Deserialize;
use std::collections::HashMap;

/// Container struct for all of the [`Db2ItemSparse`] rows.
pub struct Db2ItemSparseTable {
    // TODO(seputaes) Do we need to keep this data in memory?
    // items: HashMap<u32, Db2ItemSparse>,
    /// Contains a mapping of Item IDs to their base item level.
    pub base_item_levels: HashMap<u32, u32>,
}

/// A struct representation of a single row in the Item DB2 table
/// of World of Warcraft game files.
///
/// This table contains a significant amount of extra data than the fields on this struct,
/// but they're ignored here since those field's aren't used ... yet.
#[derive(Debug, Deserialize)]
pub struct Db2ItemSparse {
    /// The unique ID of the item.
    #[serde(rename = "ID")]
    pub id: u32,

    /// The base item level of the item.
    #[serde(rename = "ItemLevel")]
    pub item_level: u32,
}

/// Functionality for working with Items from the DB2 table.
///
/// Currently, all this does is extract out Base Item Levels and place them into their own vector
/// and enables us to keep a relatively small amount of semi-static data in memory
/// so there is no need to hit the API to get this information.
impl Db2ItemSparseTable {
    /// Deserializes a CSV string which represents the DB2 ItemSparse table
    /// in World of Warcraft.
    pub fn from_csv(csv: &str) -> Db2ItemSparseTable {
        let mut reader = csv::Reader::from_reader(csv.as_bytes());
        let iter = reader.deserialize::<Db2ItemSparse>();

        let mut base_item_levels = HashMap::new();

        for row in iter {
            // TODO(seputaes): Logging for the error
            if row.is_err() {
                continue;
            }

            let row = row.unwrap();

            base_item_levels.insert(row.id, row.item_level);
        }

        Db2ItemSparseTable {
            // TODO(seputaes) Do we need to keep this data in memory?
            // items: HashMap::new(),
            base_item_levels,
        }
    }

    /// Returns the base item level for the supplied `item_id`.
    ///
    /// Some item data is not available to us, either through the API
    /// or the DB2 files. It's unclear whether the missing data
    /// is even available to players.
    ///
    /// If the item ID is not present in the data files containing
    /// item level information, then a default `1` is returned.
    ///
    /// TODO(seputaes): Add some logging here. It would be good to know
    ///                 when there isn't base ilvl info.
    pub fn base_ilvl(&self, item_id: &u32) -> u32 {
        *self.base_item_levels.get(item_id).unwrap_or(&1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ITEM_SPARSE_CSV_HEADER: &str = "ID,AllowableRace,Description_lang,Display3_lang,Display2_lang,Display1_lang,Display_lang,DmgVariance,DurationInInventory,QualityModifier,BagFamily,ItemRange,StatPercentageOfSocket[0],StatPercentageOfSocket[1],StatPercentageOfSocket[2],StatPercentageOfSocket[3],StatPercentageOfSocket[4],StatPercentageOfSocket[5],StatPercentageOfSocket[6],StatPercentageOfSocket[7],StatPercentageOfSocket[8],StatPercentageOfSocket[9],StatPercentEditor[0],StatPercentEditor[1],StatPercentEditor[2],StatPercentEditor[3],StatPercentEditor[4],StatPercentEditor[5],StatPercentEditor[6],StatPercentEditor[7],StatPercentEditor[8],StatPercentEditor[9],Stackable,MaxCount,RequiredAbility,SellPrice,BuyPrice,VendorStackCount,PriceVariance,PriceRandomValue,Flags[0],Flags[1],Flags[2],Flags[3],OppositeFactionItemID,ModifiedCraftingReagentItemID,ContentTuningID,PlayerLevelToItemLevelCurveID,ItemNameDescriptionID,RequiredTransmogHoliday,RequiredHoliday,LimitCategory,Gem_properties,Socket_match_enchantment_ID,TotemCategoryID,InstanceBound,ZoneBound[0],ZoneBound[1],ItemSet,LockID,StartQuestID,PageID,ItemDelay,MinFactionID,RequiredSkillRank,RequiredSkill,ItemLevel,AllowableClass,ExpansionID,ArtifactID,SpellWeight,SpellWeightCategory,SocketType[0],SocketType[1],SocketType[2],SheatheType,Material,PageMaterialID,LanguageID,Bonding,DamageType,StatModifier_bonusStat[0],StatModifier_bonusStat[1],StatModifier_bonusStat[2],StatModifier_bonusStat[3],StatModifier_bonusStat[4],StatModifier_bonusStat[5],StatModifier_bonusStat[6],StatModifier_bonusStat[7],StatModifier_bonusStat[8],StatModifier_bonusStat[9],ContainerSlots,MinReputation,RequiredPVPMedal,RequiredPVPRank,RequiredLevel,InventoryType,OverallQualityID";

    #[test]
    fn base_ilvl_returns_base_ilvl_if_exists() {
        let mut csv = String::from(ITEM_SPARSE_CSV_HEADER);
        csv.push_str("\n183421,-1,,,,,Stone Legion Sabatons,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,5259,7889,4250,2750,0,0,0,0,0,0,1,0,0,424559,2122798,1,1,0.9565,0,8192,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,100,-1,8,0,0,0,0,0,0,0,5,0,0,2,0,74,7,40,36,-1,-1,-1,-1,-1,-1,0,0,0,0,48,8,3");

        let table = Db2ItemSparseTable::from_csv(&csv);
        assert_eq!(100, table.base_ilvl(&183421));
    }

    #[test]
    fn base_ilvl_returns_default_if_not_found() {
        let mut csv = String::from(ITEM_SPARSE_CSV_HEADER);
        csv.push_str("\n183421,-1,,,,,Stone Legion Sabatons,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,5259,7889,4250,2750,0,0,0,0,0,0,1,0,0,424559,2122798,1,1,0.9565,0,8192,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,100,-1,8,0,0,0,0,0,0,0,5,0,0,2,0,74,7,40,36,-1,-1,-1,-1,-1,-1,0,0,0,0,48,8,3");

        let table = Db2ItemSparseTable::from_csv(&csv);
        assert_eq!(1, table.base_ilvl(&25));
    }
}
