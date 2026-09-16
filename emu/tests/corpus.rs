use std::fs;
use std::path::{Path, PathBuf};

use emu::engine::{get_column_count, read_csv_cell, read_csv_row, AssetStream};

fn corpus() -> Option<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../.cargo/game");

    root.is_dir().then_some(root)
}

fn every_csv(root: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            every_csv(&path, found);
        } else if path.extension().is_some_and(|ext| ext == "csv") {
            found.push(path);
        }
    }
}

// The splitter is checked against the shipped data rather than against itself: rejoining the
// cells has to give back the line the game saw. The one legal difference is a row that ends
// on a delimiter, which does not produce the trailing empty cell.
#[test]
fn every_shipped_csv_row_rejoins_into_the_line_it_came_from() {
    let Some(root) = corpus() else {
        eprintln!("corpus absent, skipping");
        return;
    };

    let mut files = Vec::new();
    every_csv(&root, &mut files);

    assert!(files.len() > 1000, "found only {} csv files", files.len());

    let mut rows = 0u64;

    for path in &files {
        let Ok(bytes) = fs::read(path) else {
            continue;
        };

        let mut stream = AssetStream::new(&bytes, b'\n');

        while read_csv_row(&mut stream) {
            let joined: Vec<&[u8]> = stream
                .cells
                .iter()
                .map(|cell| &stream.bytes[cell.at..cell.at + cell.len])
                .collect();

            let rebuilt = joined.join(&b','[..]);
            let first = stream.cells.first().map_or(0, |cell| cell.at);
            let last = stream.cells.last().map_or(0, |cell| cell.at + cell.len);
            let line = &stream.bytes[first..last];

            assert_eq!(rebuilt, line, "{} row {rows} did not rejoin", path.display());
            rows += 1;
        }
    }

    assert!(rows > 10_000, "only walked {rows} rows");
}

// 804 enemy rows is the loader's own table height, so a line splitter that miscounts by one
// anywhere in a 60 000-line file shows up here.
#[test]
fn the_enemy_table_has_the_row_count_its_loader_expects() {
    let Some(root) = corpus() else {
        eprintln!("corpus absent, skipping");
        return;
    };

    let Ok(bytes) = fs::read(root.join("enemies/t_unit.csv")) else {
        eprintln!("enemy table absent, skipping");
        return;
    };

    let mut stream = AssetStream::new(&bytes, b'\n');
    let mut rows = 0;

    while read_csv_row(&mut stream) {
        rows += 1;
    }

    assert_eq!(rows, 804);
}

// A cat row carries a trailing `, // <name>` comment that nothing strips, so the column count
// is ragged and the comment column has to read as zero rather than fault.
#[test]
fn a_trailing_comment_column_reads_as_zero_and_leaves_the_row_ragged() {
    let Some(root) = corpus() else {
        eprintln!("corpus absent, skipping");
        return;
    };

    let Ok(bytes) = fs::read(root.join("cats/000/unit001.csv")) else {
        eprintln!("unit001 absent, skipping");
        return;
    };

    let mut stream = AssetStream::new(&bytes, b'\n');

    assert!(read_csv_row(&mut stream));

    let commented = get_column_count(&stream);
    assert_eq!(read_csv_cell(&stream, 0), 100, "the first stat still reads");
    assert_eq!(read_csv_cell(&stream, commented as i32 - 1), 0, "the comment column reads as zero");
    assert_eq!(read_csv_cell(&stream, 9_999), 0, "and a column past the end reads as zero too");

    assert!(read_csv_row(&mut stream));

    assert!(
        get_column_count(&stream) < commented,
        "the second row is shorter, because only the first carries the comment",
    );
}
