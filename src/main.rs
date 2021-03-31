use etherealpost::stats::market_price;

fn main() {
    let prices: [u64; 3] = [60000, 60000, 50000];
    let market_price = market_price(&prices);
    println!(
        "The market price of {:?} is: {}",
        prices,
        market_price.unwrap()
    );
}
