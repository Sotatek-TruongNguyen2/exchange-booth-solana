pub fn calc_output_amount(input_amount: u64, rate: u32, reverse: bool) -> Option<u64> {
    match reverse {
        true => input_amount.checked_mul(rate as u64),
        _ => input_amount.checked_div(rate as u64),
    }
}
