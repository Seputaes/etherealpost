use serde::Deserialize;

/// A raw Auctions resource which is returned by Blizzard's
/// Auction House API for single Connected Realm, and consisting of [`Auction`]s.
///
/// Documentation for this API can be found
/// [on Blizzard's site](https://develop.battle.net/documentation/world-of-warcraft/game-data-apis).
///
/// While the raw resource which is returned from the API contains some other fields,
/// the only important one for the purpose of this library is `auctions`, so the others
/// are discarded.
#[derive(Deserialize)]
pub struct AuctionFile {
    /// Vector containing all of the current auctions currently on the
    /// Connected Realm's Auction House.
    pub auctions: Vec<Auction>,
}

impl AuctionFile {
    /// Deserialize an instance of [`AuctionFile`] from a JSON string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::str::FromStr;
    /// use etherealpost::auctions::AuctionFile;
    ///
    /// let json = "
    ///   {
    ///     \"auctions\": [
    ///       {
    ///         \"id\": 1234,
    ///         \"quantity\": 1,
    ///         \"item\": {
    ///           \"id\": 72092
    ///         },
    ///         \"unit_price\": 164068,
    ///         \"time_left\": \"MEDIUM\"
    ///       }
    ///     ]
    ///   }";
    /// let auction_file = AuctionFile::from_json(json).unwrap();
    /// println!(
    ///     "The unit price for auction {} is {}",
    ///     auction_file.auctions[0].id,
    ///     auction_file.auctions[0].unit_price.unwrap()
    /// );
    /// ```
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

/// A single Auction     that is currently on the Auction House.
///
/// There are three price fields that can be part of the Auction: `unit_price`, `buyout`, and `bid`.
///
/// Of these 3 fields, only the following combinations are possible:
///
/// 1. `unit_price` only
/// 2. `buyout` only
/// 3. `bid` only
/// 4. `bid` and `buyout` only
///
/// Otherwise, the fields not present will be `None`.
///
#[derive(Deserialize)]
pub struct Auction {
    /// Unique ID for the auction. This ID is unique _per connected realm_ and not guaranteed
    /// to be unique across the entire region or world.
    pub id: u64,

    /// The quantity (stack size) of the auction.
    pub quantity: u16,

    /// The item which is being auctioned.
    pub item: Item,

    /// The Unit Price (per item) for the auction.
    ///
    /// If Unit Price is present, `buyout` and `bid` _will not_ be present.
    pub unit_price: Option<u64>,

    /// The price to buyout the auction.
    ///
    /// If Buyout is present, `unit_price` _will not_ be present. `bid` _may or may not_ be present.
    pub buyout: Option<u64>,

    /// The current bid price for the auction.
    ///
    /// If Bid is present, `unit_price` _will not_ be present and `buyout` _may or may not_
    /// be present.
    pub bid: Option<u64>,

    /// The current time left for the auction. See [`TimeLeft`].
    pub time_left: TimeLeft,
}

/// An item which is up for auction on an [`Auction`].
#[derive(Deserialize)]
pub struct Item {
    /// The ID of the item. This is also the in-game ID for the item, and
    /// you can easily look this up on various resources (WowHead, etc).
    pub id: u64,

    /// Context is defined as the "creation context". Typically, this
    /// indicates where the item dropped (eg, LFR/Normal/Heroic/Mythic raid).
    ///
    /// Generally, this information isn't needed in the context of the auction house.
    pub context: Option<u16>,

    /// List of bonuses present on the item.
    ///
    /// There are thousands of possible bonus IDs. These represent bonuses such as item
    /// level changes, item quality changes, affixes.
    ///
    /// For example, for affixes, a bonus ID of `19` indicates **of the Fireflash**. In fact, there
    /// are dozens of bonus ids for **Fireflash** and unfortunately, Blizzard doesn't
    /// provide an API to explain this data.
    ///
    /// TODO(seputaes): Need either a hardcoded map or a database lookup for these.
    ///                 Raidbots [has some resources](https://www.raidbots.com/static/data/live/bonuses.json)
    ///                 available, but it's not comprehensive.
    pub bonus_lists: Option<Vec<u32>>,

    /// List of modifiers present on the item.
    ///
    /// Like `bonus_lists`, Blizzard doesn't provide an API to explain this one. It can
    /// indicate many number of things. For example, a type of `9` indicates the player's
    /// level when the item dropped for them.
    ///
    /// The player's level (type `9`) is frequently used when calculating
    /// the effective item level based on a an [ItemLevelCurve](`crate::parse::ItemLevelCurve`).
    pub modifiers: Option<Vec<ItemModifier>>,

    /// If this item is a Pet or Pet Cage, this is the Pet's Breed ID.
    pub pet_breed_id: Option<u32>,

    /// If this item is a Pet or Pet Cake, this is the Pet's level (0-25).
    pub pet_level: Option<u8>,

    /// If this item is a Pet or Pet Cage, this is the Pet's Quality ID.
    pub pet_quality_id: Option<u16>,

    /// If this item is a Pet or a Pet Cage, this is the Pet's Species ID.
    pub pet_species_id: Option<u32>,
}

impl Item {
    /// Creates a new [`AuctionPet`] from the auction item, if the item is a pet.
    pub fn pet(&self) -> Option<AuctionPet> {
        match (
            self.pet_breed_id,
            self.pet_level,
            self.pet_quality_id,
            self.pet_species_id,
        ) {
            (Some(breed), Some(level), Some(quality), Some(species)) => Some(AuctionPet {
                breed,
                quality,
                species,
                level,
            }),
            _ => None,
        }
    }
}

/// Metadata about an [`Item`] which is a pet.
pub struct AuctionPet {
    /// The breed ID of the pet.
    pub breed: u32,

    // The quality ID of the pet.
    pub quality: u16,

    /// The species ID of the pet.
    pub species: u32,

    /// The pet level (1-25).
    pub level: u8,
}

/// An Auction Item Modifier.
///
/// Not much information is available form Blizzard about this field, and there
/// is no API to describe it.
///
/// # Known Modifiers
///
/// - `9` - The player's level when the item was looted.
///
/// TODO(seputaes): Need to find some additional data for this.
#[derive(Deserialize)]
pub struct ItemModifier {
    /// The modifier type ID. Serialized from `type`.
    #[serde(alias = "type")]
    pub modifier_type: u16,

    /// The value of the modifier.
    pub value: u64,
}

/// The amount of time left on an [`Auction`].
#[derive(Deserialize)]
pub enum TimeLeft {
    /// Parses from `VERY_LONG` and means more than **12 hours** remaining.
    #[serde(rename = "VERY_LONG")]
    VeryLong,

    /// Parses from `LONG` and means between **2 hours** and **12 hours** remaining.
    #[serde(rename = "LONG")]
    Long,

    /// Parses from `MEDIUM` and means between **30 minutes** and **2 hours** remaining.
    #[serde(rename = "MEDIUM")]
    Medium,

    /// Parses from `SHORT` and means less than **30 minutes** remaining.
    #[serde(rename = "SHORT")]
    Short,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn item_with_all_pet_fields_returns_pet() {
        let item = Item {
            id: 0,
            context: None,
            bonus_lists: None,
            modifiers: None,
            pet_species_id: Some(1),
            pet_quality_id: Some(2),
            pet_level: Some(3),
            pet_breed_id: Some(4),
        };
        let pet = item.pet().unwrap();

        assert_eq!(1, pet.species);
        assert_eq!(2, pet.quality);
        assert_eq!(3, pet.level);
        assert_eq!(4, pet.breed);
    }

    #[test]
    fn item_with_missing_pet_fields_returns_none() {
        let item = Item {
            id: 0,
            context: None,
            bonus_lists: None,
            modifiers: None,
            pet_species_id: Some(1),
            pet_quality_id: Some(2),
            pet_level: None,
            pet_breed_id: Some(4),
        };
        assert!(item.pet().is_none());
    }
}
