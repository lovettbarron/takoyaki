use serde::{Deserialize, Serialize};
use std::fmt;

/// 1-indexed slot ID for project-level sample references (1..=256).
///
/// ProjectFile uses 1-indexed slot IDs. Attempting to create with 0 or >256
/// is an error. This newtype prevents confusion with 0-indexed bank/marker IDs.
///
/// # FNDN-03 Compliance
/// This enforces the 1-indexed vs. 0-indexed distinction at the type level,
/// preventing off-by-one bugs when translating between project slots and array indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectSlotId(u16);

impl ProjectSlotId {
    /// Create a new `ProjectSlotId` from a 1-indexed value.
    ///
    /// Returns `Err` if value is 0 or greater than 256.
    pub fn new(value: u16) -> Result<Self, &'static str> {
        if value == 0 || value > 256 {
            Err("ProjectSlotId must be in range 1..=256 (1-indexed)")
        } else {
            Ok(ProjectSlotId(value))
        }
    }

    /// Return the raw 1-indexed value.
    pub fn get(&self) -> u16 {
        self.0
    }

    /// Convert to a 0-based array index (subtract 1).
    ///
    /// ProjectSlot(1) → index 0, ProjectSlot(256) → index 255.
    pub fn to_zero_index(&self) -> usize {
        (self.0 - 1) as usize
    }
}

impl fmt::Display for ProjectSlotId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ProjectSlot({})", self.0)
    }
}

/// 0-indexed slot ID for bank-level and marker-level references (0..=255).
///
/// Bank files and marker files use 0-indexed slot IDs. Since u8 already enforces
/// 0..=255, the newtype provides type-level distinction from `ProjectSlotId`
/// rather than additional range validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BankSlotId(u8);

impl BankSlotId {
    /// Create a new `BankSlotId`. All u8 values (0..=255) are valid.
    pub fn new(value: u8) -> Self {
        BankSlotId(value)
    }

    /// Return the raw 0-indexed value.
    pub fn get(&self) -> u8 {
        self.0
    }

    /// Return the slot as a usize for array indexing.
    pub fn to_index(&self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for BankSlotId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BankSlot({})", self.0)
    }
}

/// Bank number — 0-indexed internally (0..=15), displayed as 1-indexed (1..=16).
///
/// OT bank filenames use 1-indexed numbers ("bank01.work" through "bank16.work").
/// Internally, banks are stored 0-indexed. This type manages the conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BankNumber(u8);

impl BankNumber {
    /// Create a `BankNumber` from a 0-indexed internal value (0..=15).
    ///
    /// Returns `Err` if value is greater than 15.
    pub fn new(value: u8) -> Result<Self, &'static str> {
        if value > 15 {
            Err("BankNumber must be in range 0..=15")
        } else {
            Ok(BankNumber(value))
        }
    }

    /// Return the 0-indexed internal value.
    pub fn get(&self) -> u8 {
        self.0
    }

    /// Return the 1-indexed display number (Bank 1 through Bank 16).
    pub fn display_number(&self) -> u8 {
        self.0 + 1
    }

    /// Create from a filename 1-indexed number (1..=16).
    ///
    /// "bank01.work" → `from_filename_number(1)` → internal value 0.
    /// "bank16.work" → `from_filename_number(16)` → internal value 15.
    ///
    /// Returns `Err` if n is 0 or greater than 16.
    pub fn from_filename_number(n: u8) -> Result<Self, &'static str> {
        if n == 0 || n > 16 {
            Err("Bank filename number must be in range 1..=16")
        } else {
            Ok(BankNumber(n - 1))
        }
    }
}

impl fmt::Display for BankNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bank{}", self.display_number())
    }
}
