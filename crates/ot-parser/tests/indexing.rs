use ot_parser::types::{ProjectSlotId, BankSlotId, BankNumber};

#[test]
fn test_project_slot_id_valid_range() {
    assert!(ProjectSlotId::new(1).is_ok());
    assert!(ProjectSlotId::new(128).is_ok());
    assert!(ProjectSlotId::new(256).is_ok());
}

#[test]
fn test_project_slot_id_rejects_zero() {
    assert!(ProjectSlotId::new(0).is_err());
}

#[test]
fn test_project_slot_id_rejects_overflow() {
    assert!(ProjectSlotId::new(257).is_err());
    assert!(ProjectSlotId::new(u16::MAX).is_err());
}

#[test]
fn test_project_slot_id_to_zero_index() {
    let slot = ProjectSlotId::new(1).unwrap();
    assert_eq!(slot.to_zero_index(), 0);
    let slot = ProjectSlotId::new(256).unwrap();
    assert_eq!(slot.to_zero_index(), 255);
}

#[test]
fn test_bank_slot_id_full_range() {
    let _ = BankSlotId::new(0);
    let _ = BankSlotId::new(255);
}

#[test]
fn test_bank_slot_id_index() {
    let slot = BankSlotId::new(0);
    assert_eq!(slot.to_index(), 0);
    let slot = BankSlotId::new(255);
    assert_eq!(slot.to_index(), 255);
}

#[test]
fn test_bank_number_valid_range() {
    assert!(BankNumber::new(0).is_ok());
    assert!(BankNumber::new(15).is_ok());
}

#[test]
fn test_bank_number_rejects_overflow() {
    assert!(BankNumber::new(16).is_err());
}

#[test]
fn test_bank_number_display() {
    let bank = BankNumber::new(0).unwrap();
    assert_eq!(bank.display_number(), 1);
    let bank = BankNumber::new(15).unwrap();
    assert_eq!(bank.display_number(), 16);
}

#[test]
fn test_bank_number_from_filename() {
    let bank = BankNumber::from_filename_number(1).unwrap();
    assert_eq!(bank.get(), 0);
    let bank = BankNumber::from_filename_number(16).unwrap();
    assert_eq!(bank.get(), 15);
    assert!(BankNumber::from_filename_number(0).is_err());
    assert!(BankNumber::from_filename_number(17).is_err());
}
