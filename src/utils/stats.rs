/// 15.87 represents -1 standard deviation from the mean of a normal distribution curve
const FIRST_STANDARD_DEV_PERCENTILE: f64 = 15.87;

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

/// Calculates the Market Price given an array of item buyout our unit prices.
///
/// In the context of a realm, "Market Price" is defined as the bottom 1 standard
/// deviation of all auctions currently listed for that item. In strict numerical terms,
/// the bottom 1 standard deviation of the bottom `15.87%` of auctions. This is the
/// _average_ price you would pay per item (or per unit) if you bought out the
/// lowest priced 15.87% of units currently listed.
///
/// Suppose the following auctions are listed:
///
/// - `5 @ 6.00g`
/// - `10 @ 5.00g`
/// - `15 @ 5.45g`
/// - `1 @ 4.00g`
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
/// - `prices` - array of item buyout or unit prices for which to calculate the market price.
///
/// # Return Values
///
/// - `Some(u64)` - For when the Market Price is able to be successfully calculated.
/// - `None` - For when the Market Price _cannot_ be mathematically calculated. For example,
///   since Standard Deviation is part of the definition of a Market Price, and you cannot
///   calculate the Standard Deviation of fewer than 2 data points, this method
///   will return `None` when `prices` contains fewer than 2 items.
///
pub fn market_price(prices: &[u64]) -> Option<u64> {
    if prices.len() < 2 {
        return None;
    }

    let p_index = percentile_index(FIRST_STANDARD_DEV_PERCENTILE, prices.len() as usize);

    let mut v = prices.to_vec();
    v.sort();

    let calc_prices = &v[0..=p_index];
    let sum: u64 = calc_prices.iter().sum();

    let res = (sum as f64 / calc_prices.len() as f64).round() as u64;
    Some(res)
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

/// Calculates the mean (average) for a given array of numbers or units.
///
/// # Arguments
///
/// * `numbers` array of numbers for which to calculate the mean.
///
pub fn mean(numbers: &[u64]) -> f64 {
    numbers.iter().sum::<u64>() as f64 / numbers.len() as f64
}

/// Calculates the median for a given array of numbers or units.
///
/// The numbers array is mutated by sorting it as part of the calculation.
pub fn median(numbers: &mut [u64]) -> u64 {
    numbers.sort();
    let mid = numbers.len() / 2;
    numbers[mid]
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
    fn median_sorted_odd() {
        assert_eq!(2, median(&mut [1, 2, 3]));
    }

    #[test]
    fn median_sorted_even() {
        assert_eq!(3, median(&mut [1, 2, 3, 4]));
    }

    #[test]
    fn median_unsorted_odd() {
        assert_eq!(3, median(&mut [5, 7, 2, 1, 3]))
    }

    #[test]
    fn median_unsorted_even() {
        assert_eq!(5, median(&mut [5, 7, 1, 2, 6, 4]))
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
}
