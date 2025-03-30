use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};
use std::{
    collections::HashSet,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

use crate::max_random_biguint;
use crate::{max_timeflake_biguint, Timeflake, MAX_TIMESTAMP};

#[test]
fn test_random() {
    let mut rng = rand::rng();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as u64;

    for _ in 0..1000 {
        let flake = Timeflake::new_random(&mut rng);
        let timestamp = flake.timestamp();
        let random = flake.random();

        assert!(timestamp >= now, "Timestamp should be >= current time");
        assert!(timestamp <= MAX_TIMESTAMP, "Timestamp out of range");

        assert!(random.to_u128().is_some(), "Random component should be convertible to u128");
        let rand_value = random.to_u128().unwrap();
        assert!(rand_value <= max_random_biguint().to_u128().unwrap(), "Random value out of range");

        assert!(
            flake.to_bigint() >= &BigUint::zero(),
            "Flake int representation should be non-negative"
        );
        assert!(
            flake.to_bigint() <= &max_timeflake_biguint(),
            "Flake int representation out of range"
        );
    }
}

#[test]
fn test_from_values_timestamp_only() {
    let now = 123u64;

    for _ in 0..1000 {
        let flake = Timeflake::from_components(now, &BigUint::zero()).unwrap();

        let timestamp = flake.timestamp();
        let random = flake.random();

        assert_eq!(timestamp, now, "Timestamp should match the provided value");
        assert!(random.is_zero(), "Random component should be zero");

        assert!(
            flake.to_bigint() >= &BigUint::zero(),
            "Flake int representation should be non-negative"
        );
        assert!(
            flake.to_bigint() <= &max_timeflake_biguint(),
            "Flake int representation out of range"
        );

        assert!(random.to_u128().is_some(), "Random component should be convertible to u128");
        let rand_value = random.to_u128().unwrap();
        assert!(rand_value <= max_random_biguint().to_u128().unwrap(), "Random value out of range");
    }
}

#[test]
fn test_from_values_timestamp_and_random() {
    let now = 123u64;
    let rand = 456u128;
    let random_biguint = BigUint::from(rand);

    for _ in 0..1000 {
        let flake = Timeflake::from_components(now, &random_biguint).unwrap();

        let timestamp = flake.timestamp();
        let random = flake.random();

        assert_eq!(timestamp, now, "Timestamp should match the provided value");

        assert_eq!(
            random.to_u128().unwrap(),
            rand,
            "Random component should match the provided value"
        );

        assert!(
            flake.to_bigint() >= &BigUint::zero(),
            "Flake int representation should be non-negative"
        );
        assert!(
            flake.to_bigint() <= &max_timeflake_biguint(),
            "Flake int representation out of range"
        );

        assert!(random.to_u128().is_some(), "Random component should be convertible to u128");
        let rand_value = random.to_u128().unwrap();
        assert!(rand_value <= max_random_biguint().to_u128().unwrap(), "Random value out of range");
    }
}

#[test]
fn test_parse_base62_and_conversions() {
    let base62_str = "02i1KoFfY3auBS745gImbZ";
    let flake = Timeflake::from_base62(base62_str).unwrap();

    assert_eq!(flake.timestamp(), 1579091935216, "Timestamp should be 1579091935216");

    let expected_random = BigUint::parse_bytes(b"724773312193627487660233", 10).unwrap();
    assert_eq!(flake.random(), expected_random, "Random component mismatch");

    let expected_int_value =
        BigUint::parse_bytes(b"1909005012028578488143182045514754249", 10).unwrap();
    assert_eq!(flake.to_bigint(), &expected_int_value, "Flake int representation mismatch");
    assert_eq!(flake.to_hex(), "016fa936bff0997a0a3c428548fee8c9", "Hex representation mismatch");
    assert_eq!(flake.to_base62(), base62_str, "Base62 representation mismatch");
    assert_eq!(
        flake.to_bytes(),
        b"\x01o\xa96\xbf\xf0\x99z\n<B\x85H\xfe\xe8\xc9",
        "Byte representation mismatch"
    );

    let expected_uuid = Uuid::parse_str("016fa936-bff0-997a-0a3c-428548fee8c9").unwrap();
    assert_eq!(flake.to_uuid(), expected_uuid, "UUID representation mismatch");
}

#[test]
fn test_parse_bytes_and_conversions() {
    let byte_data: [u8; 16] = [
        0x01, 0x6f, 0xa9, 0x36, 0xbf, 0xf0, 0x99, 0x7a, 0x0a, 0x3c, 0x42, 0x85, 0x48, 0xfe, 0xe8,
        0xc9,
    ];
    let flake = Timeflake::from_bytes(byte_data).unwrap();

    assert_eq!(flake.timestamp(), 1579091935216, "Timestamp should be 1579091935216");

    let expected_random = BigUint::parse_bytes(b"724773312193627487660233", 10).unwrap();
    assert_eq!(flake.random(), expected_random, "Random component mismatch");

    let expected_int_value =
        BigUint::parse_bytes(b"1909005012028578488143182045514754249", 10).unwrap();
    assert_eq!(flake.to_bigint(), &expected_int_value, "Flake int representation mismatch");
    assert_eq!(flake.to_hex(), "016fa936bff0997a0a3c428548fee8c9", "Hex representation mismatch");
    assert_eq!(flake.to_base62(), "02i1KoFfY3auBS745gImbZ", "Base62 representation mismatch");
    assert_eq!(flake.to_bytes(), &byte_data, "Byte representation mismatch");

    let expected_uuid = Uuid::parse_str("016fa936-bff0-997a-0a3c-428548fee8c9").unwrap();
    assert_eq!(flake.to_uuid(), expected_uuid, "UUID representation mismatch");
}

#[test]
fn test_parse_hex_and_conversions() {
    let hex_str = "016fa936bff0997a0a3c428548fee8c9";
    let byte_data = hex::decode(hex_str).unwrap();
    let flake = Timeflake::from_bytes(byte_data.clone().try_into().unwrap()).unwrap();

    assert_eq!(flake.timestamp(), 1579091935216, "Timestamp should be 1579091935216");

    let expected_random = BigUint::parse_bytes(b"724773312193627487660233", 10).unwrap();
    assert_eq!(flake.random(), expected_random, "Random component mismatch");

    let expected_int_value =
        BigUint::parse_bytes(b"1909005012028578488143182045514754249", 10).unwrap();
    assert_eq!(flake.to_bigint(), &expected_int_value, "Flake int representation mismatch");
    assert_eq!(flake.to_hex(), hex_str, "Hex representation mismatch");
    assert_eq!(flake.to_base62(), "02i1KoFfY3auBS745gImbZ", "Base62 representation mismatch");
    assert_eq!(flake.to_bytes().to_vec(), byte_data, "Byte representation mismatch");

    let expected_uuid = Uuid::parse_str("016fa936-bff0-997a-0a3c-428548fee8c9").unwrap();
    assert_eq!(flake.to_uuid(), expected_uuid, "UUID representation mismatch");
}

#[test]
fn test_parse_int_and_conversions() {
    let int_value = BigUint::parse_bytes(b"1909005012028578488143182045514754249", 10).unwrap();
    let flake = Timeflake::from_bigint(int_value.clone()).unwrap();

    assert_eq!(flake.timestamp(), 1579091935216, "Timestamp should be 1579091935216");

    let expected_random = BigUint::parse_bytes(b"724773312193627487660233", 10).unwrap();
    assert_eq!(flake.random(), expected_random, "Random component mismatch");
    assert_eq!(flake.to_bigint(), &int_value, "Flake int representation mismatch");
    assert_eq!(flake.to_hex(), "016fa936bff0997a0a3c428548fee8c9", "Hex representation mismatch");
    assert_eq!(flake.to_base62(), "02i1KoFfY3auBS745gImbZ", "Base62 representation mismatch");

    assert_eq!(
        flake.to_bytes(),
        &[
            0x01, 0x6f, 0xa9, 0x36, 0xbf, 0xf0, 0x99, 0x7a, 0x0a, 0x3c, 0x42, 0x85, 0x48, 0xfe,
            0xe8, 0xc9
        ],
        "Byte representation mismatch"
    );

    let expected_uuid = Uuid::parse_str("016fa936-bff0-997a-0a3c-428548fee8c9").unwrap();
    assert_eq!(flake.to_uuid(), expected_uuid, "UUID representation mismatch");
}

#[test]
fn test_timestamp_increment() {
    let flake1 = Timeflake::new_random(&mut rand::rng());
    thread::sleep(Duration::from_millis(400));

    let flake2 = Timeflake::new_random(&mut rand::rng());
    thread::sleep(Duration::from_millis(1100));

    let flake3 = Timeflake::new_random(&mut rand::rng());

    assert!(
        flake1.to_bigint() < flake2.to_bigint() && flake2.to_bigint() < flake3.to_bigint(),
        "Flake order should be increasing"
    );

    let ts1 = flake1.timestamp();
    let ts2 = flake2.timestamp();
    let ts3 = flake3.timestamp();
    assert!(ts1 < ts2 && ts2 < ts3, "Timestamps should be strictly increasing");

    let timestamps = vec![ts1, ts2, ts3];
    let unique_timestamps: std::collections::HashSet<_> = timestamps.iter().collect();
    assert_eq!(unique_timestamps.len(), 3, "Timestamps should all be unique");
}

#[test]
fn test_uniqueness() {
    let mut seen = HashSet::new();
    let mut rng = rand::rng();

    for i in 0..1_000_000 {
        let flake = Timeflake::new_random(&mut rng);
        let key = flake.to_base62();

        if seen.contains(&key) {
            panic!("Flake collision found after {} generations", i);
        }
        assert_eq!(key.len(), 22, "Base62 key length should be 22, but got {}", key.len());

        seen.insert(key);
    }
}
