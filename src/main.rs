use bigdecimal::BigDecimal;

fn main() {
    let b = BigDecimal::parse_bytes(b"123444444", 84874774).unwrap();
    println!("{}", b.to_string());
}
