use std::ffi::CString;
use std::io::BufReader;
use std::num::NonZero;
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use pulseaudio::protocol;
use rodio::mixer::{Mixer, MixerSource, mixer};
use tracing::warn;

const CHANNELS: u16 = 2;
const SAMPLE_RATE: u32 = 44_100;
const SAMPLE_BYTES: usize = 2;
const CLIENT_NAME: &str = "Battle Cats Complete";
const FRAME_BYTES: u32 = CHANNELS as u32 * SAMPLE_BYTES as u32;
const TARGET_BYTES: u32 = SAMPLE_RATE / 25 * FRAME_BYTES;
const REQUEST_BYTES: u32 = SAMPLE_RATE / 100 * FRAME_BYTES;
const AUTH_TAG: u32 = 0;
const NAME_TAG: u32 = 1;
const STREAM_TAG: u32 = 2;

pub struct ServerLink {
    mixer: Mixer,
    stopped: Arc<AtomicBool>,
}

struct Wire {
    socket: BufReader<UnixStream>,
    version: u16,
    channel: u32,
    wanted: usize,
}

impl ServerLink {
    pub fn open() -> Option<Self> {
        let wire = connect()
            .inspect_err(|reason| warn!("emu: sound server unavailable: {reason}"))
            .ok()?;
        let (mixer, mixed) = mixer(NonZero::new(CHANNELS)?, NonZero::new(SAMPLE_RATE)?);
        let stopped = Arc::new(AtomicBool::new(false));
        let watched = Arc::clone(&stopped);
        let spawned = std::thread::Builder::new()
            .name("emu_audio".to_owned())
            .spawn(move || {
                if let Err(reason) = pump(wire, mixed, &watched) {
                    warn!("emu: sound server stream ended: {reason}");
                }
            });

        if let Err(error) = spawned {
            warn!("emu: audio thread could not start: {error}");

            return None;
        }

        Some(Self { mixer, stopped })
    }

    pub fn mixer(&self) -> &Mixer {
        &self.mixer
    }
}

impl Drop for ServerLink {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Relaxed);
    }
}

fn connect() -> Result<Wire, String> {
    let path = pulseaudio::socket_path_from_env().ok_or("no server socket")?;
    let stream = UnixStream::connect(path).map_err(|error| error.to_string())?;
    let mut socket = BufReader::new(stream);
    let cookie = pulseaudio::cookie_path_from_env()
        .and_then(|path| std::fs::read(path).ok())
        .unwrap_or_default();
    let auth = protocol::AuthParams {
        version: protocol::MAX_VERSION,
        supports_shm: false,
        supports_memfd: false,
        cookie,
    };

    protocol::write_command_message(
        socket.get_mut(),
        AUTH_TAG,
        &protocol::Command::Auth(auth),
        protocol::MAX_VERSION,
    )
    .map_err(|error| error.to_string())?;

    let (_, reply) = protocol::read_reply_message::<protocol::AuthReply>(&mut socket, protocol::MAX_VERSION)
        .map_err(|error| error.to_string())?;
    let version = protocol::MAX_VERSION.min(reply.version);
    let mut props = protocol::Props::new();

    props.set(
        protocol::Prop::ApplicationName,
        CString::new(CLIENT_NAME).map_err(|error| error.to_string())?,
    );
    protocol::write_command_message(
        socket.get_mut(),
        NAME_TAG,
        &protocol::Command::SetClientName(props),
        version,
    )
    .map_err(|error| error.to_string())?;
    protocol::read_reply_message::<protocol::SetClientNameReply>(&mut socket, version)
        .map_err(|error| error.to_string())?;

    let mut params = protocol::PlaybackStreamParams {
        sample_spec: protocol::SampleSpec {
            format: protocol::SampleFormat::S16Le,
            channels: CHANNELS as u8,
            sample_rate: SAMPLE_RATE,
        },
        channel_map: protocol::ChannelMap::stereo(),
        cvolume: Some(protocol::ChannelVolume::norm(CHANNELS as u8)),
        sink_name: Some(protocol::DEFAULT_SINK.to_owned()),
        ..Default::default()
    };

    params.buffer_attr.max_length = TARGET_BYTES * 4;
    params.buffer_attr.target_length = TARGET_BYTES;
    params.buffer_attr.pre_buffering = 0;
    params.buffer_attr.minimum_request_length = REQUEST_BYTES;
    params.flags.adjust_latency = true;

    protocol::write_command_message(
        socket.get_mut(),
        STREAM_TAG,
        &protocol::Command::CreatePlaybackStream(params),
        version,
    )
    .map_err(|error| error.to_string())?;

    let (_, info) = protocol::read_reply_message::<protocol::CreatePlaybackStreamReply>(&mut socket, version)
        .map_err(|error| error.to_string())?;

    Ok(Wire {
        socket,
        version,
        channel: info.channel,
        wanted: info.requested_bytes as usize,
    })
}

fn send(wire: &mut Wire, mixed: &mut MixerSource, bytes: usize) -> Result<(), String> {
    let mut block = Vec::with_capacity(bytes);

    for _ in 0..bytes / SAMPLE_BYTES {
        let sample = mixed.next().unwrap_or(0.0).clamp(-1.0, 1.0);

        block.extend_from_slice(&((sample * f32::from(i16::MAX)) as i16).to_le_bytes());
    }

    protocol::write_memblock(wire.socket.get_mut(), wire.channel, &block, 0)
        .map_err(|error| error.to_string())
}

fn pump(mut wire: Wire, mut mixed: MixerSource, stopped: &AtomicBool) -> Result<(), String> {
    let first = wire.wanted;

    send(&mut wire, &mut mixed, first)?;

    while !stopped.load(Ordering::Relaxed) {
        let (_, message) = protocol::read_command_message(&mut wire.socket, wire.version)
            .map_err(|error| error.to_string())?;

        match message {
            protocol::Command::Request(protocol::Request { channel, length }) if channel == wire.channel => {
                send(&mut wire, &mut mixed, length as usize)?;
            }
            protocol::Command::Error(code) => return Err(format!("{code:?}")),
            _ => (),
        }
    }

    Ok(())
}
