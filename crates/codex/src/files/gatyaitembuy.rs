use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::columns::{self, Column};
use crate::common::file::{self, Separator};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum GatyaItemBuyError {
    EmptyFile,
}

impl fmt::Display for GatyaItemBuyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "The provided file bytes contained no valid item catalog data."),
        }
    }
}

impl std::error::Error for GatyaItemBuyError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GatyaItemBuy {
    pub rarity: i32,
    pub reflect_or_storage: i32,
    pub price: i32,
    pub stage_drop_item_id: i32,
    pub quantity: i32,
    pub sever_id: i32,
    pub category: i32,
    pub index: i32,
    pub src_item_id: i32,
    pub main_menu_type: i32,
    pub gatya_ticket_id: i32,
    pub img_id: i32,
    pub comment: String,
    pub rest: Vec<Option<i32>>,
}

impl Default for GatyaItemBuy {
    fn default() -> Self {
        Self {
            rarity: -1,
            reflect_or_storage: -1,
            price: -1,
            stage_drop_item_id: -1,
            quantity: -1,
            sever_id: -1,
            category: -1,
            index: -1,
            src_item_id: -1,
            main_menu_type: -1,
            gatya_ticket_id: -1,
            img_id: -1,
            comment: String::new(),
            rest: Vec::new(),
        }
    }
}

impl GatyaItemBuy {
    const COMMENT_INDEX: usize = 12;

    pub const COLUMNS: &'static [Column<Self>] = columns::columns! {
        absent -1;
        rarity             : 0;
        reflect_or_storage : 1;
        price              : 2;
        stage_drop_item_id : 3;
        quantity           : 4;
        sever_id           : 5;
        category           : 6;
        index              : 7;
        src_item_id        : 8;
        main_menu_type     : 9;
        gatya_ticket_id    : 10;
        img_id             : 11;
    };

    pub fn icon_index(&self, line: usize) -> usize {
        usize::try_from(self.img_id).unwrap_or(line)
    }

    fn from_csv_line(csv_line: &str, delimiter: char) -> Self {
        let parts: Vec<&str> = strip_comment(csv_line).split(delimiter).map(str::trim).collect();

        let mut row = Self::default();
        columns::apply(&parts, Self::COLUMNS, &mut row);

        if let Some(comment) = parts.get(Self::COMMENT_INDEX) {
            row.comment = (*comment).to_owned();
        }

        row.rest = parts
            .iter()
            .skip(Self::COMMENT_INDEX + 1)
            .map(|part| part.parse::<i32>().ok())
            .collect();

        row
    }

    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Vec<Self>, GatyaItemBuyError> {
        parse_inner(bytes.as_ref(), separator)
    }
}

fn strip_comment(line: &str) -> &str {
    line.split_once("//").map_or(line, |(before_comment, _)| before_comment)
}

fn is_header(line: &str, delimiter: char) -> bool {
    strip_comment(line)
        .split(delimiter)
        .next()
        .is_some_and(|first| first.trim().parse::<i32>().is_err() && !first.trim().is_empty())
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<Vec<GatyaItemBuy>, GatyaItemBuyError> {
    let file_content = file::scrub(bytes);
    let delimiter = file::resolve(separator, &file_content);

    let mut lines = file_content.lines().peekable();
    let _ = lines.next_if(|line| is_header(line, delimiter));

    let mut rows = Vec::new();
    let mut has_content = false;

    for line in lines {
        has_content |= !strip_comment(line).trim().is_empty();
        rows.push(GatyaItemBuy::from_csv_line(line, delimiter));
    }

    if !has_content {
        return Err(GatyaItemBuyError::EmptyFile);
    }

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = "Rarity,reflectORstorage,Price,stageDropItemID,quantity,SeverID,category,index,srcItemID,mainMenuType,gatyaTicketID,imgID,comment";

    const FIRST_ROW: &str = "1,0,1000,0,1,0,3,0,-1,0,-1,-1,スピードアップ";

    const DIVERGENT_ROW: &str = "1,0,1000,18,1,404,1,4,-1,5,97,155,にゃんこ福引2017春";

    fn parse_one(line: &str) -> GatyaItemBuy {
        GatyaItemBuy::parse(format!("{HEADER}\n{line}"), None)
            .map(|mut rows| rows.remove(0))
            .unwrap()
    }

    #[test]
    fn every_column_reaches_a_field_of_its_own() {
        columns::assert_one_field_per_column(GatyaItemBuy::COLUMNS);
    }

    #[test]
    fn a_real_row_lands_column_for_column() {
        let row = parse_one(FIRST_ROW);

        assert_eq!(row.rarity, 1, "rarity");
        assert_eq!(row.reflect_or_storage, 0, "reflect_or_storage");
        assert_eq!(row.price, 1000, "price");
        assert_eq!(row.stage_drop_item_id, 0, "stage_drop_item_id");
        assert_eq!(row.quantity, 1, "quantity");
        assert_eq!(row.sever_id, 0, "sever_id");
        assert_eq!(row.category, 3, "category");
        assert_eq!(row.index, 0, "index");
        assert_eq!(row.src_item_id, -1, "src_item_id");
        assert_eq!(row.main_menu_type, 0, "main_menu_type");
        assert_eq!(row.gatya_ticket_id, -1, "gatya_ticket_id");
        assert_eq!(row.img_id, -1, "img_id");
        assert_eq!(row.comment, "スピードアップ", "comment");
        assert!(row.rest.is_empty());
    }

    #[test]
    fn the_drop_identifier_is_not_the_line() {
        let rows = GatyaItemBuy::parse(format!("{HEADER}\n{FIRST_ROW}\n{DIVERGENT_ROW}"), None).unwrap();

        assert_eq!(rows[0].stage_drop_item_id, 0);
        assert_eq!(rows[1].stage_drop_item_id, 18);
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn an_unnamed_icon_falls_back_to_the_line() {
        let named = parse_one(DIVERGENT_ROW);
        let unnamed = parse_one(FIRST_ROW);

        assert_eq!(named.icon_index(27), 155);
        assert_eq!(unnamed.icon_index(27), 27);
    }

    #[test]
    fn a_header_is_dropped_only_when_the_file_carries_one() {
        let with_header = GatyaItemBuy::parse(format!("{HEADER}\n{FIRST_ROW}"), None).unwrap();
        let without_header = GatyaItemBuy::parse(FIRST_ROW, None).unwrap();

        assert_eq!(with_header.len(), 1);
        assert_eq!(without_header.len(), 1);
        assert_eq!(with_header[0], without_header[0]);
    }

    #[test]
    fn a_blank_or_short_line_holds_its_place_in_the_file() {
        let rows = GatyaItemBuy::parse(format!("{HEADER}\n{FIRST_ROW}\n\n1,0,1000\n{DIVERGENT_ROW}"), None).unwrap();

        assert_eq!(rows.len(), 4);
        assert_eq!(rows[1], GatyaItemBuy::default());
        assert_eq!(rows[2].price, 1000);
        assert_eq!(rows[2].stage_drop_item_id, -1);
        assert_eq!(rows[2].comment, "");
        assert_eq!(rows[3].stage_drop_item_id, 18);
    }

    #[test]
    fn a_trailing_slash_comment_is_cut_before_the_columns() {
        let row = parse_one("1,0,1000,0,1,0,3,0,-1,0,-1,-1,スピードアップ     //unused");

        assert_eq!(row.comment, "スピードアップ");
        assert!(row.rest.is_empty());
    }

    #[test]
    fn a_trailing_column_is_kept_rather_than_dropped() {
        let row = parse_one("1,0,1000,0,1,0,3,0,-1,0,-1,-1,スピードアップ,12,x");

        assert_eq!(row.rest, [Some(12), None]);
    }

    #[test]
    fn a_file_without_rows_is_rejected() {
        assert_eq!(GatyaItemBuy::parse("", None), Err(GatyaItemBuyError::EmptyFile));
        assert_eq!(GatyaItemBuy::parse(HEADER, None), Err(GatyaItemBuyError::EmptyFile));
        assert_eq!(GatyaItemBuy::parse(format!("{HEADER}\n\n\n"), None), Err(GatyaItemBuyError::EmptyFile));
    }
}
