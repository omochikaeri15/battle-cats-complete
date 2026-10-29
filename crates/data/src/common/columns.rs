//! Table-driven declaration of a flat row's column layout.
//!
//! A parser declares its layout once as a slice of [`Column`] and reads a row by
//! walking that table. The table can be published, so a consumer reads the
//! mapping from the parser itself rather than restating it.

use std::fmt;

use serde::Serialize;

/// The arithmetic a raw column value passes through on its way into a field.
///
/// The engine stores some values in units the rest of the crate does not use:
/// a few durations are recorded at half their frame count, and several
/// distances are quadrupled. The conversion is part of a column's definition
/// rather than of the field it lands in, since the same field may be scaled in
/// one layout and raw in another.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, serde::Deserialize)]
pub enum Scale {
    /// The column is stored exactly as it is read.
    #[default]
    Raw,
    /// The column is doubled, converting a half-frame duration into frames.
    Double,
    /// The column is divided by four, converting a quadrupled distance into engine units.
    Quarter,
}

impl Scale {
    /// Applies the conversion to one raw column value.
    ///
    /// # Arguments
    /// * `value` - The integer read from the column, or the column's default when it was absent or unparseable.
    ///
    /// # Returns
    /// An `i32` holding the converted value.
    pub const fn apply(self, value: i32) -> i32 {
        match self {
            Self::Raw => value,
            Self::Double => value.saturating_mul(2),
            Self::Quarter => value / 4,
        }
    }
}

/// A field type a column's text can be read into.
///
/// The implementation decides what the column's text means for the field, so a
/// column falls back to its declared default in the field's own type rather
/// than in a single transport type every field must be squeezed through.
pub trait FromColumn: Sized {
    /// Reads a column's text into this type.
    ///
    /// # Arguments
    /// * `text` - The trimmed text of the column, or of the column's declared default.
    ///
    /// # Returns
    /// An `Option` holding the value the text names, or `None` when it names none.
    fn from_column(text: &str) -> Option<Self>;

    /// Applies a column's conversion to a value of this type.
    ///
    /// # Arguments
    /// * `scale` - The conversion the column declares.
    ///
    /// # Returns
    /// The converted value, which for a type no conversion applies to is the value unchanged.
    fn scaled(self, _scale: Scale) -> Self {
        self
    }
}

macro_rules! integral {
    ($($target:ty),*) => {$(
        impl FromColumn for $target {
            fn from_column(text: &str) -> Option<Self> {
                text.parse().ok()
            }

            fn scaled(self, scale: Scale) -> Self {
                match scale {
                    Scale::Raw => self,
                    Scale::Double => self.saturating_mul(2),
                    Scale::Quarter => self / 4,
                }
            }
        }
    )*};
}

macro_rules! floating {
    ($($target:ty),*) => {$(
        impl FromColumn for $target {
            fn from_column(text: &str) -> Option<Self> {
                text.parse().ok()
            }

            fn scaled(self, scale: Scale) -> Self {
                match scale {
                    Scale::Raw => self,
                    Scale::Double => self * 2.0,
                    Scale::Quarter => self / 4.0,
                }
            }
        }
    )*};
}

integral!(i8, i16, i32, i64, u8, u16, u32, u64, usize);
floating!(f32, f64);

impl FromColumn for bool {
    fn from_column(text: &str) -> Option<Self> {
        text.parse::<i64>().ok().map(|value| value == 1)
    }
}

impl FromColumn for String {
    fn from_column(text: &str) -> Option<Self> {
        Some(text.to_owned())
    }
}

impl<T: FromColumn> FromColumn for Option<T> {
    fn from_column(text: &str) -> Option<Self> {
        Some(T::from_column(text))
    }

    fn scaled(self, scale: Scale) -> Self {
        self.map(|value| value.scaled(scale))
    }
}

/// Reads one column's text into a field's own type.
///
/// The column's declared default stands in when the row does not reach the
/// column or its text names no value, and is read in the same type by the same
/// rules. A default that names no value either leaves the field alone.
///
/// # Arguments
/// * `cell` - The raw text at the column's index, or `None` when the row stops short of it.
/// * `default` - The text the column falls back to.
/// * `scale` - The conversion the column declares.
///
/// # Returns
/// An `Option` holding the value to store, or `None` when neither the column
/// nor its default names one.
pub fn parse_cell<F: FromColumn>(cell: Option<&str>, default: &str, scale: Scale) -> Option<F> {
    cell.map(str::trim)
        .and_then(F::from_column)
        .or_else(|| F::from_column(default))
        .map(|value| value.scaled(scale))
}

/// The definition of one column of a raw row, and the field it lands in.
///
/// A layout's full column mapping is published as a slice of these, so a
/// consumer needing to know which index feeds which field, how it is scaled, or
/// what it falls back to can read that from the same table the parser itself
/// runs on, rather than mirroring the parser by hand.
#[derive(Serialize)]
#[serde(bound = "")]
pub struct Column<T> {
    /// The name of the `T` field this column populates, matching its serialized key.
    pub field: &'static str,
    /// The zero-based position of this column within the raw row.
    pub index: usize,
    /// The conversion applied to the raw value before it is stored.
    pub scale: Scale,
    /// The text used when the column is absent from the row or names no value.
    pub default: &'static str,
    #[serde(skip)]
    store: fn(&mut T, Option<&str>),
}

impl<T> Clone for Column<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Column<T> {}

impl<T> fmt::Debug for Column<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Column")
            .field("field", &self.field)
            .field("index", &self.index)
            .field("scale", &self.scale)
            .field("default", &self.default)
            .finish()
    }
}

impl<T> Column<T> {
    pub(crate) const fn new(
        field: &'static str,
        index: usize,
        scale: Scale,
        default: &'static str,
        store: fn(&mut T, Option<&str>),
    ) -> Self {
        Self { field, index, scale, default, store }
    }

    /// Reads this column out of a raw row and stores it into the target.
    ///
    /// A column the row does not reach, or whose text names no value,
    /// contributes [`Column::default`] instead. A field neither names a value
    /// for keeps whatever it already held.
    ///
    /// # Arguments
    /// * `row` - The already-split columns of one raw row.
    /// * `target` - The structure the column's value is stored into.
    pub fn read(&self, row: &[&str], target: &mut T) {
        (self.store)(target, row.get(self.index).copied());
    }
}

/// Reads every column of a table out of a raw row.
///
/// # Arguments
/// * `row` - The already-split columns of one raw row.
/// * `table` - The layout to apply, in the order it is declared.
/// * `target` - The structure the columns are stored into.
///
/// # Returns
/// A `usize` holding the first index past the widest column the table
/// describes, where a caller retaining trailing columns begins.
pub fn apply<T>(row: &[&str], table: &[Column<T>], target: &mut T) -> usize {
    let mut widest = 0;

    for column in table {
        widest = widest.max(column.index);
        column.read(row, target);
    }

    widest + 1
}

/// Declares a column table as a slice of [`Column`].
///
/// Each entry is `field: index`, optionally followed by a [`Scale`] variant and
/// then the value the column falls back to when it is missing or names no
/// value. A table whose columns share one fallback states it once as a leading
/// `absent <literal>;`, which any entry may still override.
macro_rules! columns {
    (@scale) => { $crate::common::columns::Scale::Raw };
    (@scale $scale:ident) => { $crate::common::columns::Scale::$scale };
    (@default $absent:literal,) => { stringify!($absent) };
    (@default $absent:literal, $default:literal) => { stringify!($default) };
    (@table $absent:literal; $($field:ident : $index:literal $(, $scale:ident $(, $default:literal)?)?);* $(;)?) => {
        &[$($crate::common::columns::Column::new(
            stringify!($field),
            $index,
            $crate::common::columns::columns!(@scale $($scale)?),
            $crate::common::columns::columns!(@default $absent, $($($default)?)?),
            |target, cell| {
                if let Some(value) = $crate::common::columns::parse_cell(
                    cell,
                    $crate::common::columns::columns!(@default $absent, $($($default)?)?),
                    $crate::common::columns::columns!(@scale $($scale)?),
                ) {
                    target.$field = value;
                }
            },
        )),*]
    };
    (absent $absent:literal; $($body:tt)*) => {
        $crate::common::columns::columns!(@table $absent; $($body)*)
    };
    ($($body:tt)*) => {
        $crate::common::columns::columns!(@table 0; $($body)*)
    };
}

pub(crate) use columns;

#[cfg(test)]
pub(crate) fn assert_one_field_per_column<T>(table: &[Column<T>])
where
    T: Default + Serialize,
{
    let width = table.iter().map(|column| column.index).max().map_or(0, |last| last + 1);

    let changed_fields = |sentinel: &str, at: usize| -> Vec<String> {
        let mut cells = vec!["0"; width];
        let mut baseline = T::default();
        apply(&cells, table, &mut baseline);

        cells[at] = sentinel;
        let mut probed = T::default();
        apply(&cells, table, &mut probed);

        let before = serde_json::to_value(&baseline).unwrap();
        let after = serde_json::to_value(&probed).unwrap();

        let (Some(before), Some(after)) = (before.as_object(), after.as_object()) else {
            panic!("a column table's target must serialize as an object");
        };

        before
            .iter()
            .filter(|(key, value)| after.get(*key) != Some(*value))
            .map(|(key, _)| key.clone())
            .collect()
    };

    for column in table {
        let mut reached = false;

        for sentinel in ["1", "4"] {
            let changed = changed_fields(sentinel, column.index);

            assert!(
                changed.is_empty() || changed == [column.field],
                "column {} at index {} changed {:?}",
                column.field,
                column.index,
                changed,
            );

            reached |= !changed.is_empty();
        }

        assert!(reached, "column {} at index {} reached no field", column.field, column.index);
    }
}
