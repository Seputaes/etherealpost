/// 15.87 represents -1 standard deviation from the mean of a normal distribution curve
const MINIMUM_PRICES_PERCENTILE: f64 = 15.0;
const FIRST_STANDARD_DEV_PERCENTILE: f64 = 15.87;

/// Calculates the Market Price given an array of item buyout our unit prices.
///
/// In the context of a realm, "Market Price" is defined as the bottom "1 standard
/// deviation" of all auctions currently listed for that item. In strict numerical terms,
/// this is (up to) the bottom `15.87%` of auctions. While this isn't a strict definition
/// of 1 standard deviation from the mean, we use same variance percentile from the man
/// as the baseline since it gives a good indicator of what you might actually buy.
/// This is the _average_ price you would pay per item (or per unit) if you bought out the
/// lowest priced 15.87% of units currently listed.
///
/// # Example
/// Suppose the following auctions are listed:
///
/// * `5 @ 6.00g`
/// * `10 @ 5.00g`
/// * `15 @ 5.45g`
/// * `1 @ 4.00g`
///
/// Since there are a total of 31 items up for sale, the bottom 15.87% of these auctions
/// is the average price of the lowest 4 auctions (`floor(31 * 0.1587) = 4`), which
/// comes out to `4.75g`.
///
/// For stackable items, each "unit" of the stack is considered independently so as to calculate
/// the total "quantity" of that item currently available. For unstackable items, the "unit"
/// quantity is always 1.
///
/// This "Market Price" typically represents a snapshot at a single point in time, specifically
/// when an Auction House scan took place.
///
/// # Arguments
///
/// * `prices` - array of item buyout or unit prices for which to calculate the market price.
///
/// # Return Values
///
/// * `Some(u64)` - For when the Market Price is able to be successfully calculated.
/// * `None` - For when the Market Price _cannot_ be mathematically calculated. For example,
///   since Standard Deviation is part of the definition of a Market Price, and you cannot
///   calculate the Standard Deviation of fewer than 2 data points, this method
///   will return `None` when `prices` contains fewer than 2 items.
///
pub fn market_price(prices: &[u64]) -> Option<u64> {
    if prices.is_empty() {
        return None;
    } else if prices.len() == 1 {
        return Some(prices[0]);
    }

    let p_index = percentile_index(FIRST_STANDARD_DEV_PERCENTILE, prices.len());

    let mut v = prices.to_vec();
    v.sort_unstable();

    let calc_prices = &v[0..=p_index];
    let sum: u64 = calc_prices.iter().sum();

    let res = (sum as f64 / calc_prices.len() as f64).round() as u64;
    Some(res)
}

/// An alternate method of calculating market prices which normalizes prices and
/// throws away significant outliers.
///
/// If you're familiar with Trade Skill Master, this calculation is more in line with
/// how "Market Value" is calculated for a snapshot.
///
/// In the context of a realm and a given snapshot of auction data, "Market Price" is defined
/// by this method to mean "the average price of between the bottom 15% and 30% of prices",
/// where the exact percentage of prices considered depends on the curve after the first 15%
/// are considered. This is done so as to filter out unreasonable outliers on either end of
/// the spectrum to get a more realistic view of what one would expect to pay for
/// a normal quantity of this item.
///
/// # Example
/// Suppose the following simplified example:
///
/// * `3 @ 1.00g`
/// * `1 @ 4.00g`
/// * `10 @ 5.00g`
/// * `15 @ 5.45g`
/// * `5 @ 6.00g`
/// * `2 @ 15.00g`
///
/// Since there are a total of 36 auctions up for sale, the bottom 15% of these auctions
/// would be the first 5 auctions, giving us `[1, 1, 1, 4, 5]`. However, we continue looking
/// at more auctions until either we reach 30% of auctions OR a price increases by 20% from
/// its previous value. In this case, 30% of the auctions would give us:
/// `[1, 1, 1, 4, 5, 5, 5, 5, 5, 5]`. Since the last element does not vary more than 20%
/// from its previous element, we consider the entire list.
///
/// Next, we only consider data that is within 1.5 standard deviations of the mean.
/// The standard deviation for this data set ~`1.79164`, and the mean is `3.7`.
/// This means that the final considered data must be within `{1.90836, 5.49164}`, inclusive.
/// This leaves us with `[4, 5, 5, 5, 5, 5, 5]`. The final result is the mean, which is `4.86g`.
///
/// # Arguments
///
/// * `prices` slice of item buyout or unit prices for which to calculate the market price.
///   this list will be mutated by sorting it in place.
///
/// # Return Values
///
/// * `Some(u64)` - For when the Market Price is able to be calculated.
/// * `None` - For when the Market Price _cannot_ be mathematically calculated. For example,
///   since Standard Deviation is part of the definition of a Market Price, and you cannot
///   calculate the Standard Deviation of fewer than 2 data points, this method will return
///   `None` when `prices` contains 0 items. If it contains 1 item, it will return
///   `Some(u64)` for that single item.
pub fn normalized_market_price(prices: &mut [u64]) -> Option<u64> {
    if prices.is_empty() {
        return None;
    } else if prices.len() == 1 {
        return Some(prices[0]);
    }

    prices.sort_unstable();

    // calculate the theoretical index ranges
    let p0_index = percentile_index(MINIMUM_PRICES_PERCENTILE, prices.len());
    let p1_index = percentile_index(MINIMUM_PRICES_PERCENTILE * 2.0, prices.len());

    // start with the first 15%, which will be used as the minimum
    let mut target_index = p0_index;

    // if p1 is > p0, keep going until we hit p1 or the max variance
    if p1_index > p0_index {
        let mut last_num = prices[p0_index];
        for p in &prices[p0_index + 1..=p1_index] {
            let max = (last_num as f64) * 1.2;
            if (*p as f64) < max {
                // also include this number
                target_index += 1;
                last_num = *p;
            } else {
                break;
            }
        }
    }

    let calc_prices = &prices[..=target_index];

    if calc_prices.len() < 2 {
        let sum: u64 = calc_prices.iter().sum();
        return Some((sum as f64 / calc_prices.len() as f64).round() as u64);
    }

    let filtered_prices = normalize_from_std_dev(calc_prices, 1.5);

    Some(mean(filtered_prices).round() as u64)
}

/// Normalize an array of prices based on the standard deviation.
///
/// Any values outside of the standard deviation specified in `std_dvs`
/// will be filtered excluded from the resulting slice.
///
/// **IMPORTANT:** `prices` _must_ be sorted ascending.
///
/// # Arguments
///
/// * `prices` - Array of prices for which to calculate the normalized distribution.
/// * `std_devs` - The number of standard deviations from the mean to include in the
///   normalization.
///
fn normalize_from_std_dev(prices: &[u64], std_dvs: f64) -> &[u64] {
    let mean = mean(prices);
    let target = std_dvs * std_dev(prices, true).unwrap().abs();
    let range = (mean - target, mean + target);

    let mut i0 = 0;
    let mut i1 = prices.len() - 1;

    for (i, p) in prices.iter().enumerate() {
        let f = *p as f64;
        if f < range.0 {
            // advance the minimum index
            i0 = i + 1;
        } else if f > range.1 {
            // the previous number is the target
            // can break since the array is sorted
            i1 = i - 1;
            break;
        }
    }
    &prices[i0..=i1]
}

/// Calculates the maximum index in a collection of length `len` that should be used in order to
/// splice out that percentile of results.
///
/// For example, say an array has `35` elements and you request the `50th` percentile. The
/// value returned will be `16` (17th) element.
///
/// # Arguments
///
/// * `percentile` - The target percentile.
/// * `len` - The length of the collection for which you wish to get the percentile index.
///
/// # Panics
///
/// This function will panic when the inputs cannot possibly generate a valid index:
///
/// * `percentile` is a negative number
/// * `len` is `0`
///
fn percentile_index(percentile: f64, len: usize) -> usize {
    assert!(
        percentile.floor().is_sign_positive(),
        "Cannot calculate a percentile < 0"
    );
    assert!(
        len > 0,
        "Cannot calculate the percentile index of an empty array"
    );

    let percentile_index: f64 = (percentile / 100.0) * len as f64;
    let percentile_index = percentile_index.floor() as usize; // floor() is an integer

    if percentile_index == 0 {
        0
    } else {
        percentile_index - 1
    }
}

/// Calculates the Standard Deviation of the given array of numbers.
///
/// Since you cannot calculate a standard deviation from less than two
/// data points, returns `None` if `numbers` contains less than two numbers.
///
/// # Arguments
///
/// - `numbers` The numbers for which to calculate the standard deviation.
/// - `is_population` Whether the numbers represents the full population or a sample.
///
pub fn std_dev(numbers: &[u64], is_population: bool) -> Option<f64> {
    let len = numbers.len() as f64;

    if len < 2.0 {
        return None;
    }

    let mean = mean(numbers);

    let mut sum: f64 = 0.0;
    for n in numbers {
        sum += (*n as f64 - mean).powi(2);
    }

    if is_population {
        Some((sum / len).sqrt())
    } else {
        Some((sum / (len - 1.0)).sqrt())
    }
}

/// Calculates the mean (average) for a given array of numbers or units.
///
/// # Arguments
///
/// * `numbers` array of numbers for which to calculate the mean.
///
fn mean(numbers: &[u64]) -> f64 {
    numbers.iter().sum::<u64>() as f64 / numbers.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAX_RELATIVE_DIFF: f64 = 0.000000001;

    #[test]
    fn percentile_index_0th_for_1() {
        assert_eq!(0, percentile_index(0.0, 1))
    }

    #[test]
    fn percentile_index_50th_for_1() {
        assert_eq!(0, percentile_index(50.0, 1));
    }

    #[test]
    fn percentile_index_100th_for_1() {
        assert_eq!(0, percentile_index(100.0, 1))
    }

    #[test]
    fn percentile_index_0th_for_50() {
        assert_eq!(0, percentile_index(0.0, 50))
    }

    #[test]
    fn percentile_index_1st_for_50() {
        assert_eq!(0, percentile_index(1.0, 50))
    }

    #[test]
    fn percentile_index_2nd_for_50() {
        assert_eq!(0, percentile_index(2.0, 50))
    }

    #[test]
    fn percentile_index_3rd_for_50() {
        assert_eq!(0, percentile_index(3.0, 50))
    }

    #[test]
    fn percentile_index_4th_for_50() {
        assert_eq!(1, percentile_index(4.0, 50))
    }

    #[test]
    fn percentile_index_50th_for_35() {
        assert_eq!(16, percentile_index(50.0, 35))
    }

    #[test]
    fn percentile_index_15_87th_for_1000() {
        assert_eq!(157, percentile_index(15.87, 1000))
    }

    #[test]
    fn percentile_index_15_87_for_10000() {
        assert_eq!(1585, percentile_index(15.87, 10000))
    }

    #[test]
    fn mean_simple_ordered() {
        assert_eq!(2.0, mean(&[1, 2, 3]));
    }

    #[test]
    fn mean_simple_unordered() {
        assert_eq!(2.0, mean(&[3, 1, 2]));
    }

    #[test]
    fn mean_non_integer_mean() {
        assert_eq!(2.5, mean(&[1, 2, 3, 4]))
    }

    #[test]
    fn std_dev_population_odd_len() {
        assert_relative_eq!(
            0.81649658092,
            std_dev(&mut [1, 2, 3], true).unwrap(),
            max_relative = MAX_RELATIVE_DIFF
        );
    }

    #[test]
    fn std_dev_population_even_len() {
        assert_relative_eq!(
            0.82915619758,
            std_dev(&mut [1, 2, 3, 3], true).unwrap(),
            max_relative = MAX_RELATIVE_DIFF
        );
    }

    #[test]
    fn std_dev_sample_odd_len() {
        assert_eq!(1.0, std_dev(&mut [1, 2, 3], false).unwrap())
    }

    #[test]
    fn std_dev_sample_even_len() {
        assert_relative_eq!(
            0.95742710775,
            std_dev(&mut [1, 2, 3, 3], false).unwrap(),
            max_relative = MAX_RELATIVE_DIFF
        );
    }

    #[test]
    fn std_dev_population_large_arr() {
        let mut arr: [u64; 20] = [
            9, 30, 51, 66, 139, 159, 179, 181, 196, 249, 282, 296, 301, 356, 384, 410, 455, 461,
            475, 481,
        ];
        let res = std_dev(&mut arr, true);
        assert_relative_eq!(
            152.1584700238,
            res.unwrap(),
            max_relative = MAX_RELATIVE_DIFF
        );
    }

    #[test]
    fn std_dev_sample_large_arr() {
        let mut arr: [u64; 20] = [
            9, 30, 51, 66, 139, 159, 179, 181, 196, 249, 282, 296, 301, 356, 384, 410, 455, 461,
            475, 481,
        ];
        let res = std_dev(&mut arr, false).unwrap();
        assert_relative_eq!(156.111296330, res, max_relative = MAX_RELATIVE_DIFF);
    }

    #[test]
    fn std_dev_invalid_array_size() {
        let mut arr0: [u64; 0] = [];
        let mut arr1: [u64; 1] = [1];

        let res0 = std_dev(&mut arr0, false);
        assert!(res0.is_none());

        let res1 = std_dev(&mut arr1, false);
        assert!(res1.is_none());
    }

    #[test]
    fn market_price_large_data() {
        let arr: [u64; 31] = [
            60000, 60000, 60000, 60000, 60000, 50000, 50000, 50000, 50000, 50000, 50000, 50000,
            50000, 50000, 50000, 54500, 54500, 54500, 54500, 54500, 54500, 54500, 54500, 54500,
            54500, 54500, 54500, 54500, 54500, 54500, 40000,
        ];
        let res = market_price(&arr).unwrap();
        assert_eq!(47500, res);
    }

    #[test]
    fn market_price_empty_array() {
        let res = market_price(&[]);
        assert!(res.is_none());
    }

    #[test]
    fn market_price_single_item() {
        let res = market_price(&[5112]);
        assert_eq!(5112, res.unwrap());
    }

    #[test]
    fn normalized_market_price_large_array() {
        let mut arr: [u64; 24] = [
            50000, 130000, 130000, 150000, 150000, 150000, 160000, 170000, 170000, 190000, 200000,
            200000, 200000, 200000, 200000, 200000, 210000, 210000, 290000, 450000, 450000, 460000,
            470000, 1000000,
        ];
        let res = normalized_market_price(&mut arr).unwrap();
        assert_eq!(145000, res)
    }

    #[test]
    fn normalized_market_price_empty_array() {
        let res = normalized_market_price(&mut []);
        assert!(res.is_none());
    }

    #[test]
    fn normalized_market_price_single_item() {
        let res = normalized_market_price(&mut [10000]).unwrap();
        assert_eq!(10000, res);
    }
}
