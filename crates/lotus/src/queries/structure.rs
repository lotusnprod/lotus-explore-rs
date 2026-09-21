// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Structure-string classification and escaping for SPARQL embedding.
//!
//! Chemical structures may arrive as SMILES, MDL Molfile V2000, or V3000.
//! This module provides [`classify_structure`] to detect the format and
//! [`escape_structure_literal`] to safely embed the result in a SPARQL string.

/// Classifies a structure string by its chemical format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureKind {
    /// No structure provided.
    Empty,
    /// SMILES string.
    Smiles,
    /// MDL Molfile V2000 format.
    MolfileV2000,
    /// MDL Molfile V3000 format.
    MolfileV3000,
}

impl StructureKind {
    /// Human-readable label for UI display.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Empty => "—",
            Self::Smiles => "SMILES",
            Self::MolfileV2000 => "Molfile V2000",
            Self::MolfileV3000 => "Molfile V3000",
        }
    }
}

/// Classify a raw structure string into a [`StructureKind`].
///
/// Detection rules:
/// - `M  END` marker → Molfile (V2000 or V3000 based on `V2000`/`V3000` tag or `V3000`/`BEGIN CTAB`)
/// - Otherwise → SMILES
#[must_use]
pub fn classify_structure(text: &str) -> StructureKind {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return StructureKind::Empty;
    }
    let upper = text.to_ascii_uppercase();
    let has_end = upper.contains("M  END");
    if has_end && (upper.contains("V3000") || upper.contains("BEGIN CTAB")) {
        return StructureKind::MolfileV3000;
    }
    if has_end && upper.contains("V2000") {
        return StructureKind::MolfileV2000;
    }
    StructureKind::Smiles
}

/// Returns `true` if the text looks like an MDL Molfile V2000 or V3000.
fn looks_like_molfile(text: &str) -> bool {
    matches!(
        classify_structure(text),
        StructureKind::MolfileV2000 | StructureKind::MolfileV3000
    )
}

/// Escape a SMILES or Molfile string for safe embedding in a SPARQL literal.
///
/// Molfiles (multi-line) are wrapped in triple-quoted strings; SMILES strings
/// in double-quoted strings with escaped inner quotes and backslashes.
#[must_use]
pub fn escape_structure_literal(smiles: &str) -> String {
    let normalized = smiles.replace("\r\n", "\n").replace('\r', "\n");
    let is_molfile = looks_like_molfile(&normalized);
    let candidate = if is_molfile {
        normalized
    } else {
        normalized.trim().to_string()
    };

    let escaped_bs = candidate.replace('\\', r"\\");
    if is_molfile || candidate.contains('\n') {
        format!("'''{escaped_bs}'''")
    } else {
        let escaped = escaped_bs.replace('"', r#"\""#);
        format!("\"{escaped}\"")
    }
}
