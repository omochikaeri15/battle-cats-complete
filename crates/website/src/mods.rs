use std::cell::RefCell;
use std::rc::Rc;

use tracing::{info, warn};

use kokoro::Lazy;
use kokoro::domains::mods::Package;
use omote::platform::mods::{ModCommand, ModInfo, ModState, Mounted};

use crate::protocol::Reply;
use crate::worker::{self, Engine, Layer};

const MOD_FILE: &str = "uploaded-mod.bcm";
const MOD_INFO: &str = "uploaded-mod.info";
const MOD_KEY: &str = "mod";

thread_local! {
    static HELD: RefCell<Option<Option<ModInfo>>> = const { RefCell::new(None) };
    static SPARE: RefCell<Option<Rc<[u8]>>> = const { RefCell::new(None) };
    static MOUNTED: RefCell<Option<Mounted>> = const { RefCell::new(None) };
}

pub(crate) const STORED: [&str; 2] = [MOD_FILE, MOD_INFO];

pub(crate) async fn command(engine: &Rc<Engine>, command: ModCommand) -> Reply {
    let outcome = match command {
        ModCommand::Held => Ok(()),
        ModCommand::Upload { name, bytes } => upload(engine, name, bytes).await,
        ModCommand::Enable => enable(engine).await,
        ModCommand::Disable => {
            disable(engine);

            Ok(())
        }
        ModCommand::Delete => delete(engine).await,
    };

    match outcome {
        Ok(()) => Reply::Mods(ModState { held: held(engine).await, mounted: MOUNTED.with(|mounted| mounted.borrow().clone()) }),
        Err(reason) => {
            warn!("A mod command failed: {reason}");

            Reply::Failed(reason)
        }
    }
}

async fn held(engine: &Engine) -> Option<ModInfo> {
    if let Some(known) = HELD.with(|held| held.borrow().clone()) {
        return known;
    }

    let stored = worker::recall(engine, MOD_INFO).await.and_then(|bytes| {
        postcard::from_bytes::<ModInfo>(&bytes).inspect_err(|err| warn!("The uploaded mod's details could not be read back: {err}")).ok()
    });

    HELD.with(|held| *held.borrow_mut() = Some(stored.clone()));

    stored
}

async fn upload(engine: &Rc<Engine>, name: String, bytes: Vec<u8>) -> Result<(), String> {
    let package = Package::open(&bytes, |_| {})?;
    let metadata = package.metadata;
    let info = ModInfo {
        name,
        title: metadata.title,
        author: metadata.author,
        version: metadata.version,
        package: metadata.package,
        description: metadata.description,
        files: package.files.len(),
        bytes: bytes.len() as u64,
    };

    disable(engine);

    let encoded = postcard::to_allocvec(&info).map_err(|err| format!("the mod's details could not be encoded: {err}"))?;
    let kept = match worker::keep(engine, MOD_FILE, &bytes).await {
        Ok(()) => worker::keep(engine, MOD_INFO, &encoded).await,
        Err(reason) => Err(reason),
    };

    match kept {
        Ok(()) => SPARE.with(|spare| *spare.borrow_mut() = None),
        Err(reason) => {
            warn!("The uploaded mod could not be kept in browser storage and lasts only until the page closes: {reason}");
            SPARE.with(|spare| *spare.borrow_mut() = Some(Rc::from(bytes)));
        }
    }

    info!(name = info.name, files = info.files, bytes = info.bytes, "A mod was uploaded");
    HELD.with(|held| *held.borrow_mut() = Some(Some(info)));

    Ok(())
}

async fn enable(engine: &Rc<Engine>) -> Result<(), String> {
    if MOUNTED.with(|mounted| mounted.borrow().is_some()) {
        return Ok(());
    }

    let spare = SPARE.with(|spare| spare.borrow().clone());
    let stored;
    let bytes: &[u8] = match &spare {
        Some(bytes) => bytes,
        None => {
            stored = worker::recall(engine, MOD_FILE).await.ok_or("there is no uploaded mod to enable")?;
            &stored
        }
    };
    let package = Package::open(bytes, |_| {})?;

    for conflict in &package.conflicts {
        warn!(key = %conflict, "duplicate filename in mod, all copies excluded");
    }

    let names: Vec<Box<str>> = package.files.iter().map(|(name, _)| name.clone()).collect();
    let mounted = Mounted { names, withheld: package.conflicts };

    engine.loader.relayer(Layer { files: package.files.into_iter().collect(), withheld: mounted.withheld.iter().cloned().collect() });
    engine.vault.vfs.destroy(Lazy { key: MOD_KEY, names: Vec::new(), withheld: Vec::new() });

    if let Err(err) = engine.vault.vfs.create(Lazy { key: MOD_KEY, names: mounted.names.clone(), withheld: mounted.withheld.clone() }) {
        engine.loader.relayer(Layer::default());

        return Err(format!("the mod could not be mounted: {err}"));
    }

    settle(engine, &mounted);
    info!(files = mounted.names.len(), withheld = mounted.withheld.len(), "The uploaded mod was enabled");
    MOUNTED.with(|held| *held.borrow_mut() = Some(mounted));

    Ok(())
}

fn disable(engine: &Rc<Engine>) {
    let Some(mounted) = MOUNTED.with(|mounted| mounted.borrow_mut().take()) else {
        return;
    };

    engine.loader.relayer(Layer::default());
    engine.vault.vfs.destroy(Lazy { key: MOD_KEY, names: Vec::new(), withheld: Vec::new() });
    settle(engine, &mounted);
    info!("The uploaded mod was disabled");
}

async fn delete(engine: &Rc<Engine>) -> Result<(), String> {
    disable(engine);
    SPARE.with(|spare| *spare.borrow_mut() = None);
    HELD.with(|held| *held.borrow_mut() = Some(None));

    for name in STORED {
        if let Err(reason) = worker::discard(engine, name).await {
            warn!("{name} could not be dropped from browser storage: {reason}");
        }
    }

    info!("The uploaded mod was deleted");

    Ok(())
}

fn settle(engine: &Rc<Engine>, mounted: &Mounted) {
    let mut touched = mounted.names.clone();

    touched.extend(mounted.withheld.iter().cloned());
    engine.vault.purge(&touched);
    worker::forget_sheets();
    crate::arena::remount();
}
