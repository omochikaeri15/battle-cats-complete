use std::fmt;

use serde::Serialize;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, serde::Deserialize)]
pub enum Scale {
    #[default]
    Raw,
    Double,
    Quarter,
}

impl Scale {
    pub const fn apply(self, value: i32) -> i32 {
        match self {
            Self::Raw => value,
            Self::Double => value.saturating_mul(2),
            Self::Quarter => value / 4,
        }
    }
}

pub trait FromColumn: Sized {
    fn from_column(text: &str) -> Option<Self>;

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

pub fn parse_cell<F: FromColumn>(cell: Option<&str>, default: &str, scale: Scale) -> Option<F> {
    cell.map(str::trim)
        .and_then(F::from_column)
        .or_else(|| F::from_column(default))
        .map(|value| value.scaled(scale))
}

#[derive(Serialize)]
#[serde(bound = "")]
pub struct Column<T> {
    pub field: &'static str,
    pub index: usize,
    pub scale: Scale,
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

    pub fn read(&self, row: &[&str], target: &mut T) {
        (self.store)(target, row.get(self.index).copied());
    }
}

pub fn apply<T>(row: &[&str], table: &[Column<T>], target: &mut T) -> usize {
    let mut widest = 0;

    for column in table {
        widest = widest.max(column.index);
        column.read(row, target);
    }

    widest + 1
}

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
