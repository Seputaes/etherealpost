use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::auctions;
use crate::parse;
use crate::stats;
use crate::wow::data_tables;

/// A summarized snapshot of a raw [AuctionFile](`crate::auctions::AuctionFile`) that contains
/// various statistical data and pre-computed item mappings and associations.
///
/// Given an [AuctionFile](`crate::auctions::AuctionFile`), the implementations of this struct
/// will provide essentially all access that could be needed into an auction file in order to
/// save that snapshot into a database, or extract data to further analyze in an adhoc capacity.
///
/// # Pre-Computed Data and Mappings
///
/// The layout of this struct contains several helpful pre-computed mappings which will
/// aid future lookups. Examples include:
///
/// * Map Item IDs to all auctions in the snapshot for that item.
/// * For equippable items, map both Item ID and that item's level so that consumers
///   can look up both the "item price" as well as the more granular "item level" price.
///   For example, this distinction is useful for identifying a "transmog" price vs
///   a "buying it for the stats" price.
/// * A mapping of Pet Species IDs to all items on the auction house for that species,
///   including _both_ pet cages and the original item drop which learns the pet.
pub struct AuctionsSummary<'a> {
    /// Mapping of **Item ID** to all buyable auctions for that item.
    pub item_auctions: HashMap<u64, Vec<&'a auctions::Auction>>,

    /// Mapping of **Item ID** to the Auction Item Summary information for those items.
    pub item_summaries: HashMap<u64, ItemSummary>,

    /// Nested map of **Item ID -> Item Level** to all buyable auctions for
    /// the items sharing the same item level.
    ///
    /// This only contains equippable items.
    pub item_level_auctions: HashMap<u64, HashMap<u32, Vec<&'a auctions::Auction>>>,

    /// Nested map of **Item ID -> Item Level** to the Auction Item Summary for
    /// just the items sharing the same item level.
    ///
    /// This only contains to equippable items.
    pub item_level_summaries: HashMap<u64, HashMap<u32, ItemSummary>>,

    /// Mapping of **Pet Species ID** to all buyable auctions for that pet.
    ///
    /// This combines pet cages and non-pet cage pet items (which can be learned)
    /// mapped to the same species ID.
    pub pet_auctions: HashMap<u32, Vec<&'a auctions::Auction>>,

    /// Mapping of **Pet Species ID** to the Auction Item Summary for the
    /// pets of that species.
    ///
    /// This combines pet cages and non-pet cage pet items (which can be learned)
    /// mapped to the same species ID.
    pub pet_summaries: HashMap<u32, ItemSummary>,
}

/// Summarized information and statistics about a grouping of items on the auction house,
/// such as an Item or Pet.
pub struct ItemSummary {
    /// The calculated market price for all of the items which are represented
    /// by this summary.
    ///
    /// For more information on how this is calculated, see
    /// [normalized_market_price_with_qty](`crate::stats::normalized_market_price_with_qty`).
    pub market_price: u64,

    /// The population standard deviation for all of the prices of the auctions which are
    /// represented by this summary.
    pub std_dev: f64,

    /// The minimum buyout of all of the auctions that are represented by this summary.
    pub min_buyout: u64,

    /// The total quantity of an the item represented by this group available in this
    /// auction house snapshot.
    ///
    /// This is the total **units** of an item available. For example, if there
    /// are 2 auctions for cloth, one with a quantity of `20` and the other `40`, then
    /// this field will be `60`.
    pub total_qty: u64,

    /// The total number of unique auctions (regardless of the quantity of each) that
    /// for the item represented by this group available in this auction house snapshot.
    pub num_auctions: u64,
}

/// Implementation for generating and working with an Auctions Summary.
impl<'a> AuctionsSummary<'a> {
    /// Takes an [AuctionFile](`crate::auctions::AuctionFile`) struct representation
    /// of the JSON file for all of the auctions currently on a realm.
    ///
    /// The file will be parsed into an Auctions Summary that contains
    /// various statistical data and pre-computed item mappings and associations
    /// about all of the auctions in that file.
    ///
    /// This file is downloaded from the Blizzard API.
    ///
    /// # Arguments
    ///
    /// * `auction_file` - The parsed auction file to process into a summary.
    /// * `curve_points` - Precomputed mapping of Curve IDs to the curve that should
    ///   be used to calculate an item's level. This value can be computed using
    ///   a [Db2CurvePoints](`crate::wow::data_tables::Db2CurvePoints`) table that
    ///   has been passed into [ItemLevelCurve](`super::ItemLevelCurve::for_whole_table`).
    /// * `db2_bonuses` - Parsed DB2 table which contains Item Level Bonus IDs and their
    ///   associated Curve IDs.
    /// * `item_to_pet` - Precomputed mapping of **Item IDs** to the **Pet Species ID**
    ///   which they learn.
    /// * `equippable_items` - Precomputed set of **Item IDs** which are equippable.
    pub fn from_auction_file(
        auction_file: &'a auctions::AuctionFile,
        curve_points: &parse::ItemLevelCurvePoints,
        db2_bonuses: &data_tables::Db2ItemBonuses,
        base_ilvls: &HashMap<u64, u32>,
        item_to_pet: &HashMap<u64, u32>,
        equippable_items: &HashSet<u64>,
    ) -> Self {
        let mut item_auctions: HashMap<u64, Vec<&auctions::Auction>> = HashMap::new();
        let mut item_level_auctions: HashMap<u64, HashMap<u32, Vec<&auctions::Auction>>> =
            HashMap::new();
        let mut pet_auctions: HashMap<u32, Vec<&auctions::Auction>> = HashMap::new();

        let mut all_prices: HashMap<u64, Vec<(u64, u64)>> = HashMap::new();
        let mut ilvl_prices: HashMap<(u64, u32), Vec<(u64, u64)>> = HashMap::new();
        let mut pet_prices: HashMap<u32, Vec<(u64, u64)>> = HashMap::new();

        let mut all_qty: HashMap<u64, u64> = HashMap::new();
        let mut ilvl_qty: HashMap<(u64, u32), u64> = HashMap::new();
        let mut pet_qty: HashMap<u32, u64> = HashMap::new();

        let mut all_num_auc: HashMap<u64, u64> = HashMap::new();
        let mut ilvl_num_auc: HashMap<(u64, u32), u64> = HashMap::new();
        let mut pet_num_auc: HashMap<u32, u64> = HashMap::new();

        for auction in &auction_file.auctions {
            if !Self::use_auction(&auction) {
                continue;
            }

            let price = Self::auction_price(&auction);

            // Add to all prices
            all_prices
                .entry(auction.item.id)
                .or_insert_with(Vec::new)
                .push((price, auction.quantity as u64));

            // Increment the unit and num auctions count for all auctions
            *(all_qty.entry(auction.item.id).or_insert(0)) += auction.quantity as u64;
            *(all_num_auc.entry(auction.item.id).or_insert(0)) += 1;

            // add the auction to all auctions
            item_auctions
                .entry(auction.item.id)
                .or_insert_with(Vec::new)
                .push(&auction);

            // pet cages
            if let Some(pet_cage) = auction.item.pet() {
                Self::insert_pet_auctions(
                    &auction,
                    &pet_cage.species,
                    &price,
                    &mut pet_prices,
                    &mut pet_auctions,
                    &mut pet_qty,
                    &mut pet_num_auc,
                );
                continue;
            }

            // check if this item is is a pet but not in a pet cage
            if let Some(species_id) = item_to_pet.get(&auction.item.id) {
                Self::insert_pet_auctions(
                    &auction,
                    species_id,
                    &price,
                    &mut pet_prices,
                    &mut pet_auctions,
                    &mut pet_qty,
                    &mut pet_num_auc,
                );

                continue;
            }

            // if the item is not equippable, the item level is the base item level
            let is_equippable = equippable_items.contains(&auction.item.id);

            let effective_level = Self::resolve_item_level(
                &auction.item,
                is_equippable,
                &db2_bonuses,
                &base_ilvls,
                &curve_points,
            );

            let ilvl_key = (auction.item.id, effective_level);

            ilvl_prices
                .entry(ilvl_key)
                .or_insert_with(Vec::new)
                .push((price, auction.quantity as u64));

            item_level_auctions
                .entry(auction.item.id)
                .or_insert_with(HashMap::new)
                .entry(effective_level)
                .or_insert_with(Vec::new)
                .push(&auction);

            *(ilvl_qty.entry(ilvl_key).or_insert(0)) += auction.quantity as u64;
            *(ilvl_num_auc.entry(ilvl_key).or_insert(0)) += 1;
        }

        let mut all_items: HashMap<u64, ItemSummary> = HashMap::new();
        let mut ilvl_items: HashMap<u64, HashMap<u32, ItemSummary>> = HashMap::new();
        let mut pet_items: HashMap<u32, ItemSummary> = HashMap::new();

        // Insert summaries for the global Item IDs
        Self::insert_item_summary(&mut all_prices, &mut all_items, &all_num_auc, &all_qty);
        // Insert summaries for pet species
        Self::insert_item_summary(&mut pet_prices, &mut pet_items, &pet_num_auc, &pet_qty);

        // Insert nested summaries for Item ID -> Item Level
        for (key, prices) in &mut ilvl_prices {
            if let Some(mp) = stats::normalized_market_price_with_qty(prices) {
                ilvl_items.entry(key.0).or_insert_with(HashMap::new).insert(
                    key.1,
                    ItemSummary {
                        market_price: mp,
                        std_dev: stats::std_dev_amount_qty(prices, true).unwrap_or(0.0),
                        min_buyout: prices.first().unwrap().0, // market_price function sorts
                        num_auctions: *ilvl_num_auc.get(key).unwrap(),
                        total_qty: *ilvl_qty.get(key).unwrap(),
                    },
                );
            }
        }

        AuctionsSummary {
            item_auctions,
            item_level_auctions,
            pet_auctions,
            item_summaries: all_items,
            item_level_summaries: ilvl_items,
            pet_summaries: pet_items,
        }
    }

    /// Whether or not an auction should be included in the summary.
    ///
    /// This is currently defined as having either a buyout or a unit price,
    /// which means that the auction isn't "bid only."
    fn use_auction(auction: &auctions::Auction) -> bool {
        auction.buyout.is_some() || auction.unit_price.is_some()
    }

    /// Extracts the price that should be used for the auction.
    ///
    /// This should only ever be called after [use_auction](`Self::use_auction`)
    /// has been called due to an unchecked unwrap.
    fn auction_price(auction: &auctions::Auction) -> u64 {
        auction.buyout.or(auction.unit_price).unwrap()
    }

    /// Resolves the item level _adjustment_ that should be added to an item's
    /// base item level in order to determine its actual item level.
    ///
    /// This could be a negative number.
    fn find_ilvl_adjustment(
        item: &auctions::Item,
        db2_bonuses: &data_tables::Db2ItemBonuses,
    ) -> Option<i32> {
        match &item.bonus_lists {
            Some(bonus_ids) => db2_bonuses.resolve_ilvl_adjustment(&bonus_ids),
            _ => None,
        }
    }

    /// Resolves the **Curve ID** that should be used to compute an item's
    /// level.
    fn find_curve_id(
        item: &auctions::Item,
        db2_bonuses: &data_tables::Db2ItemBonuses,
    ) -> Option<u32> {
        match &item.bonus_lists {
            Some(bonus_ids) => db2_bonuses.resolve_curve_id(&bonus_ids),
            None => None,
        }
    }

    /// Resolves the player's level when the item dropped, if
    /// this is an item which has such data.
    fn drop_level(item: &auctions::Item) -> Option<u32> {
        match &item.modifiers {
            Some(modifiers) => {
                for modifier in modifiers {
                    if modifier.modifier_type == 9 {
                        return Some(modifier.value as u32);
                    }
                }
                None
            }
            None => None,
        }
    }

    /// Shorthand method which inserts an Item Summary into the various maps
    /// that make up an Auction Summary.
    fn insert_item_summary<T>(
        prices: &mut HashMap<T, Vec<(u64, u64)>>,
        items: &mut HashMap<T, ItemSummary>,
        num_aucs: &HashMap<T, u64>,
        total_qty: &HashMap<T, u64>,
    ) where
        T: Eq + Hash + Copy,
    {
        for (key, prices) in prices.iter_mut() {
            if let Some(mp) = stats::normalized_market_price_with_qty(prices) {
                items.insert(
                    *key,
                    ItemSummary {
                        market_price: mp,
                        std_dev: stats::std_dev_amount_qty(prices, true).unwrap_or(0.0),
                        min_buyout: prices.first().unwrap().0, // market_price function sorts
                        num_auctions: *num_aucs.get(key).unwrap(),
                        total_qty: *total_qty.get(key).unwrap(),
                    },
                );
            }
        }
    }

    /// Shorthand method which inserts Pet Auctions and Prices into the various maps
    /// that make up an Auction Summary.
    fn insert_pet_auctions(
        auction: &'a auctions::Auction,
        species_id: &u32,
        price: &u64,
        pet_prices: &mut HashMap<u32, Vec<(u64, u64)>>,
        pet_auctions: &mut HashMap<u32, Vec<&'a auctions::Auction>>,
        pet_qty: &mut HashMap<u32, u64>,
        pet_num_auc: &mut HashMap<u32, u64>,
    ) {
        pet_prices
            .entry(*species_id)
            .or_insert_with(Vec::new)
            .push((*price, auction.quantity as u64));

        pet_auctions
            .entry(*species_id)
            .or_insert_with(Vec::new)
            .push(&auction);

        *(pet_qty.entry(*species_id).or_insert(0)) += auction.quantity as u64;
        *(pet_num_auc.entry(*species_id).or_insert(0)) += 1;
    }

    /// Resolves the actual item level of an item using its bonuses, curves,
    /// and player drop level, if it applies to this item.
    ///
    /// TODO(seputaes): Holy eyesore, Batman! Make this cleaner...
    fn resolve_item_level(
        item: &auctions::Item,
        is_equippable: bool,
        db2_bonuses: &data_tables::Db2ItemBonuses,
        base_ilvls: &HashMap<u64, u32>,
        curve_points: &parse::ItemLevelCurvePoints,
    ) -> u32 {
        match is_equippable {
            true => match AuctionsSummary::find_curve_id(&item, &db2_bonuses) {
                Some(curve_id) => match AuctionsSummary::drop_level(&item) {
                    Some(drop_level) => match curve_points.get(&curve_id) {
                        Some(curve) => curve.calc_ilvl(&drop_level),
                        // if this is none, it's not in the table that was pre-cached
                        None => *(base_ilvls.get(&item.id).unwrap_or(&1)),
                    },
                    None => *(base_ilvls.get(&item.id).unwrap_or(&1)),
                },
                // there's no curve associated with this item, check for a standard ilvl adjustment
                None => match AuctionsSummary::find_ilvl_adjustment(&item, &db2_bonuses) {
                    Some(adjustment) => {
                        let base_level = *(base_ilvls.get(&item.id).unwrap_or(&1)) as i32;
                        (base_level + adjustment) as u32
                    }
                    None => *(base_ilvls.get(&item.id).unwrap_or(&1)),
                },
            },
            false => *(base_ilvls.get(&item.id).unwrap_or(&1)),
        }
    }
}
